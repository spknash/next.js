use anyhow::Result;
use roaring::RoaringBitmap;
use turbo_tasks::Vc;

use crate::module_graph::chunk_group_info::{
    ChunkGroupInfo, RoaringBitmapWrapper, RoaringBitmapWrapperCell,
};

/// Allows to gather information about which assets are already available.
#[turbo_tasks::value]
pub struct AvailableChunkGroups {
    chunk_groups: RoaringBitmapWrapper,
}

#[turbo_tasks::value_impl]
impl AvailableChunkGroups {
    #[turbo_tasks::function]
    pub async fn new(chunk_group: u32) -> Result<Vc<Self>> {
        Ok(AvailableChunkGroups {
            chunk_groups: RoaringBitmapWrapper::new(
                RoaringBitmap::from_sorted_iter(std::iter::once(chunk_group)).unwrap(),
            ),
        }
        .cell())
    }

    #[turbo_tasks::function]
    pub async fn with_chunk_group(&self, chunk_group: u32) -> Result<Vc<Self>> {
        let mut chunk_groups = self.chunk_groups.clone();
        chunk_groups.insert(chunk_group);
        Ok(AvailableChunkGroups { chunk_groups }.cell())
    }

    #[turbo_tasks::function]
    pub async fn hash(&self, chunk_group_info: Vc<ChunkGroupInfo>) -> Result<Vc<u64>> {
        Ok(Vc::cell(
            chunk_group_info
                .await?
                .hash_chunk_groups(&self.chunk_groups),
        ))
    }

    #[turbo_tasks::function]
    pub async fn is_available(&self, module: Vc<RoaringBitmapWrapperCell>) -> Result<Vc<bool>> {
        Ok(Vc::cell(self.is_available_individual(&*module.await?)))
    }
}

impl AvailableChunkGroups {
    pub fn is_available_individual(&self, module_chunk_groups: &RoaringBitmapWrapper) -> bool {
        // `self.chunk_groups` is the union of all parent chunk groups (i.e. a single chunking path
        // leading to this module)
        //
        // `module_chunk_groups` is the union of all chunk groups of the module (i.e. the union of
        // all paths leading to this module)
        //
        // The module is available, if there is at least one parent chunk group (bit is set in
        // `self.chunk_groups`) that contains the module (bit is set in `module`)
        !self.chunk_groups.is_disjoint(module_chunk_groups)
    }
}
