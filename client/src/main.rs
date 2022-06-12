#![allow(clippy::new_without_default)]

extern crate core;

use std::collections::HashMap;
use std::ops::Index;

use euclid::{rect, vec2, Rect, Vector2D};
use eyre::Result;
use glfw::{Action, Key, WindowEvent};
use glium::Surface;
use tracing::{info, Level};
use tracing_subscriber::fmt::format;
use tracing_subscriber::util::SubscriberInitExt;

use rustaria::api::identifier::Identifier;
use rustaria::api::registry::Registry;
use rustaria::api::{Assets, Carrier, CarrierAccess};
use rustaria::chunk::tile::{Tile, TilePrototype};
use rustaria::chunk::{Chunk, ChunkLayer, CHUNK_SIZE};
use rustaria::entity::component::{CollisionComponent, HumanoidComponent, PhysicsComponent, PositionComponent};
use rustaria::entity::prototype::EntityPrototype;
use rustaria::entity::EntityWorld;
use rustaria::network::packet::{ClientBoundPacket, ServerBoundPacket};
use rustaria::network::{new_networking, ClientNetwork};
use rustaria::player::ServerBoundPlayerPacket;
use rustaria::ty::chunk_entry_pos::ChunkEntryPos;
use rustaria::ty::chunk_pos::ChunkPos;
use rustaria::ty::direction::DirMap;
use rustaria::{ty, Server};

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
    let dirt = client
        .carrier
        .tile
        .identifier_to_id(&Identifier::new("dirt"))
        .expect("where dirt");

    let air = client
        .carrier
        .tile
        .identifier_to_id(&Identifier::new("air"))
        .expect("where air");

    let dirt_chunk = Chunk {
        tile: ChunkLayer::new_copy(client.carrier.create(dirt)),
    };

    let air_chunk = Chunk {
        tile: ChunkLayer::new_copy(client.carrier.create(air)),
    };

    client.server.put_chunk(ChunkPos { x: 0, y: 0 }, air_chunk.clone());
    client.server.put_chunk(ChunkPos { x: 1, y: 0 }, dirt_chunk.clone());
    client.server.put_chunk(ChunkPos { x: 2, y: 0 }, air_chunk.clone());

    client.server.put_chunk(ChunkPos { x: 0, y: 1 }, air_chunk.clone());
    client.server.put_chunk(ChunkPos { x: 1, y: 1 }, air_chunk.clone());
    client.server.put_chunk(ChunkPos { x: 2, y: 1 }, air_chunk.clone());



    client.network.send(ServerBoundPacket::RequestChunk(ChunkPos { x: 0, y: 0 }))?;
    client.network.send(ServerBoundPacket::RequestChunk(ChunkPos { x: 1, y: 0 }))?;
    client.network.send(ServerBoundPacket::RequestChunk(ChunkPos { x: 2, y: 0 }))?;
    client.network.send(ServerBoundPacket::RequestChunk(ChunkPos { x: 0, y: 1 }))?;
    client.network.send(ServerBoundPacket::RequestChunk(ChunkPos { x: 1, y: 1 }))?;
    client.network.send(ServerBoundPacket::RequestChunk(ChunkPos { x: 2, y: 1 }))?;

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
                        jump_frames: 0.25,
                        jump_speed: 20.,
                        run_acceleration: 1.3,
                        run_slowdown: 0.8,
                        run_max_speed: 12.0,

                        // ignore this shit
                        dir: Default::default(),
                        jumping: false,
                        jumped: false,
                        jump_frames_remaining: 0.0
                    })
                },
            )]),
        };
        let (client, server) = new_networking();

        Ok(Self {
            renderer: WorldRenderer::new(&frontend, &carrier, &assets)?,
            frontend,
            server: Server::new(&carrier, server)?,
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
            if let event @ (WindowEvent::Key(Key::W | Key::A | Key::S | Key::D | Key::Space, _, _, _)
            | WindowEvent::Scroll(_, _)) = event
            {
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
                    self.player.packet(packet, &mut self.entity, &self.chunks)?;
                }
            }
        }
        self.player.tick(&mut self.network, &mut self.entity, &self.chunks)?;
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
