use crate::Camera;
use euclid::Vector2D;
use eyre::{ContextCompat, Result};
use glfw::{Action, Key, WindowEvent};
use hecs::{Component, Entity, EntityRef, Ref, RefMut};
use rustaria::api::id::Id;
use rustaria::api::identifier::Identifier;
use rustaria::api::Carrier;
use rustaria::entity::component::{PositionComponent, VelocityComponent};
use rustaria::entity::prototype::EntityPrototype;
use rustaria::entity::EntityWorld;
use rustaria::network::packet::ClientBoundPacket;
use rustaria::network::ClientNetwork;
use rustaria::player::{ClientBoundPlayerPacket, Player, ServerBoundPlayerPacket};
use rustaria::ty::WS;
use std::collections::VecDeque;
use tracing::debug;

const MAX_CORRECTION: f32 = 0.05;

pub(crate) struct PlayerSystem {
    server_player: Option<Entity>,
    base_server_world: EntityWorld,
    prediction_world: EntityWorld,

    send_speed: Vector2D<f32, WS>,

    w: bool,
    a: bool,
    s: bool,
    d: bool,
    zoom: f32,
    speed: Vector2D<f32, WS>,

    unprocessed_events: VecDeque<(u32, Vector2D<f32, WS>)>,
    tick: u32,
    player_entity: Id<EntityPrototype>,
}

impl PlayerSystem {
    pub fn new(carrier: &Carrier) -> Result<Self> {
        Ok(Self {
            server_player: None,
            base_server_world: EntityWorld::new(carrier)?,
            prediction_world: EntityWorld::new(carrier)?,
            send_speed: Default::default(),
            w: false,
            a: false,
            s: false,
            d: false,
            zoom: 10.0,
            speed: Default::default(),
            unprocessed_events: Default::default(),
            tick: 0,
            player_entity: carrier
                .entity
                .identifier_to_id(&Identifier::new("player"))
                .wrap_err("Player where")?,
        })
    }

    pub fn event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Scroll(x, y) => {
                self.zoom += y as f32 / 1.0;
            }
            WindowEvent::Key(key, _, action, _) => {
                match key {
                    Key::W => {
                        self.w = !matches!(action, Action::Release);
                    }
                    Key::A => {
                        self.a = !matches!(action, Action::Release);
                    }
                    Key::S => {
                        self.s = !matches!(action, Action::Release);
                    }
                    Key::D => {
                        self.d = !matches!(action, Action::Release);
                    }
                    _ => {}
                }

                // Compile speed
                self.speed = Vector2D::zero();
                self.speed.x = (self.d as u32 as f32) - (self.a as u32 as f32);
                self.speed.y = (self.w as u32 as f32) - (self.s as u32 as f32);
            }
            _ => {}
        }
    }

    pub fn tick(
        &mut self,
        network: &mut ClientNetwork,
        entity_world: &mut EntityWorld,
    ) -> Result<()> {
        if let Some(entity) = self.check(entity_world) {
            self.send_speed = self.speed * (self.zoom / 30.0);
            self.tick += 1;

            // Send our speed at this tick
            network.send(ServerBoundPlayerPacket::SetDir(self.tick, self.send_speed))?;
            self.unprocessed_events
                .push_front((self.tick, self.send_speed));
            self.send_speed = Vector2D::zero();

            self.correct_offset(entity, entity_world);
        }
        Ok(())
    }

    pub fn packet(
        &mut self,
        packet: ClientBoundPlayerPacket,
        entity_world: &mut EntityWorld,
    ) -> Result<()> {
        match packet {
            ClientBoundPlayerPacket::RespondPos(tick, pos) => {
                if let Some(entity) = self.check(entity_world) {
                    if let Some(pos) = pos {
                        entity_world
                            .storage
                            .get_mut_comp::<PositionComponent>(entity)
                            .unwrap()
                            .pos = pos;
                    }

                    // Remove all events that the server has now applied.
                    while let Some((value_id, speed)) = self.unprocessed_events.pop_back() {
                        // Move the base server entity forward.
                        // This totally ignores if the server sends a different speed, this is intentional.
                        // By this being on the predicted speed we can safely isolate the error amount by doing
                        // self.server_entity - self.base_server_entity, this lets us correct it in a sneaky timeframe.
                        self.base_server_world
                            .storage
                            .get_mut_comp::<VelocityComponent>(entity)
                            .unwrap()
                            .velocity = speed;
                        self.base_server_world.tick();

                        // If we reach the tick that we currently received,
                        // stop as the next events are the ones that the server has not yet seen.
                        if value_id == tick {
                            break;
                        }
                    }

                    // Recompile our prediction
                    self.compile_prediction();
                }
            }
            ClientBoundPlayerPacket::Joined(entity) => {
                debug!("Received joined packet");
                self.server_player = Some(entity);
                entity_world.storage.insert(entity, self.player_entity);
                self.base_server_world
                    .storage
                    .insert(entity, self.player_entity);
                self.prediction_world
                    .storage
                    .insert(entity, self.player_entity);
            }
        }

        Ok(())
    }

    pub fn get_camera(&mut self) -> Camera {
        Camera {
            pos: {
                if let Some(entity) = self.server_player {
                    self.prediction_world
                        .storage
                        .get_comp::<PositionComponent>(entity)
                        .unwrap()
                        .pos
                        .to_array()
                } else {
                    [0., 0.]
                }
            },
            zoom: self.zoom,
        }
    }

    // If the server says a different value try to correct it without freaking the player out.
    fn correct_offset(&mut self, entity: Entity, entity_world: &mut EntityWorld) {
        let server_pos = entity_world
            .storage
            .get_comp::<PositionComponent>(entity)
            .unwrap()
            .pos;
        let mut base_server_pos = self
            .base_server_world
            .storage
            .get_mut_comp::<PositionComponent>(entity)
            .unwrap();
        let mut prediction_pos = self
            .prediction_world
            .storage
            .get_mut_comp::<PositionComponent>(entity)
            .unwrap();

        let server_offset = server_pos - base_server_pos.pos;
        let distance = server_offset.length();

        // If the distance is too big just teleport the donut.
        if distance > 10.0 {
            base_server_pos.pos = server_pos;
            prediction_pos.pos = server_pos;
        } else if distance > 0.0 {
            // Slightly drift the donut.
            let amount = server_offset.clamp_length(0.0, MAX_CORRECTION);
            base_server_pos.pos += amount;
            prediction_pos.pos += amount;
        }
    }

    // When a client receives a packet, rebase the base_server_entity and
    // then apply the events not yet to be responded by the server.
    fn compile_prediction(&mut self) -> Option<()> {
        let entity = self.server_player?;

        // Put prediction on the server value
        self.prediction_world.storage.insert_raw(
            entity,
            self.base_server_world.storage.clone(entity)?.build(),
        );

        // If reconciliation is on, we apply values that the server has not yet processed.
        for (_, speed) in &self.unprocessed_events {
            self.prediction_world
                .storage
                .get_mut_comp::<VelocityComponent>(entity)
                .unwrap()
                .velocity = *speed;
            self.prediction_world.tick();
        }

        self.prediction_world
            .storage
            .get_mut_comp::<VelocityComponent>(entity)
            .unwrap()
            .velocity = self.speed;

        Some(())
    }

    fn check(&mut self, world: &EntityWorld) -> Option<Entity> {
        if let Some(entity) = self.server_player {
            return if world.storage.contains(entity) {
                return Some(entity);
            } else {
                // kill everything
                self.server_player = None;
                self.base_server_world.storage.remove(entity);
                self.prediction_world.storage.remove(entity);
                None
            };
        } else {
            None
        }
    }
}
