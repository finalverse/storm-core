// File: crates/storm-ai/src/lib.rs
// AI Dispatcher and ML models for StormCore
// Fixed imports and added missing prelude module

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Semaphore};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use anyhow::Result;

pub mod models;
pub mod dispatcher;
pub mod grok;
pub mod local_ml;

pub use dispatcher::*;
pub use models::*;

// Add prelude module for easy imports
pub mod prelude {
    pub use crate::{
        AIDispatcher, AIConfig, AIRequest, AIResponse, AITier, TaskType, AIContext,
        create_ai_request, GenerationParams, AIClient, AIModel,
    };
}

/// AI system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub grok_api_key: Option<String>,
    pub grok_api_endpoint: String,
    pub local_ml_enabled: bool,
    pub model_cache_dir: String,
    pub max_concurrent_requests: usize,
    pub ai_enhancement_timeout_ms: u64,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            grok_api_key: std::env::var("GROK_API_KEY").ok(),
            grok_api_endpoint: "https://api.x.ai/v1".to_string(),
            local_ml_enabled: true,
            model_cache_dir: "./models".to_string(),
            max_concurrent_requests: 10,
            ai_enhancement_timeout_ms: 200,
        }
    }
}

/// AI dispatcher - coordinates between local ML and external APIs
pub struct AIDispatcher {
    config: AIConfig,
    request_queue: mpsc::UnboundedSender<AIRequest>,
    response_handlers: Arc<RwLock<HashMap<uuid::Uuid, ResponseCallback>>>,
    grok_client: Option<grok::GrokClient>,
    local_ml: Option<local_ml::LocalMLEngine>,
    request_semaphore: Arc<Semaphore>,
}

/// AI request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRequest {
    pub id: uuid::Uuid,
    pub tier: AITier,
    pub task_type: TaskType,
    pub input_data: Vec<u8>,
    pub context: AIContext,
    pub timeout_ms: u64,
}

/// AI processing tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AITier {
    Low,    // Local ML, simple operations
    Mid,    // Local ML, complex operations
    High,   // External API, advanced operations
}

/// Types of AI tasks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    Pathfinding,
    AnomalyDetection,
    ContentGeneration,
    BehaviorPrediction,
    AssetOptimization,
    NetworkPrediction,
}

/// AI context for requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIContext {
    pub harmony_level: f32,
    pub entity_ids: Vec<u64>,
    pub protocol: String,
    pub world_state: Option<String>,
}

/// AI response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub request_id: uuid::Uuid,
    pub result: Result<Vec<u8>, String>,
    pub metrics: AIMetrics,
    pub confidence: f32,
}

/// AI performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIMetrics {
    pub latency_ms: u64,
    pub compute_cost: f32,
    pub model_used: String,
    pub cache_hit: bool,
}

type ResponseCallback = Box<dyn Fn(AIResponse) + Send + Sync>;

/// Generation parameters for AI content generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationParams {
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
}

impl Default for GenerationParams {
    fn default() -> Self {
        Self {
            max_tokens: Some(150),
            temperature: Some(0.7),
            top_p: Some(0.9),
            stop_sequences: None,
        }
    }
}

/// AI Client trait for external AI services
#[async_trait::async_trait]
pub trait AIClient: Send + Sync {
    async fn generate(&self, prompt: String, params: GenerationParams) -> Result<String>;
}

/// AI Model trait for ML models
#[async_trait::async_trait]
pub trait AIModel: Send + Sync {
    async fn predict(&self, input: &[f32]) -> Result<Vec<f32>>;
}

impl AIDispatcher {
    pub async fn new(config: &AIConfig) -> Result<Self> {
        info!("Initializing AI dispatcher with candle 0.9.1");

        let (tx, mut rx) = mpsc::unbounded_channel::<AIRequest>();
        let response_handlers = Arc::new(RwLock::new(HashMap::new()));

        // Initialize Grok client if API key is available
        let grok_client = if let Some(ref api_key) = config.grok_api_key {
            match grok::GrokClient::new(api_key, &config.grok_api_endpoint) {
                Ok(client) => Some(client),
                Err(e) => {
                    warn!("Failed to initialize Grok client: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Initialize local ML engine if enabled
        let local_ml = if config.local_ml_enabled {
            match local_ml::LocalMLEngine::new(&config.model_cache_dir).await {
                Ok(engine) => Some(engine),
                Err(e) => {
                    warn!("Failed to initialize local ML engine: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let request_semaphore = Arc::new(Semaphore::new(config.max_concurrent_requests));

        let dispatcher = Self {
            config: config.clone(),
            request_queue: tx,
            response_handlers: response_handlers.clone(),
            grok_client,
            local_ml,
            request_semaphore,
        };

        // Spawn request processing task
        let handlers = response_handlers.clone();
        let grok = dispatcher.grok_client.clone();
        let local = dispatcher.local_ml.clone();
        let semaphore = dispatcher.request_semaphore.clone();

        tokio::spawn(async move {
            while let Some(request) = rx.recv().await {
                let handlers = handlers.clone();
                let grok = grok.clone();
                let local = local.clone();
                let semaphore = semaphore.clone();

                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    Self::process_request(request, handlers, grok, local).await;
                });
            }
        });

        info!("AI dispatcher initialized successfully");
        Ok(dispatcher)
    }

    /// Submit an AI request for processing
    pub async fn submit_request<F>(&self, request: AIRequest, callback: F) -> Result<()>
    where
        F: Fn(AIResponse) + Send + Sync + 'static,
    {
        // Store callback
        {
            let mut handlers = self.response_handlers.write().await;
            handlers.insert(request.id, Box::new(callback));
        }

        // Queue request
        self.request_queue.send(request)?;
        Ok(())
    }

    /// Process pending requests (called from main update loop)
    pub async fn process_pending_requests(&self) -> Result<()> {
        // This is handled by the background task, so we just need to
        // clean up any expired callbacks
        self.cleanup_expired_handlers().await;
        Ok(())
    }

    /// Shutdown the AI dispatcher
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down AI dispatcher");

        if let Some(ref local_ml) = self.local_ml {
            local_ml.shutdown().await?;
        }

        Ok(())
    }

    async fn process_request(
        request: AIRequest,
        handlers: Arc<RwLock<HashMap<uuid::Uuid, ResponseCallback>>>,
        grok_client: Option<grok::GrokClient>,
        local_ml: Option<local_ml::LocalMLEngine>,
    ) {
        let start_time = std::time::Instant::now();

        let result = match request.tier {
            AITier::Low | AITier::Mid => {
                if let Some(ref local) = local_ml {
                    local.process_request(&request).await
                } else {
                    Err(anyhow::anyhow!("Local ML not available"))
                }
            }
            AITier::High => {
                if let Some(ref grok) = grok_client {
                    grok.process_request(&request).await
                } else {
                    Err(anyhow::anyhow!("Grok API not available"))
                }
            }
        };

        let latency = start_time.elapsed().as_millis() as u64;

        let response = AIResponse {
            request_id: request.id,
            result: result.map_err(|e| e.to_string()),
            metrics: AIMetrics {
                latency_ms: latency,
                compute_cost: 1.0, // Placeholder
                model_used: format!("{:?}", request.tier),
                cache_hit: false,
            },
            confidence: 0.8, // Placeholder
        };

        // Call response handler
        {
            let handlers_guard = handlers.read().await;
            if let Some(handler) = handlers_guard.get(&request.id) {
                handler(response);
            }
        }

        // Remove handler
        {
            let mut handlers_guard = handlers.write().await;
            handlers_guard.remove(&request.id);
        }
    }

    async fn cleanup_expired_handlers(&self) {
        // Remove handlers that have been waiting too long
        // Implementation would check timestamps and remove old handlers
    }
}

/// Helper function to create AI requests
pub fn create_ai_request(
    task_type: TaskType,
    tier: AITier,
    input_data: Vec<u8>,
    context: AIContext,
) -> AIRequest {
    AIRequest {
        id: uuid::Uuid::new_v4(),
        tier,
        task_type,
        input_data,
        context,
        timeout_ms: 5000, // Default timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_dispatcher_creation() {
        let config = AIConfig {
            grok_api_key: None,
            grok_api_endpoint: "test".to_string(),
            local_ml_enabled: false,
            model_cache_dir: "/tmp".to_string(),
            max_concurrent_requests: 1,
            ai_enhancement_timeout_ms: 1000,
        };

        let dispatcher = AIDispatcher::new(&config).await;
        assert!(dispatcher.is_ok());
    }
}