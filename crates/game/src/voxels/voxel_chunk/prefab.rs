use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    prelude::*,
};
use chunk_serde::{BinSerializer, CompressedChunkData};

use crate::voxels::{Blocks, cellular_automata::CellData};

#[derive(Default)]
pub struct ChunkPrefabLoader;

impl AssetLoader for ChunkPrefabLoader {
    type Asset = ChunkPrefab;
    type Settings = ();
    type Error = &'static str;
    fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext,
    ) -> impl bevy::tasks::ConditionalSendFuture<Output = std::result::Result<Self::Asset, Self::Error>>
    {
        async move {
            let mut data = String::new();
            reader
                .read_to_string(&mut data)
                .await
                .or(Err("Failed to read to str"))?;
            from_str(data).await
        }
    }
    fn extensions(&self) -> &[&str] {
        &["phoxel"]
    }
}
#[derive(Asset, TypePath)]
pub struct ChunkPrefab {
    chunk: CompressedChunkData<Blocks>,
    automita: CompressedChunkData<CellData>,
}

async fn from_str(data: String) -> Result<ChunkPrefab, &'static str> {
    todo!()
}

async fn from_bytes(data: Vec<u8>) -> Result<ChunkPrefab, &'static str> {
    todo!()
}

// impl chunk_serde::Serialize for ChunkPrefab {
//     fn insert(&self, vec: &mut BinSerializer) -> Result<usize> {
//         vec.extend_from_slice(b"Phoxb");
//         let mut used = self.chunk.insert(vec);
//         used += self.automita.insert(vec);
//         Ok(used + 5)
//     }

//     fn extract(slice: &[u8]) -> Result<(Self, usize)> {}
// }
