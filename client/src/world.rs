use rustaria::api::{Api, Resources};
use rustaria::chunk::storage::ChunkStorage;
use rustaria::entity::EntityWorld;
use rustaria::network::{ClientNetwork, new_networking};
use rustaria::Server;
use eyre::{Result, WrapErr};
use glfw::WindowEvent;
use glium::Frame;
use rustaria::chunk::{BlockLayer, Chunk};
use crate::render::world::WorldRenderer;
use rustaria::network::packet::ClientBoundPacket;
use rustaria::player::ServerBoundPlayerPacket;
use rustaria::world::{ClientBoundWorldPacket, World};
use crate::{Camera, ClientApi, Debug, Frontend, PlayerSystem};

pub mod player;

/// This exists when a client has joined a world.
pub struct ClientWorld {
	integrated: Option<Server>,

	network: ClientNetwork,
	player: PlayerSystem,
	world: World,

	renderer: WorldRenderer
}

impl ClientWorld {
	pub fn new_integrated(frontend: &Frontend, api: &ClientApi, world: World) -> Result<ClientWorld> {
		let (network, server_network) = new_networking();
		// Send join packet
		network.send(ServerBoundPlayerPacket::Join())?;

		let storage = world.chunk.clone();
		Ok(ClientWorld {
			integrated: Some(Server::new(api, server_network, world).wrap_err("Failed to start server")?),
			network,
			player: PlayerSystem::new(api)?,
			world: World::new(api, storage)?,
			renderer: WorldRenderer::new(frontend, api)?
		})
	}

	pub fn event(&mut self, frontend: &Frontend, event: WindowEvent) {
		self.player.event(event, frontend);
	}

	pub fn get_camera(&mut self) -> Option<Camera> {
		Some(self.player.get_camera())
	}

	pub fn tick(&mut self, api: &ClientApi, debug: &mut Debug) -> Result<()> {
		if let Some(server) = &mut self.integrated {
			server.tick(api)?;
		}
		for packet in self.network.poll() {
			match packet {
				ClientBoundPacket::Player(packet) => {
					self.player.packet(api, packet, &mut self.world)?;
				}
				ClientBoundPacket::World(packet) => {
					match packet {
						ClientBoundWorldPacket::Chunk(pos, chunk) => {
							self.world.chunk.insert(pos, chunk);
						}
						ClientBoundWorldPacket::SetBlock(pos, layer_id, block_id) => {
							self.world.place_block(api, pos, layer_id, block_id);
						}
					}
				}
			}
		}
		self.player.tick(api, &mut self.network, &mut self.world)?;
		self.world.tick(api, debug);
		self.renderer.tick(&self.player, &self.world, debug)?;
		self.world.chunk.reset_dirty();
		Ok(())
	}

	pub fn draw(&mut self, frontend: &Frontend, camera: &Camera, frame: &mut Frame) -> Result<()> {
		self.renderer.draw(frontend, camera, frame)?;
		Ok(())
	}
}
