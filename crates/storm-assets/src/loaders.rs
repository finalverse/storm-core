// File: crates/storm-assets/src/loaders.rs
// Asset loaders for different file formats

use async_trait::async_trait;
use std::path::Path;
use anyhow::Result;
use crate::AssetData;

/// Asset loader trait
#[async_trait]
pub trait AssetLoader: Send + Sync {
    async fn load(&self, path: &Path) -> Result<AssetData>;
    fn supported_extensions(&self) -> Vec<&'static str>;
}

/// GLTF loader
pub struct GLTFLoader;

#[async_trait]
impl AssetLoader for GLTFLoader {
    async fn load(&self, _path: &Path) -> Result<AssetData> {
        // TODO: Implement actual GLTF loading
        Ok(AssetData::Raw(vec![]))
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["gltf", "glb"]
    }
}

/// OBJ loader
pub struct OBJLoader;

#[async_trait]
impl AssetLoader for OBJLoader {
    async fn load(&self, _path: &Path) -> Result<AssetData> {
        // TODO: Implement actual OBJ loading
        Ok(AssetData::Raw(vec![]))
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["obj"]
    }
}

/// Image loader
pub struct ImageLoader;

#[async_trait]
impl AssetLoader for ImageLoader {
    async fn load(&self, _path: &Path) -> Result<AssetData> {
        // TODO: Implement actual image loading using the image crate
        Ok(AssetData::Raw(vec![]))
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["png", "jpg", "jpeg", "tga", "bmp"]
    }
}

/// Audio loader
pub struct AudioLoader;

#[async_trait]
impl AssetLoader for AudioLoader {
    async fn load(&self, _path: &Path) -> Result<AssetData> {
        // TODO: Implement actual audio loading
        Ok(AssetData::Raw(vec![]))
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["wav", "ogg", "mp3"]
    }
}