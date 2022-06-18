#![feature(drain_filter)]
#![allow(clippy::new_without_default)]

extern crate core;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use euclid::vec2;
use eyre::{Context, Result};
use glfw::{Key, WindowEvent};
use glium::Surface;
use tracing_error::ErrorLayer;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use debug::Debug;
use render::ty::viewport::Viewport;
use rustaria::debug::DebugCategory;
use rustaria::TPS;
use rustaria::ty::chunk_pos::ChunkPos;
use rustaria::ty::identifier::Identifier;
use rustaria::world::chunk::{Chunk, ChunkLayer};
use rustaria::world::chunk::storage::ChunkStorage;
use rustaria::world::World;

use crate::api::ClientApi;
use crate::frontend::Frontend;
use crate::render::world::chunk::block::BlockRendererPrototype;
use crate::render::world::chunk::layer::BlockLayerRendererPrototype;
use crate::ty::Timing;
use crate::game::ClientGame;
use crate::game::player::PlayerSystem;

mod frontend;
mod render;
mod game;
pub mod debug;
pub mod api;
mod ty;

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
    let mut client = Client::new()?;
    client.api.reload(&client.frontend)?;
    client.run()?;
    Ok(())
}

pub struct Client {
    viewport: Viewport,
    debug: Debug,
    game: Option<ClientGame>,
    api: ClientApi,
    frontend: Frontend,
}

impl Client {
    pub fn new() -> Result<Client> {
        let run_dir = std::env::current_dir().wrap_err("Could not find current directory.")?;
        let frontend = Frontend::new().wrap_err("Could not initialize frontend.")?;
        let mut debug = Debug::new(&frontend).wrap_err("Could not initialize debug render.")?;
        debug.enable(DebugCategory::TileSpread);
        debug.enable(DebugCategory::EntityCollision);
        //debug.enable(DebugCategory::ChunkMeshing);
        //debug.enable(DebugCategory::ChunkBorders);
//
        Ok(Client {
            api: ClientApi::new(run_dir, vec![PathBuf::from("../plugin")])?,
            viewport: Viewport::new(vec2(0.0, 0.0), 1.0),
            debug,
            frontend,
            game: None,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut timing = Timing::new();
        while self.frontend.running() {
            self.tick_events()?;

            while timing.next_tick() {
                self.tick()?;
            }

            self.draw(&timing)?;
            self.debug.tick();
        }

        Ok(())
    }

    pub fn tick_events(&mut self) -> Result<()> {
        let start = Instant::now();
        for event in self.frontend.poll_events() {
            if let WindowEvent::Key(Key::O, _, _, _) = event {
                self.game = Some(self.join_world()?);
            }
            if let WindowEvent::Key(Key::R, _, _, _) = event {
                self.api.reload(&self.frontend)?;
                if let Some(game) = &mut self.game {
                    game.renderer.reset();
                }
            }
            if let Some(world) = &mut self.game {
                world.event(&self.frontend, event);
            }
        }
        self.debug.log_event(start);
        Ok(())
    }

    pub fn tick(&mut self) -> Result<()> {
        let start = Instant::now();
        if let Some(world) = &mut self.game {
            world.tick(&self.frontend, &self.api, &self.viewport, &mut self.debug)?
        }
        self.debug.log_tick(start);
        Ok(())
    }

    pub fn draw(&mut self, timing: &Timing) -> Result<()> {
        let start = Instant::now();
        let mut frame = self.frontend.start_draw();
        frame.clear_color(0.10, 0.10, 0.10, 1.0);

        if let Some(world) = &mut self.game {
            if let Some(viewport) = world.get_viewport() {
                self.viewport.pos -= ((self.viewport.pos - viewport.pos) * 0.2) * timing.step();
                //self.viewport.pos = viewport.pos;
                self.viewport.zoom = viewport.zoom;
                self.viewport.recompute_rect(Some(&self.frontend));
            }


            world.draw(&self.api, &self.frontend, &mut frame, &self.viewport, &mut self.debug, timing)?;
        }
        self.debug.log_draw(start);
        self.debug.draw(&self.frontend, &self.viewport, &mut frame)?;
        frame.finish()?;
        Ok(())
    }

    pub fn join_world(&self) -> Result<ClientGame> {
        let mut storage = ChunkStorage::new(9, 9);

        for y in 0..9 {
            for x in 0..9 {
                storage.insert(ChunkPos {
                    x: x as u32,
                    y: y as u32,
                }, Chunk {
                    layers: self.api.carrier.block_layer.table.iter().map(|(id, prototype)| {
                        let dirt = prototype.blocks.create(
                            prototype
                                .blocks
                                .get_id(&Identifier::new("dirt"))
                                .expect("where dirt"),
                        );
                        let air = prototype.blocks.create(
                            prototype
                                .blocks
                                .get_id(&Identifier::new("air"))
                                .expect("where air"),
                        );

                        (id, if x == 2 && y == 1 {
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
                        } else if x == 0 || (y > 0 && x != 2) || x > 3 {
                            ChunkLayer::new_copy(air)
                        } else {
                            ChunkLayer::new_copy(dirt)
                        })
                    }).collect(),
                });
            }
        }
        ClientGame::new_integrated(
            &self.frontend,
            &self.api,
            World::new(&self.api, storage).unwrap(),
        )
    }
}
