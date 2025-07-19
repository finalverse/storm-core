// File: crates/storm-ai/src/grok.rs
// Grok API client for external AI processing

use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use crate::{AIRequest, TaskType};

#[derive(Clone)]
pub struct GrokClient {
    client: Client,
    api_key: String,
    endpoint: String,
}

#[derive(Serialize)]
struct GrokRequest {
    model: String,
    messages: Vec<GrokMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize)]
struct GrokMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct GrokResponse {
    choices: Vec<GrokChoice>,
}

#[derive(Deserialize)]
struct GrokChoice {
    message: GrokMessageResponse,
}

#[derive(Deserialize)]
struct GrokMessageResponse {
    content: String,
}

impl GrokClient {
    pub fn new(api_key: &str, endpoint: &str) -> Result<Self> {
        let client = Client::new();

        Ok(Self {
            client,
            api_key: api_key.to_string(),
            endpoint: endpoint.to_string(),
        })
    }

    pub async fn process_request(&self, request: &AIRequest) -> Result<Vec<u8>> {
        let prompt = self.create_prompt_for_task(request)?;

        let grok_request = GrokRequest {
            model: "grok-beta".to_string(),
            messages: vec![
                GrokMessage {
                    role: "system".to_string(),
                    content: "You are an AI assistant for a 3D virtual world engine. Provide concise, technical responses.".to_string(),
                },
                GrokMessage {
                    role: "user".to_string(),
                    content: prompt,
                }
            ],
            max_tokens: 1000,
            temperature: 0.7,
        };

        let response = self.client
            .post(&format!("{}/chat/completions", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&grok_request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Grok API error: {}", response.status()));
        }

        let grok_response: GrokResponse = response.json().await?;

        if let Some(choice) = grok_response.choices.first() {
            Ok(choice.message.content.as_bytes().to_vec())
        } else {
            Err(anyhow::anyhow!("No response from Grok API"))
        }
    }

    fn create_prompt_for_task(&self, request: &AIRequest) -> Result<String> {
        let input_str = String::from_utf8_lossy(&request.input_data);

        let prompt = match request.task_type {
            TaskType::ContentGeneration => {
                format!("Generate 3D virtual world content based on: {}", input_str)
            }
            TaskType::BehaviorPrediction => {
                format!("Predict NPC behavior patterns for: {}", input_str)
            }
            TaskType::AssetOptimization => {
                format!("Suggest optimizations for 3D assets: {}", input_str)
            }
            _ => {
                format!("Process the following virtual world data: {}", input_str)
            }
        };

        Ok(prompt)
    }
}