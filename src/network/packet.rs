use crate::ty::id::Id;
use crate::Chunk;
use crate::world::chunk::block::BlockPrototype;
use crate::world::chunk::layer::BlockLayerPrototype;
use crate::player::{ClientBoundPlayerPacket, ServerBoundPlayerPacket};
use crate::ty::chunk_pos::ChunkPos;
use crate::ty::block_pos::BlockPos;
use crate::world::{ClientBoundWorldPacket, ServerBoundWorldPacket};

#[macro_export]
macro_rules! packet {
    ($NAME:ident($SERVER:ident, $CLIENT:ident)) => {
        // Server
        impl From<$SERVER> for $crate::network::packet::ServerBoundPacket {
            fn from(value: $SERVER) -> Self {
                $crate::network::packet::ServerBoundPacket::$NAME(value)
            }
        }
        // Client
        impl From<$CLIENT> for $crate::network::packet::ClientBoundPacket {
            fn from(value: $CLIENT) -> Self {
                $crate::network::packet::ClientBoundPacket::$NAME(value)
            }
        }
    };
}

pub enum ServerBoundPacket {
    World(ServerBoundWorldPacket),
    Player(ServerBoundPlayerPacket),
}

pub enum ClientBoundPacket {
    World(ClientBoundWorldPacket),
    Player(ClientBoundPlayerPacket),
}
