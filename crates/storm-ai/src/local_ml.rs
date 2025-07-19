// File: crates/storm-ai/src/local_ml.rs
// Local machine learning engine using candle-rs 0.9.1

use candle_core::{Device, Tensor, DType, Shape};
use candle_nn::{linear, Linear, Module, VarBuilder, VarMap};
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use crate::{AIRequest, TaskType};

pub struct LocalMLEngine {
    device: Device,
    models: HashMap<TaskType, Arc<dyn MLModel>>, // Use Arc to make it cloneable
    cache_dir: String,
}

// Make LocalMLEngine cloneable
impl Clone for LocalMLEngine {
    fn clone(&self) -> Self {
        Self {
            device: self.device.clone(),
            models: self.models.clone(),
            cache_dir: self.cache_dir.clone(),
        }
    }
}

trait MLModel: Send + Sync {
    fn predict(&self, input: &Tensor) -> Result<Tensor>;
    fn model_name(&self) -> &str;
}

struct PathfindingModel {
    linear: Linear,
    device: Device,
}

impl MLModel for PathfindingModel {
    fn predict(&self, input: &Tensor) -> Result<Tensor> {
        let output = self.linear.forward(input)?;
        Ok(output)
    }

    fn model_name(&self) -> &str {
        "pathfinding_v1"
    }
}

struct AnomalyDetectionModel {
    encoder: Linear,
    decoder: Linear,
    device: Device,
}

impl MLModel for AnomalyDetectionModel {
    fn predict(&self, input: &Tensor) -> Result<Tensor> {
        let encoded = self.encoder.forward(input)?;
        let decoded = self.decoder.forward(&encoded)?;
        Ok(decoded)
    }

    fn model_name(&self) -> &str {
        "anomaly_detection_v1"
    }
}

impl LocalMLEngine {
    pub async fn new(cache_dir: &str) -> Result<Self> {
        // Try to use CUDA if available, fallback to CPU
        let device = Device::cuda_if_available(0)?;
        tracing::info!("LocalMLEngine using device: {:?}", device);

        let mut models = HashMap::new();

        // Initialize pathfinding model
        let pathfinding_model = Self::create_pathfinding_model(&device)?;
        models.insert(TaskType::Pathfinding, Arc::new(pathfinding_model) as Arc<dyn MLModel>);

        // Initialize anomaly detection model
        let anomaly_model = Self::create_anomaly_detection_model(&device)?;
        models.insert(TaskType::AnomalyDetection, Arc::new(anomaly_model) as Arc<dyn MLModel>);

        Ok(Self {
            device,
            models,
            cache_dir: cache_dir.to_string(),
        })
    }

    pub async fn process_request(&self, request: &AIRequest) -> Result<Vec<u8>> {
        if let Some(model) = self.models.get(&request.task_type) {
            // Convert input data to tensor
            let input_tensor = self.bytes_to_tensor(&request.input_data)?;

            // Run prediction
            let output_tensor = model.predict(&input_tensor)?;

            // Convert tensor back to bytes
            let output_bytes = self.tensor_to_bytes(&output_tensor)?;

            Ok(output_bytes)
        } else {
            Err(anyhow::anyhow!("Model not available for task: {:?}", request.task_type))
        }
    }

    pub async fn shutdown(&self) -> Result<()> {
        // Cleanup any resources
        Ok(())
    }

    fn create_pathfinding_model(device: &Device) -> Result<PathfindingModel> {
        // Create a simple linear model for pathfinding with modern candle API
        let mut varmap = VarMap::new();
        let vs = VarBuilder::from_varmap(&varmap, DType::F32, device);

        let linear = linear(64, 32, vs.pp("pathfinding"))?;

        Ok(PathfindingModel {
            linear,
            device: device.clone(),
        })
    }

    fn create_anomaly_detection_model(device: &Device) -> Result<AnomalyDetectionModel> {
        // Create an autoencoder for anomaly detection with modern API
        let mut varmap = VarMap::new();
        let vs = VarBuilder::from_varmap(&varmap, DType::F32, device);

        let encoder = linear(128, 64, vs.pp("encoder"))?;
        let decoder = linear(64, 128, vs.pp("decoder"))?;

        Ok(AnomalyDetectionModel {
            encoder,
            decoder,
            device: device.clone(),
        })
    }

    fn bytes_to_tensor(&self, bytes: &[u8]) -> Result<Tensor> {
        // Convert bytes to float tensor with proper error handling
        let floats: Vec<f32> = bytes.iter().map(|&b| b as f32 / 255.0).collect();

        // Pad or truncate to expected input size
        let mut padded_floats = vec![0.0f32; 64]; // Assuming 64-element input
        for (i, &val) in floats.iter().take(64).enumerate() {
            padded_floats[i] = val;
        }

        // Create tensor with explicit shape
        let shape = Shape::from_dims(&[1, 64]);
        Tensor::from_vec(padded_floats, shape, &self.device)
            .map_err(|e| anyhow::anyhow!("Tensor creation error: {}", e))
    }

    fn tensor_to_bytes(&self, tensor: &Tensor) -> Result<Vec<u8>> {
        // Convert tensor back to bytes with modern API
        let data = tensor.to_vec2::<f32>()
            .map_err(|e| anyhow::anyhow!("Tensor conversion error: {}", e))?;

        let bytes: Vec<u8> = data.into_iter()
            .flatten()
            .map(|f| (f * 255.0).clamp(0.0, 255.0) as u8)
            .collect();

        Ok(bytes)
    }
}