#![allow(clippy::new_without_default)]

extern crate core;

use euclid::Vector2D;
use eyre::Result;
use glfw::{Key, WindowEvent};
use glium::Surface;
use mlua::Lua;
use rustaria::api::{Api, Resources};
use tracing::info;
use tracing_error::ErrorLayer;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::frontend::Frontend;
use crate::render::debug::DebugRenderer;
use crate::render::Camera;
use crate::world::ClientWorld;
use rustaria::api::identifier::Identifier;
use rustaria::api::registry::Registry;
use rustaria::chunk::storage::ChunkStorage;
use rustaria::chunk::{Chunk, ChunkLayer};
use rustaria::debug::DebugKind;
use world::player::PlayerSystem;

mod frontend;
mod render;
mod world;

fn main() -> Result<()> {
    let fmt_layer = fmt::layer()
        //.with_max_level(Level::TRACE)
        .with_span_events(FmtSpan::ACTIVE)
        .event_format(format().compact())
        .without_time();
    tracing_subscriber::registry()
        .with(ErrorLayer::default())
        .with(fmt_layer)
        .init();

    color_eyre::install()?;
    let mut runtime = Client::new()?;
    runtime.reload();
    runtime.run()?;
    Ok(())
}

pub struct Client {
    api: Api,
    camera: Camera,
    debug: DebugRenderer,
    frontend: Frontend,

    world: Option<ClientWorld>,
}

impl Client {
    pub fn new() -> Result<Client> {
        let frontend = Frontend::new()?;
        let mut debug = DebugRenderer::new(&frontend)?;
        debug.enable(DebugKind::EntityVelocity);
        debug.enable(DebugKind::EntityCollision);
        debug.enable(DebugKind::ChunkBorders);

        Ok(Client {
            api: Api::new(),
            camera: Camera {
                pos: Vector2D::zero(),
                zoom: 10.0,
            },
            debug,
            frontend,
            world: None,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        while self.frontend.running() {
            self.tick_events()?;
            self.tick()?;
            self.draw()?;
        }

        Ok(())
    }

    pub fn tick_events(&mut self) -> Result<()> {
        for event in self.frontend.poll_events() {
            if let WindowEvent::Key(Key::O, _, _, _) = event {
                self.world = Some(self.join_world()?);
            }
            if let Some(world) = &mut self.world {
                world.event(&self.frontend, event);
            }
        }
        Ok(())
    }

    pub fn tick(&mut self) -> Result<()> {
        if let Some(world) = &mut self.world {
            world.tick(&self.api, &mut self.debug)?
        }
        self.debug.finish()?;
        Ok(())
    }

    pub fn draw(&mut self) -> Result<()> {
        let mut frame = self.frontend.start_draw();
        frame.clear_color(0.01, 0.01, 0.01, 1.0);

        if let Some(world) = &mut self.world {
            if let Some(camera) = world.get_camera() {
                self.camera.pos -= (self.camera.pos - camera.pos) * 0.05;
                self.camera.zoom = camera.zoom;
            }

            world.draw(&self.frontend, &self.camera, &mut frame)?
        }
        self.debug.draw(&self.frontend, &self.camera, &mut frame)?;

        frame.finish()?;
        Ok(())
    }

    pub fn join_world(&self) -> Result<ClientWorld> {
        let mut out = Vec::new();
        for y in 0..9 {
            for x in 0..9 {
                out.push(Chunk {
                    layers: self.api.carrier.chunk_layers.map(|id, prototype| {
                        let dirt = prototype.registry.create(
                            prototype
                                .registry
                                .identifier_to_id(&Identifier::new("dirt"))
                                .expect("where dirt"),
                        );
                        let air = prototype.registry.create(
                            prototype
                                .registry
                                .identifier_to_id(&Identifier::new("air"))
                                .expect("where air"),
                        );

                        if x == 2 && y == 1 {
                            let a = air;
                            let d = dirt;

                            ChunkLayer {
                                data: [
                                    [a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a],
                                    [a, d, d, d, a, d, a, a, a, a, a, a, a, a, a, a],
                                    [a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a],
                                    [a, d, d, d, a, d, a, a, a, a, a, a, a, a, a, a],
                                    [a, d, d, d, a, d, a, a, a, a, a, a, a, a, a, a],
                                    [a, d, d, d, a, d, a, a, a, a, a, a, a, a, a, a],
                                    [a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a],
                                    [a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a],
                                    [a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a],
                                    [a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a],
                                    [a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a],
                                    [a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a],
                                    [a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a],
                                    [a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a],
                                    [a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a],
                                    [a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a],
                                ],
                            }
                        } else {
                            if x == 0 || (y > 0 && x != 2) || x > 3 {
                                ChunkLayer::new_copy(air)
                            } else {
                                ChunkLayer::new_copy(dirt)
                            }
                        }
                    }),
                });
            }
        }

        ClientWorld::new_integrated(
            &self.frontend,
            &self.api,
            ChunkStorage::new(9, 9, out).unwrap(),
        )
    }

    pub fn reload(&mut self) {
        info!("reloading");
        self.api.reload();

        //  self.carrier = Registries {
        //             tile: Registry::new(vec![
        //                 (
        //                     Identifier::new("dirt"),
        //                     TilePrototype {
        //                         image: Some(Identifier::new("image/tile/dirt.png")),
        //                         collision: true,
        //                         connection_type: ConnectionType::Connected
        //                     },
        //                 ),
        //                 (
        //                     Identifier::new("air"),
        //                     TilePrototype {
        //                         image: None,
        //                         collision: false,
        //                         connection_type: ConnectionType::Isolated
        //                     },
        //                 ),
        //             ]),
        //             entity: Registry::new(vec![(
        //                 Identifier::new("player"),
        //                 EntityPrototype {
        //                     position: PositionComponent {
        //                         pos: vec2(24.0, 20.0),
        //                     },
        //                     velocity: Some(PhysicsComponent {
        //                         vel: Default::default(),
        //                         accel: Default::default(),
        //                     }),
        //                     collision: Some(CollisionComponent {
        //                         collision_box: rect(-1.0, -1.0, 2.0, 2.0),
        //                         collided: Default::default(),
        //                     }),
        //                     humanoid: Some(HumanoidComponent {
        //                         jump_amount: 15.0,
        //                         jump_speed: 20.0,
        //                         run_acceleration: 0.08,
        //                         run_slowdown: 0.2,
        //                         run_max_speed: 11.0,
        //
        //                         // ignore this shit
        //                         dir: Default::default(),
        //                         jumping: false,
        //                         jumped: false,
        //                         jump_frames_remaining: 0.0,
        //                     }),
        //                     gravity: Some(GravityComponent { amount: 1.0 }),
        //                     image: Some(Identifier::new("image/entity/glisco.png")),
        //                     panel: rect(-1.0, -1.0, 2.0, 2.0),
        //                 },
        //             )]),
        //             resources: Resources {}
        //         };
    }
}
