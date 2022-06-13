#![allow(clippy::new_without_default)]

extern crate core;

use euclid::{rect, vec2};
use eyre::Result;
use glfw::{Key, WindowEvent};
use glium::Surface;
use tracing::{info, Level};
use tracing_subscriber::fmt::format;
use tracing_subscriber::util::SubscriberInitExt;
use rustaria::api::{Assets, Carrier, CarrierAccess};

use crate::frontend::Frontend;
use world::player::PlayerSystem;
use crate::render::debug::DebugRenderer;
use crate::render::Camera;
use crate::world::ClientWorld;
use rustaria::api::identifier::Identifier;
use rustaria::api::registry::Registry;
use rustaria::chunk::storage::ChunkStorage;
use rustaria::chunk::tile::TilePrototype;
use rustaria::chunk::{Chunk, ChunkLayer};
use rustaria::debug::DebugKind;
use rustaria::entity::component::{
    CollisionComponent, GravityComponent, HumanoidComponent, PhysicsComponent, PositionComponent,
};
use rustaria::entity::prototype::EntityPrototype;

mod frontend;
mod render;
mod world;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .event_format(format().compact())
        .without_time()
        .finish()
        .init();
    color_eyre::install()?;
    let mut runtime = Client::new()?;
    runtime.reload();
    runtime.run()?;
    Ok(())
}

pub struct Client {
    carrier: Carrier,
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

        Ok(Client {
            carrier: Carrier {
                tile: Registry::new(vec![]),
                entity: Registry::new(vec![]),
                assets: Assets {}
            },
            camera: Camera {
                pos: [0.0, 0.0],
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
            world.tick(&self.carrier, &mut self.debug)?
        }
        self.debug.finish()?;
        Ok(())
    }

    pub fn draw(&mut self) -> Result<()> {
        let mut frame = self.frontend.start_draw();
        frame.clear_color(0.01, 0.01, 0.01, 1.0);

        if let Some(world) = &mut self.world {
            if let Some(camera) = world.get_camera() {
                self.camera = camera;
            }

            world.draw(&self.frontend, &self.camera, &mut frame)?
        }
        self.debug.draw(&self.frontend, &self.camera, &mut frame)?;

        frame.finish()?;
        Ok(())
    }

    pub fn join_world(&self) -> Result<ClientWorld> {
        let dirt = self
            .carrier
            .tile
            .identifier_to_id(&Identifier::new("dirt"))
            .expect("where dirt");

        let air = self
            .carrier
            .tile
            .identifier_to_id(&Identifier::new("air"))
            .expect("where air");

        let air = self.carrier.create(air);
        let dirt = self.carrier.create(dirt);
        let mut out = Vec::new();
        for y in 0..9 {
            for x in 0..9 {
                if y > 0 {
                    out.push(Chunk {
                        tile: ChunkLayer::new_copy(air),
                    });
                } else {
                    out.push(Chunk {
                        tile: ChunkLayer::new_copy(dirt),
                    });
                }
            }
        }

        ClientWorld::new_integrated(
            &self.frontend,
            &self.carrier,
            ChunkStorage::new(9, 9, out).unwrap(),
        )
    }

    pub fn reload(&mut self) {
        info!("reloading");
        self.carrier = Carrier {
            tile: Registry::new(vec![
                (
                    Identifier::new("dirt"),
                    TilePrototype {
                        image: Some(Identifier::new("image/tile/dirt.png")),
                        collision: true,
                    },
                ),
                (
                    Identifier::new("air"),
                    TilePrototype {
                        image: None,
                        collision: false,
                    },
                ),
            ]),
            entity: Registry::new(vec![(
                Identifier::new("player"),
                EntityPrototype {
                    position: PositionComponent {
                        pos: vec2(24.0, 20.0),
                    },
                    velocity: Some(PhysicsComponent {
                        vel: Default::default(),
                        accel: Default::default(),
                    }),
                    collision: Some(CollisionComponent {
                        collision_box: rect(-1.0, -1.0, 2.0, 2.0),
                        collided: Default::default(),
                    }),
                    humanoid: Some(HumanoidComponent {
                        jump_amount: 15.0,
                        jump_speed: 20.0,
                        run_acceleration: 0.08,
                        run_slowdown: 0.2,
                        run_max_speed: 11.0,

                        // ignore this shit
                        dir: Default::default(),
                        jumping: false,
                        jumped: false,
                        jump_frames_remaining: 0.0,
                    }),
                    gravity: Some(GravityComponent { amount: 1.0 }),
                    image: Some(Identifier::new("image/entity/glisco.png")),
                    panel: rect(-1.0, -1.0, 2.0, 2.0),
                },
            )]),
            assets: Assets {}
        };
    }
}
