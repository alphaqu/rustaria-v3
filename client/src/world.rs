use rustaria::api::Carrier;
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
	pub fn new_integrated(frontend: &Frontend, carrier: &Carrier, chunks: ChunkStorage) -> Result<ClientWorld> {
		let (network, server_network) = new_networking();
		// Send join packet
		network.send(ServerBoundPlayerPacket::Join())?;

		Ok(ClientWorld {
			integrated: Some(Server::new(carrier, server_network, chunks.clone()).wrap_err("Failed to start server")?),
			network,
			player: PlayerSystem::new(carrier)?,
			entity: EntityWorld::new(carrier)?,
			chunks,
			renderer: WorldRenderer::new(frontend, carrier)?
		})
	}

	pub fn event(&mut self, frontend: &Frontend, event: WindowEvent) {
		self.player.event(event, frontend);
	}

	pub fn get_camera(&mut self) -> Option<Camera> {
		Some(self.player.get_camera())
	}

	pub fn tick(&mut self, carrier: &Carrier, debug: &mut DebugRenderer) -> Result<()> {
		if let Some(server) = &mut self.integrated {
			server.tick(carrier)?;
		}
		for packet in self.network.poll() {
			match packet {
				ClientBoundPacket::Chunk(pos, chunk) => {
					self.chunks.insert(pos, chunk);
					self.renderer.dirty_world();
				}
				ClientBoundPacket::Player(packet) => {
					self.player.packet(packet, &mut self.entity, &self.chunks)?;
				}
			}
		}
		self.renderer.dirty_world();
		self.player.tick(carrier, &mut self.network, &mut self.entity, &mut self.chunks)?;
		self.entity.tick(&self.chunks, debug);
		self.renderer.tick(&self.entity.storage, &self.player, &self.chunks, debug)?;
		Ok(())
	}

	pub fn draw(&mut self, frontend: &Frontend, camera: &Camera, frame: &mut Frame) -> Result<()> {
		self.renderer.draw(frontend, camera, frame)?;
		Ok(())
	}
}
