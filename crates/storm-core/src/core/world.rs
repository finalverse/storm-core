// File: crates/storm-core/src/core/world.rs
// Virtual world configuration and connection types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for connecting to a virtual world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    pub name: String,
    pub url: String,
    pub protocol: ProtocolType,
    pub credentials: Option<WorldCredentials>,
}

/// Supported virtual world protocols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolType {
    OpenSim,
    Finalverse,
}

/// Authentication credentials for world connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldCredentials {
    pub username: String,
    pub password: String,
    pub additional_fields: HashMap<String, String>,
}

impl WorldConfig {
    pub fn new_opensim(name: &str, grid_url: &str, username: &str, password: &str) -> Self {
        Self {
            name: name.to_string(),
            url: grid_url.to_string(),
            protocol: ProtocolType::OpenSim,
            credentials: Some(WorldCredentials {
                username: username.to_string(),
                password: password.to_string(),
                additional_fields: HashMap::new(),
            }),
        }
    }

    pub fn new_finalverse(name: &str, server_url: &str) -> Self {
        Self {
            name: name.to_string(),
            url: server_url.to_string(),
            protocol: ProtocolType::Finalverse,
            credentials: None,
        }
    }
}