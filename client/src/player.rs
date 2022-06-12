use euclid::Vector2D;
use eyre::Result;
use rustaria::network::packet::ClientBoundPacket;
use rustaria::network::ClientNetwork;
use rustaria::player::{ClientBoundPlayerPacket, Player, ServerBoundPlayerPacket};
use rustaria::ty::WS;
use std::collections::VecDeque;
use glfw::{Action, Key, WindowEvent};
use tracing::info;
use crate::Camera;

const MAX_CORRECTION: f32 = 0.0005;

pub(crate) struct PlayerSystem {
    server_player: Player,
    base_server_player: Player,
    prediction_player: Player,

    send_speed: Vector2D<f32, WS>,


    w: bool,
    a: bool,
    s: bool,
    d: bool,
    zoom: f32,
    speed: Vector2D<f32, WS>,

    unprocessed_events: VecDeque<(u32, Vector2D<f32, WS>)>,
    tick: u32,
}

impl PlayerSystem {
    pub fn new() -> Result<Self> {
        Ok(Self {
            server_player: Player {
                pos: Default::default(),
                velocity: Default::default(),
            },
            base_server_player: Player {
                pos: Default::default(),
                velocity: Default::default(),
            },
            prediction_player: Player {
                pos: Default::default(),
                velocity: Default::default(),
            },
            send_speed: Default::default(),
            w: false,
            a: false,
            s: false,
            d: false,
            zoom: 10.0,
            speed: Default::default(),
            unprocessed_events: Default::default(),
            tick: 0,
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

    pub fn tick(&mut self, network: &mut ClientNetwork) -> Result<()> {
        self.send_speed = self.speed * (self.zoom / 30.0);
        self.tick += 1;

        // Send our speed at this tick
        network.send(ServerBoundPlayerPacket::SetDir(self.tick, self.send_speed))?;
        self.unprocessed_events
            .push_front((self.tick, self.send_speed));
        self.send_speed = Vector2D::zero();

        self.correct_offset();
        Ok(())
    }

    pub fn packet(&mut self, packet: ClientBoundPlayerPacket) {
        match packet {
            ClientBoundPlayerPacket::RespondPos(tick, pos) => {
                if let Some(pos) = pos {
                    self.server_player.pos = pos;
                }

                // Remove all events that the server has now applied.
                while let Some((value_id, speed)) = self.unprocessed_events.pop_back() {
                    // Move the base server entity forward.
                    // This totally ignores if the server sends a different speed, this is intentional.
                    // By this being on the predicted speed we can safely isolate the error amount by doing
                    // self.server_entity - self.base_server_entity, this lets us correct it in a sneaky timeframe.
                    self.base_server_player.velocity = speed;
                    self.base_server_player.tick(1.0);

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
    }

    pub fn get_camera(&mut self) -> Camera {
        Camera {
            pos: self.prediction_player.pos.to_array(),
            zoom: self.zoom
        }
    }

    // If the server says a different value try to correct it without freaking the player out.
    fn correct_offset(&mut self) {
        let server_offset = self.server_player.pos - self.base_server_player.pos;
        let distance = server_offset.length();

        // If the distance is too big just teleport the donut.
        if distance > 0.1 {
            self.base_server_player.pos = self.server_player.pos;
            self.prediction_player.pos = self.server_player.pos;
        } else if distance > 0.0 {
            // Slightly drift the donut.
            let mut amount = server_offset.clamp_length(0.0, MAX_CORRECTION);
            self.base_server_player.pos += amount;
            self.prediction_player.pos += amount;
        }
    }

    // When a client receives a packet, rebase the base_server_entity and
    // then apply the events not yet to be responded by the server.
    fn compile_prediction(&mut self) {
        // Put prediction on the server value
        self.prediction_player = self.base_server_player.clone();

        // If reconciliation is on, we apply values that the server has not yet processed.
        for (_, speed) in &self.unprocessed_events {
            self.prediction_player.velocity = *speed;
            self.prediction_player.tick(1.0);
        }

        self.prediction_player.velocity = self.speed;
    }
}
