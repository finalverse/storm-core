// File: crates/storm-assets/src/lib.rs
// Asset management system for StormCore
// Handles loading, caching, and processing of 3D assets

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};
use anyhow::Result;

pub mod loaders;
pub mod cache;
pub mod processors;

pub use loaders::*;
pub use cache::*;
pub use processors::*;

/// Asset identifier
pub type AssetId = uuid::Uuid;

/// Asset manager - central hub for all asset operations
pub struct AssetManager {
    cache: Arc<RwLock<AssetCache>>,
    loaders: HashMap<String, Arc<dyn AssetLoader>>,  // Use Arc instead of Box
    processors: HashMap<String, Arc<dyn AssetProcessor>>,  // Use Arc instead of Box
    base_path: PathBuf,
}

/// Asset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub id: AssetId,
    pub name: String,
    pub asset_type: AssetType,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub last_modified: u64,
    pub dependencies: Vec<AssetId>,
    pub tags: Vec<String>,
}

/// Types of assets supported
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    Mesh,
    Texture,
    Material,
    Animation,
    Audio,
    Script,
    Scene,
    Shader,
    Raw,
}

/// Generic asset data
#[derive(Debug, Clone)]
pub enum AssetData {
    Mesh(MeshData),
    Texture(TextureData),
    Material(MaterialData),
    Audio(AudioData),
    Raw(Vec<u8>),
}

/// Mesh asset data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub materials: Vec<AssetId>,
    pub bounding_box: BoundingBox,
}

/// Vertex data
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 4],
}

/// Bounding box
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// Texture asset data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub data: Vec<u8>,
    pub mip_levels: u32,
}

/// Texture formats
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureFormat {
    RGBA8,
    RGB8,
    R8,
    DXT1,
    DXT5,
    BC7,
}

/// Material asset data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialData {
    pub name: String,
    pub albedo_texture: Option<AssetId>,
    pub normal_texture: Option<AssetId>,
    pub metallic_roughness_texture: Option<AssetId>,
    pub emissive_texture: Option<AssetId>,
    pub albedo_factor: [f32; 4],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub emissive_factor: [f32; 3],
}

/// Audio asset data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioData {
    pub sample_rate: u32,
    pub channels: u16,
    pub format: AudioFormat,
    pub data: Vec<u8>,
    pub duration_ms: u64,
}

/// Audio formats
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioFormat {
    WAV,
    OGG,
    MP3,
    FLAC,
}

impl AssetManager {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        let mut loaders: HashMap<String, Arc<dyn AssetLoader>> = HashMap::new();
        let mut processors: HashMap<String, Arc<dyn AssetProcessor>> = HashMap::new();

        // Register default loaders
        loaders.insert("gltf".to_string(), Arc::new(GLTFLoader));
        loaders.insert("glb".to_string(), Arc::new(GLTFLoader));
        loaders.insert("obj".to_string(), Arc::new(OBJLoader));
        loaders.insert("png".to_string(), Arc::new(ImageLoader));
        loaders.insert("jpg".to_string(), Arc::new(ImageLoader));
        loaders.insert("jpeg".to_string(), Arc::new(ImageLoader));
        loaders.insert("wav".to_string(), Arc::new(AudioLoader));
        loaders.insert("ogg".to_string(), Arc::new(AudioLoader));

        // Register default processors
        processors.insert("mesh".to_string(), Arc::new(MeshProcessor));
        processors.insert("texture".to_string(), Arc::new(TextureProcessor));

        Self {
            cache: Arc::new(RwLock::new(AssetCache::new())),
            loaders,
            processors,
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Load an asset by file path
    pub async fn load_asset<P: AsRef<Path>>(&self, path: P) -> Result<AssetId> {
        let full_path = self.base_path.join(path.as_ref());

        info!("Loading asset: {}", full_path.display());

        // Check if already cached
        {
            let cache = self.cache.read().await;
            if let Some(asset_id) = cache.get_by_path(&full_path) {
                info!("Asset found in cache: {}", asset_id);
                return Ok(asset_id);
            }
        }

        // Determine file type and get appropriate loader
        let extension = full_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let loader = self.loaders.get(&extension)
            .ok_or_else(|| anyhow::anyhow!("No loader available for extension: {}", extension))?;

        // Load the asset
        let asset_data = loader.load(&full_path).await?;

        // Create metadata
        let metadata = fs::metadata(&full_path).await?;
        let asset_id = AssetId::new_v4();

        let asset_metadata = AssetMetadata {
            id: asset_id,
            name: full_path.file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("unknown")
                .to_string(),
            asset_type: self.determine_asset_type(&extension),
            file_path: full_path.clone(),
            file_size: metadata.len(),
            last_modified: metadata
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            dependencies: Vec::new(),
            tags: Vec::new(),
        };

        // Store in cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(asset_id, asset_data, asset_metadata);
        }

        info!("Asset loaded successfully: {}", asset_id);
        Ok(asset_id)
    }

    /// Get asset data by ID
    pub async fn get_asset(&self, asset_id: AssetId) -> Option<AssetData> {
        let cache = self.cache.read().await;
        cache.get_data(asset_id).cloned()
    }

    /// Get asset metadata by ID
    pub async fn get_metadata(&self, asset_id: AssetId) -> Option<AssetMetadata> {
        let cache = self.cache.read().await;
        cache.get_metadata(asset_id).cloned()
    }

    /// Process an asset (e.g., optimize, compress)
    pub async fn process_asset(&self, asset_id: AssetId, processor_name: &str) -> Result<()> {
        let processor = self.processors.get(processor_name)
            .ok_or_else(|| anyhow::anyhow!("No processor found: {}", processor_name))?;

        let (asset_data, metadata) = {
            let cache = self.cache.read().await;
            let data = cache.get_data(asset_id)
                .ok_or_else(|| anyhow::anyhow!("Asset not found: {}", asset_id))?
                .clone();
            let meta = cache.get_metadata(asset_id)
                .ok_or_else(|| anyhow::anyhow!("Asset metadata not found: {}", asset_id))?
                .clone();
            (data, meta)
        };

        let processed_data = processor.process(asset_data, &metadata).await?;

        // Update cache with processed data
        {
            let mut cache = self.cache.write().await;
            cache.update_data(asset_id, processed_data);
        }

        info!("Asset processed successfully: {}", asset_id);
        Ok(())
    }

    /// Unload an asset from cache
    pub async fn unload_asset(&self, asset_id: AssetId) -> bool {
        let mut cache = self.cache.write().await;
        cache.remove(asset_id)
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        cache.get_stats()
    }

    fn determine_asset_type(&self, extension: &str) -> AssetType {
        match extension {
            "gltf" | "glb" | "obj" | "fbx" => AssetType::Mesh,
            "png" | "jpg" | "jpeg" | "tga" | "bmp" => AssetType::Texture,
            "wav" | "ogg" | "mp3" | "flac" => AssetType::Audio,
            "glsl" | "hlsl" | "wgsl" => AssetType::Shader,
            "json" | "scene" => AssetType::Scene,
            _ => AssetType::Raw,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_asset_cache() {
        let mut cache = AssetCache::new();
        let asset_id = AssetId::new_v4();

        let data = AssetData::Raw(vec![1, 2, 3, 4]);
        let metadata = AssetMetadata {
            id: asset_id,
            name: "test".to_string(),
            asset_type: AssetType::Raw,
            file_path: PathBuf::from("test.bin"),
            file_size: 4,
            last_modified: 0,
            dependencies: Vec::new(),
            tags: Vec::new(),
        };

        cache.insert(asset_id, data, metadata);

        assert!(cache.get_data(asset_id).is_some());
        assert!(cache.get_metadata(asset_id).is_some());

        let stats = cache.get_stats();
        assert_eq!(stats.asset_count, 1);
    }

    #[tokio::test]
    async fn test_asset_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = AssetManager::new(temp_dir.path());

        let stats = manager.get_cache_stats().await;
        assert_eq!(stats.asset_count, 0);
    }
}