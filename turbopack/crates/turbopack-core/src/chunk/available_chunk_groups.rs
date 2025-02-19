use std::hash::{Hash, Hasher};

use anyhow::Result;
use rustc_hash::FxHasher;
use turbo_tasks::Vc;

use crate::module_graph::chunk_group_info::{RoaringBitmapWrapper, RoaringBitmapWrapperCell};

/// Allows to gather information about which assets are already available.
#[turbo_tasks::value]
pub struct AvailableChunkGroups {
    chunk_groups: RoaringBitmapWrapper,
}

#[turbo_tasks::value_impl]
impl AvailableChunkGroups {
    #[turbo_tasks::function]
    pub async fn new(chunk_groups: Vc<RoaringBitmapWrapperCell>) -> Result<Vc<Self>> {
        Ok(AvailableChunkGroups {
            chunk_groups: chunk_groups.owned().await?,
        }
        .cell())
    }

    #[turbo_tasks::function]
    pub async fn with_chunk_group(
        &self,
        chunk_group: Vc<RoaringBitmapWrapperCell>,
    ) -> Result<Vc<Self>> {
        Ok(AvailableChunkGroups {
            chunk_groups: RoaringBitmapWrapper::new(&*self.chunk_groups | &**chunk_group.await?),
        }
        .cell())
    }

    #[turbo_tasks::function]
    pub async fn hash(&self) -> Result<Vc<u64>> {
        let mut hasher = FxHasher::default();
        self.chunk_groups.hash(&mut hasher);
        // TODO a more deterministic hash? Previously, this hashed all `module.ident().to_string()`
        Ok(Vc::cell(hasher.finish()))
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
