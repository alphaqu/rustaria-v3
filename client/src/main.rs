#![allow(clippy::new_without_default)]

extern crate core;

use std::collections::HashMap;
use std::ops::Index;

use euclid::{vec2, Vector2D};
use eyre::Result;
use glfw::{Action, Key, WindowEvent};
use glium::Surface;
use tracing::{info, Level};
use tracing_subscriber::fmt::format;
use tracing_subscriber::util::SubscriberInitExt;

use rustaria::{Server, ty};
use rustaria::api::{Assets, Carrier};
use rustaria::api::identifier::Identifier;
use rustaria::api::registry::Registry;
use rustaria::chunk::{Chunk, CHUNK_SIZE, ChunkLayer};
use rustaria::chunk::tile::{Tile, TilePrototype};
use rustaria::entity::component::{PositionComponent, VelocityComponent};
use rustaria::entity::EntityWorld;
use rustaria::entity::prototype::EntityPrototype;
use rustaria::network::{ClientNetwork, new_networking};
use rustaria::network::packet::{ClientBoundPacket, ServerBoundPacket};
use rustaria::player::ServerBoundPlayerPacket;
use rustaria::ty::{ChunkEntryPos, ChunkPos};

use crate::frontend::Frontend;
use crate::player::PlayerSystem;
use crate::renderer::{Camera, WorldRenderer};

mod frontend;
mod player;
mod renderer;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .event_format(format().compact())
        .without_time()
        .finish()
        .init();
    color_eyre::install()?;

    info!("Boozing froge");
    let mut client = Client::new()?;
    let id = client
        .carrier
        .tile
        .identifier_to_id(&Identifier::new("dirt"))
        .expect("where dirt");

    let mut chunk = Chunk {
        tile: ChunkLayer::new_copy(client.carrier.tile.create(id)),
    };

    client.server.put_chunk(ChunkPos::zero(), chunk);
    client
        .network
        .send(ServerBoundPacket::RequestChunk(ChunkPos::zero()))?;
    client.network.send(ServerBoundPlayerPacket::Join())?;
    while client.frontend.running() {
        client.tick_events()?;
        client.tick()?;
        client.draw()?;
    }

    Ok(())
}

pub struct Client {
    frontend: Frontend,
    renderer: WorldRenderer,
    assets: Assets,

    carrier: Carrier,
    server: Server,

    network: ClientNetwork,
    entity: EntityWorld,
    player: PlayerSystem,

    chunks: HashMap<ChunkPos, Chunk>,
}

impl Client {
    pub fn new() -> Result<Self> {
        let frontend = Frontend::new()?;
        let assets = Assets {};
        let carrier = Carrier {
            tile: Registry::new(vec![(
                Identifier::new("dirt"),
                TilePrototype {
                    image: Identifier::new("image/tile/dirt.png"),
                },
            )]),
            entity: Registry::new(vec![(
                Identifier::new("player"),
                EntityPrototype {
                    position: PositionComponent {
                        pos: Default::default()
                    },
                    velocity: Some(VelocityComponent {
                        velocity: Default::default()
                    })
                },
            )])
        };
        let (client, server) = new_networking();

        Ok(Self {
            renderer: WorldRenderer::new(&frontend, &carrier, &assets)?,
            frontend,
            server: Server::new( &carrier, server)?,
            network: client,
            entity: EntityWorld::new(&carrier)?,
            player: PlayerSystem::new(&carrier)?,
            chunks: Default::default(),
            carrier,
            assets,
        })
    }

    pub fn tick_events(&mut self) -> Result<()> {
        for event in self.frontend.poll_events() {
            if let event @ (WindowEvent::Key(Key::W | Key::A | Key::S | Key::D, _, _, _) | WindowEvent::Scroll(_, _)) = event {
                self.player.event(event)
            }
        }
        Ok(())
    }

    pub fn tick(&mut self) -> Result<()> {
        self.server.tick()?;
        for packet in self.network.poll() {
            match packet {
                ClientBoundPacket::Chunk(pos, chunk) => {
                    self.chunks.insert(pos, chunk);
                }
                ClientBoundPacket::Player(packet) => {
                    self.player.packet(packet, &mut self.entity)?;
                }
            }
        }
        self.player.tick(&mut self.network, &mut self.entity)?;
        self.renderer.tick(&self.chunks)?;
        Ok(())
    }

    pub fn draw(&mut self) -> Result<()> {
        let camera = self.player.get_camera();

        let mut frame = self.frontend.start_draw();
        frame.clear_color_srgb(0.15, 0.15, 0.15, 1.0);
        let result = self.renderer.draw(&self.frontend, camera, &mut frame);
        frame.finish()?;
        result
    }
}
