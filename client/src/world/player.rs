use std::collections::{HashMap, VecDeque};

use euclid::{vec2, Vector2D};
use eyre::{ContextCompat, Result};
use glfw::{Action, Key, MouseButton, WindowEvent};
use hecs::Entity;
use tracing::debug;

use rustaria::api::id::Id;
use rustaria::api::identifier::Identifier;
use rustaria::api::{Api};
use rustaria::api::prototype::FactoryPrototype;
use rustaria::chunk::storage::ChunkStorage;
use rustaria::chunk::entry::ChunkEntryPrototype;
use rustaria::debug::DummyRenderer;
use rustaria::entity::component::{HumanoidComponent, PositionComponent};
use rustaria::entity::prototype::EntityPrototype;
use rustaria::entity::EntityWorld;
use rustaria::network::{ClientNetwork};
use rustaria::network::packet::ServerBoundPacket;
use rustaria::player::{ClientBoundPlayerPacket, PlayerCommand, ServerBoundPlayerPacket};
use rustaria::ty::world_pos::WorldPos;
use rustaria::ty::WS;

use crate::{Camera, Frontend};

const MAX_CORRECTION: f32 = 0.025;

pub(crate) struct PlayerSystem {
    pub server_player: Option<Entity>,
    base_server_world: EntityWorld,
    prediction_world: EntityWorld,

    send_command: PlayerCommand,

    w: bool,
    a: bool,
    s: bool,
    d: bool,
    jump: bool,
    zoom: f32,

    cursor_x: f32,
    cursor_y: f32,

    speed: PlayerCommand,

    unprocessed_events: VecDeque<(u32, PlayerCommand)>,
    tick: u32,
    player_entity: Id<EntityPrototype>,
    presses: Vec<Press>,
}

pub enum Press {
    Use(f32, f32),
}

impl PlayerSystem {
    pub fn new(api: &Api) -> Result<Self> {
        Ok(Self {
            server_player: None,
            base_server_world: EntityWorld::new(api)?,
            prediction_world: EntityWorld::new(api)?,
            send_command: PlayerCommand::default(),
            w: false,
            a: false,
            s: false,
            d: false,
            jump: false,
            zoom: 10.0,
            cursor_x: 0.0,
            cursor_y: 0.0,
            speed: PlayerCommand::default(),
            unprocessed_events: Default::default(),
            tick: 0,
            player_entity: api.carrier
                .entity
                .identifier_to_id(&Identifier::new("player"))
                .wrap_err("Player where")?,
            presses: vec![],
        })
    }

    pub fn event(&mut self, event: WindowEvent, frontend: &Frontend) {
        match event {
            WindowEvent::Scroll(_, y) => {
                self.zoom += y as f32 / 1.0;
            }
            WindowEvent::CursorPos(x, y) => {
                self.cursor_x = x as f32;
                self.cursor_y = y as f32;
            }
            WindowEvent::MouseButton(button, _, _) => {
                let x = ((((self.cursor_x / frontend.dimensions.0 as f32) - 0.5) * 2.0) / frontend.screen_ratio) * self.zoom;
                let y = ((((frontend.dimensions.1 as f32 - self.cursor_y) / frontend.dimensions.1 as f32) - 0.5) * 2.0) * self.zoom;
                match button {
                    MouseButton::Button1 => {
                        // Use
                        self.presses.push(Press::Use(x, y))
                    }
                    MouseButton::Button2 => {}
                    _ => {}
                }
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
                    Key::Space => {
                        self.jump = !matches!(action, Action::Release);
                    }
                    _ => {}
                }

                // Compile speed
                self.speed.dir = Vector2D::zero();
                self.speed.dir.x = (self.d as u32 as f32) - (self.a as u32 as f32);
                self.speed.dir.y = (self.w as u32 as f32) - (self.s as u32 as f32);
            }
            _ => {}
        }
    }

    pub fn tick(
        &mut self,
        api: &Api,
        network: &mut ClientNetwork,
        entity_world: &mut EntityWorld,
        chunks: &mut ChunkStorage,
    ) -> Result<()> {
        self.prediction_world.tick(api, chunks, &mut DummyRenderer);
        if let Some(entity) = self.check(entity_world) {
            self.send_command.dir = self.speed.dir;
            self.send_command.jumping = self.jump;
            {
                let mut component = entity_world
                    .storage
                    .get_mut_comp::<HumanoidComponent>(entity)
                    .unwrap();
                component.dir = self.send_command.dir;
                component.jumping = self.send_command.jumping;
            }

            self.tick += 1;

            // Send our speed at this tick
            network.send(ServerBoundPlayerPacket::SetMove(
                self.tick,
                self.send_command,
            ))?;
            self.unprocessed_events
                .push_front((self.tick, self.send_command));
            self.send_command.dir = Vector2D::zero();

            {
                let pos = self
                    .prediction_world
                    .storage
                    .get_comp::<PositionComponent>(entity)
                    .unwrap()
                    .pos;

                for press in self.presses.drain(..) {
                    match press {
                        Press::Use(x, y) => {
                            if let Ok(pos) = WorldPos::try_from(vec2::<_, WS>(x, y) + pos) {

                                if let Some(chunk) =  chunks.get_mut(pos.chunk) {
                                    for (id, layer) in chunk.layers.iter_mut() {
                                        let prototype = api.carrier.chunk_layers.get(id);
                                        let entry_id = prototype.registry.identifier_to_id(&Identifier::new("air")).expect("where my air");

                                        layer[pos.entry] = prototype.registry.get(entry_id).create(entry_id);
                                        network.send(ServerBoundPacket::SetChunkEntry(pos, id, entry_id))?;
                                    }
                                }

                            }
                        }
                    }
                }
            }

            self.correct_offset(entity, entity_world);
        }
        Ok(())
    }

    pub fn packet(
	    &mut self,
	    carrier: &Api,
	    packet: ClientBoundPlayerPacket,
	    entity_world: &mut EntityWorld,
	    chunks: &ChunkStorage,
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
                        {
                            let mut entity = self
                                .base_server_world
                                .storage
                                .get_mut_comp::<HumanoidComponent>(entity)
                                .unwrap();
                            entity.dir = speed.dir;
                            entity.jumping = speed.jumping;
                        }
                        self.base_server_world.tick(carrier, chunks, &mut DummyRenderer);

                        // If we reach the tick that we currently received,
                        // stop as the next events are the ones that the server has not yet seen.
                        if value_id == tick {
                            break;
                        }
                    }

                    // Recompile our prediction
                    self.compile_prediction(carrier, chunks);
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

    pub fn get_pos(&self) -> Vector2D<f32, WS> {
        if let Some(entity) = self.server_player {
            self.prediction_world
                .storage
                .get_comp::<PositionComponent>(entity)
                .unwrap()
                .pos
        } else {
            vec2(0.0, 0.0)
        }
    }

    pub fn get_camera(&mut self) -> Camera {
        Camera {
            pos: self.get_pos(),
            zoom: self.zoom,
        }
    }

    // If the server says a different value try to correct it without freaking the player out.
    fn correct_offset(&mut self, entity: Entity, entity_world: &EntityWorld) {
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
    fn compile_prediction(&mut self, carrier: &Api, chunks: &ChunkStorage) -> Option<()> {
        let entity = self.server_player?;

        // Put prediction on the server value
        self.prediction_world.storage.insert_raw(
            entity,
            self.base_server_world.storage.clone(entity)?.build(),
        );

        // If reconciliation is on, we apply values that the server has not yet processed.
        for (_, speed) in &self.unprocessed_events {
            {
                let mut prediction = self
                    .prediction_world
                    .storage
                    .get_mut_comp::<HumanoidComponent>(entity)
                    .unwrap();
                prediction.dir = speed.dir;
                prediction.jumping = speed.jumping;
            }
            self.prediction_world.tick(carrier, chunks, &mut DummyRenderer);
        }

        let mut prediction = self
            .prediction_world
            .storage
            .get_mut_comp::<HumanoidComponent>(entity)
            .unwrap();

        prediction.dir = self.send_command.dir;
        prediction.jumping = self.send_command.jumping;
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
