#![feature(drain_filter)]
#![allow(clippy::new_without_default)]

extern crate core;

use std::path::PathBuf;
use std::time::{Duration, Instant};
use euclid::Vector2D;
use eyre::{Context, Result};
use glfw::{Key, WindowEvent};
use glium::Surface;
use rustaria::api::Api;
use tracing::info;
use tracing_error::ErrorLayer;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::frontend::Frontend;
use debug::Debug;
use crate::render::Camera;
use crate::world::ClientWorld;
use rustaria::ty::identifier::Identifier;
use rustaria::chunk::storage::ChunkStorage;
use rustaria::chunk::{Chunk, ChunkLayer};
use rustaria::debug::DebugCategory;
use rustaria::TPS;
use rustaria::world::World;
use world::player::PlayerSystem;

mod frontend;
mod render;
mod world;
pub mod debug;

const TICK_DURATION: Duration = Duration::from_nanos((1000000000 / TPS) as u64);

fn main() -> Result<()> {
    let fmt_layer = fmt::layer()
        //.with_max_level(Level::TRACE)
        .event_format(format().compact())
        .without_time();
    tracing_subscriber::registry()
        .with(ErrorLayer::default())
        .with(fmt_layer)
        .init();

    color_eyre::install()?;
    let mut runtime = Client::new()?;
    runtime.reload()?;
    runtime.run()?;
    Ok(())
}

pub struct Client {
    api: Api,
    camera: Camera,
    debug: Debug,
    frontend: Frontend,

    world: Option<ClientWorld>,
}

impl Client {
    pub fn new() -> Result<Client> {
        let run_dir = std::env::current_dir().wrap_err("Could not find current directory.")?;
        let frontend = Frontend::new().wrap_err("Could not initialize frontend.")?;
        let mut debug = Debug::new(&frontend).wrap_err("Could not initialize debug render.")?;
        debug.enable(DebugCategory::TileSpread);
        //debug.enable(DebugCategory::EntityCollision);
        //debug.enable(DebugCategory::ChunkMeshing);
        //debug.enable(DebugCategory::ChunkBorders);
//
        Ok(Client {
            api: Api::new(run_dir, vec![PathBuf::from("../plugin")])?,
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
        let mut last_tick = Instant::now();
        while self.frontend.running() {
            self.tick_events()?;

            while let Some(value) = Instant::now().checked_duration_since(last_tick) {
                if value >= TICK_DURATION {
                    self.tick()?;
                    last_tick += TICK_DURATION;
                } else {
                    break;
                }
            }
            self.draw()?;
            self.debug.tick();

        }

        Ok(())
    }

    pub fn tick_events(&mut self) -> Result<()> {
        let start = Instant::now();
        for event in self.frontend.poll_events() {
            if let WindowEvent::Key(Key::O, _, _, _) = event {
                self.reload()?;
                self.world = Some(self.join_world()?);
            }
            if let Some(world) = &mut self.world {
                world.event(&self.frontend, event);
            }
        }
        self.debug.log_event(start);
        Ok(())
    }

    pub fn tick(&mut self) -> Result<()> {
        let start = Instant::now();
        if let Some(world) = &mut self.world {
            world.tick(&self.api, &mut self.debug)?
        }
        self.debug.log_tick(start);
        Ok(())
    }

    pub fn draw(&mut self) -> Result<()> {
        let start = Instant::now();
        let mut frame = self.frontend.start_draw();
        frame.clear_color(0.01, 0.01, 0.01, 1.0);

        if let Some(world) = &mut self.world {
            if let Some(camera) = world.get_camera() {
                self.camera.pos -= (self.camera.pos - camera.pos) * 0.05;
                self.camera.zoom = camera.zoom;
            }

            world.draw(&self.frontend, &self.camera, &mut frame)?
        }
        self.debug.log_draw(start);
        self.debug.draw(&self.frontend, &self.camera, &mut frame)?;
        frame.finish()?;
        Ok(())
    }

    pub fn join_world(&self) -> Result<ClientWorld> {
        let mut out = Vec::new();
        for y in 0..9 {
            for x in 0..9 {
                out.push(Chunk {
                    layers: self.api.carrier.block_layer.map(|id, prototype| {
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
            World::new(&self.api, ChunkStorage::new(9, 9, out).unwrap()).unwrap(),
        )
    }

    pub fn reload(&mut self) -> Result<()>{
        info!("reloading");
        self.api.reload()
    }
}
