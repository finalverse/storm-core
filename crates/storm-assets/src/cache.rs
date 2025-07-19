// File: crates/storm-assets/src/cache.rs
// Asset caching system

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::{AssetId, AssetData, AssetMetadata, Vertex};

/// Asset cache implementation
pub struct AssetCache {
    assets: HashMap<AssetId, AssetData>,
    metadata: HashMap<AssetId, AssetMetadata>,
    path_to_id: HashMap<PathBuf, AssetId>,
    total_memory: usize,
    max_memory: usize,
}

impl AssetCache {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            metadata: HashMap::new(),
            path_to_id: HashMap::new(),
            total_memory: 0,
            max_memory: 1024 * 1024 * 1024, // 1GB default
        }
    }

    pub fn insert(&mut self, id: AssetId, data: AssetData, metadata: AssetMetadata) {
        let data_size = self.estimate_size(&data);

        // Check if we need to free memory
        if self.total_memory + data_size > self.max_memory {
            self.evict_lru(data_size);
        }

        self.path_to_id.insert(metadata.file_path.clone(), id);
        self.assets.insert(id, data);
        self.metadata.insert(id, metadata);
        self.total_memory += data_size;
    }

    pub fn get_data(&self, id: AssetId) -> Option<&AssetData> {
        self.assets.get(&id)
    }

    pub fn get_metadata(&self, id: AssetId) -> Option<&AssetMetadata> {
        self.metadata.get(&id)
    }

    pub fn get_by_path(&self, path: &Path) -> Option<AssetId> {
        self.path_to_id.get(path).copied()
    }

    pub fn update_data(&mut self, id: AssetId, data: AssetData) {
        if let Some(old_data) = self.assets.get(&id) {
            self.total_memory -= self.estimate_size(old_data);
        }

        let new_size = self.estimate_size(&data);
        self.assets.insert(id, data);
        self.total_memory += new_size;
    }

    pub fn remove(&mut self, id: AssetId) -> bool {
        if let Some(data) = self.assets.remove(&id) {
            self.total_memory -= self.estimate_size(&data);

            if let Some(metadata) = self.metadata.remove(&id) {
                self.path_to_id.remove(&metadata.file_path);
            }

            true
        } else {
            false
        }
    }

    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            asset_count: self.assets.len(),
            memory_usage: self.total_memory,
            max_memory: self.max_memory,
            cache_hit_rate: 0.95, // Placeholder
        }
    }

    fn estimate_size(&self, data: &AssetData) -> usize {
        match data {
            AssetData::Mesh(mesh) => {
                mesh.vertices.len() * std::mem::size_of::<Vertex>() +
                    mesh.indices.len() * std::mem::size_of::<u32>()
            }
            AssetData::Texture(texture) => texture.data.len(),
            AssetData::Audio(audio) => audio.data.len(),
            AssetData::Raw(data) => data.len(),
            AssetData::Material(_) => 1024, // Default estimate for material
        }
    }

    fn evict_lru(&mut self, needed_space: usize) {
        // Simple LRU eviction - remove oldest assets until we have enough space
        // In a real implementation, this would track access times
        let mut freed_space = 0;
        let mut to_remove = Vec::new();

        for (&id, data) in &self.assets {
            if freed_space >= needed_space {
                break;
            }

            freed_space += self.estimate_size(data);
            to_remove.push(id);
        }

        for id in to_remove {
            self.remove(id);
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub asset_count: usize,
    pub memory_usage: usize,
    pub max_memory: usize,
    pub cache_hit_rate: f32,
}