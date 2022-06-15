use crate::ty::id::Id;
use crate::Chunk;
use crate::chunk::block::{BlockPrototype, BlockLayerPrototype};
use crate::player::{ClientBoundPlayerPacket, ServerBoundPlayerPacket};
use crate::ty::chunk_pos::ChunkPos;
use crate::ty::block_pos::BlockPos;

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
    RequestChunk(ChunkPos),
    SetBlock(BlockPos, Id<BlockLayerPrototype>, Id<BlockPrototype>),
    Player(ServerBoundPlayerPacket),
}

pub enum ClientBoundPacket {
    Chunk(ChunkPos, Chunk),
    Player(ClientBoundPlayerPacket),
}
