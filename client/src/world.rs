use rustaria::api::{Api, Resources};
use rustaria::chunk::storage::ChunkStorage;
use rustaria::entity::EntityWorld;
use rustaria::network::{ClientNetwork, new_networking};
use rustaria::Server;
use eyre::{Result, WrapErr};
use glfw::WindowEvent;
use glium::Frame;
use crate::render::world::WorldRenderer;
use rustaria::network::packet::ClientBoundPacket;
use rustaria::player::ServerBoundPlayerPacket;
use crate::{Camera, DebugRenderer, Frontend, PlayerSystem};

pub mod player;

/// This exists when a client has joined a world.
pub struct ClientWorld {
	integrated: Option<Server>,

	network: ClientNetwork,
	player: PlayerSystem,
	entity: EntityWorld,
	chunks: ChunkStorage,

	renderer: WorldRenderer
}

impl ClientWorld {
	pub fn new_integrated(frontend: &Frontend, api: &Api, chunks: ChunkStorage) -> Result<ClientWorld> {
		let (network, server_network) = new_networking();
		// Send join packet
		network.send(ServerBoundPlayerPacket::Join())?;

		Ok(ClientWorld {
			integrated: Some(Server::new(api, server_network, chunks.clone()).wrap_err("Failed to start server")?),
			network,
			player: PlayerSystem::new(api)?,
			entity: EntityWorld::new(api)?,
			chunks,
			renderer: WorldRenderer::new(frontend, api)?
		})
	}

	pub fn event(&mut self, frontend: &Frontend, event: WindowEvent) {
		self.player.event(event, frontend);
	}

	pub fn get_camera(&mut self) -> Option<Camera> {
		Some(self.player.get_camera())
	}

	pub fn tick(&mut self, api: &Api, debug: &mut DebugRenderer) -> Result<()> {
		if let Some(server) = &mut self.integrated {
			server.tick(api)?;
		}
		for packet in self.network.poll() {
			match packet {
				ClientBoundPacket::Chunk(pos, chunk) => {
					self.chunks.insert(pos, chunk);
					self.renderer.dirty_world();
				}
				ClientBoundPacket::Player(packet) => {
					self.player.packet(api, packet, &mut self.entity, &self.chunks)?;
				}
			}
		}
		self.renderer.dirty_world();
		self.player.tick(api, &mut self.network, &mut self.entity, &mut self.chunks)?;
		self.entity.tick(api, &self.chunks, debug);
		self.renderer.tick(&self.entity.storage, &self.player, &self.chunks, debug)?;
		Ok(())
	}

	pub fn draw(&mut self, frontend: &Frontend, camera: &Camera, frame: &mut Frame) -> Result<()> {
		self.renderer.draw(frontend, camera, frame)?;
		Ok(())
	}
}
