// File: crates/storm-finalverse/src/websocket.rs
// WebSocket handler for real-time Finalverse communication
// Handles bidirectional streaming of world events and user actions

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};
use tracing::{info, warn, error, debug};
use anyhow::Result;

use crate::{FinalverseMessage, EntityUpdate, EntityData};

/// WebSocket connection manager for Finalverse protocol
pub struct FinalverseWebSocket {
    /// Active WebSocket connections keyed by session ID
    connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
    /// Outbound message channel sender
    outbound_tx: mpsc::UnboundedSender<FinalverseMessage>,
    /// Inbound message channel receiver
    inbound_rx: Arc<RwLock<mpsc::UnboundedReceiver<FinalverseMessage>>>,
    /// Connection configuration
    config: WebSocketConfig,
}

/// Individual WebSocket connection state
pub struct WebSocketConnection {
    /// Session identifier for this connection
    pub session_id: String,
    /// WebSocket stream for communication
    pub stream: Option<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>,
    /// Connection state tracking
    pub state: ConnectionState,
    /// Last ping timestamp for health monitoring
    pub last_ping: std::time::Instant,
    /// Message queue for pending outbound messages
    pub pending_messages: Vec<FinalverseMessage>,
}

/// WebSocket connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    /// Connection is being established
    Connecting,
    /// Connection is active and ready
    Connected,
    /// Connection is authenticating
    Authenticating,
    /// Connection is authenticated and ready for world data
    Authenticated,
    /// Connection is being closed gracefully
    Disconnecting,
    /// Connection is closed or failed
    Disconnected,
}

/// WebSocket configuration parameters
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Ping interval for connection health checks
    pub ping_interval: std::time::Duration,
    /// Connection timeout duration
    pub connection_timeout: std::time::Duration,
    /// Maximum number of pending messages before dropping
    pub max_pending_messages: usize,
    /// Enable compression for messages
    pub enable_compression: bool,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            max_message_size: 1024 * 1024, // 1MB
            ping_interval: std::time::Duration::from_secs(30),
            connection_timeout: std::time::Duration::from_secs(10),
            max_pending_messages: 100,
            enable_compression: true,
        }
    }
}

impl FinalverseWebSocket {
    /// Create a new WebSocket manager with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(WebSocketConfig::default())
    }

    /// Create a new WebSocket manager with custom configuration
    pub fn with_config(config: WebSocketConfig) -> Result<Self> {
        let (outbound_tx, _outbound_rx) = mpsc::unbounded_channel();
        let (_inbound_tx, inbound_rx) = mpsc::unbounded_channel();

        Ok(Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            outbound_tx,
            inbound_rx: Arc::new(RwLock::new(inbound_rx)),
            config,
        })
    }

    /// Connect to a Finalverse server via WebSocket
    pub async fn connect(&self, url: &str, session_id: String) -> Result<()> {
        info!("Connecting to Finalverse server at: {}", url);

        // Establish WebSocket connection
        let (ws_stream, response) = connect_async(url).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to WebSocket: {}", e))?;

        info!("WebSocket connection established, response: {:?}", response.status());

        // Create connection object
        let connection = WebSocketConnection {
            session_id: session_id.clone(),
            stream: Some(ws_stream),
            state: ConnectionState::Connected,
            last_ping: std::time::Instant::now(),
            pending_messages: Vec::new(),
        };

        // Store connection
        let mut connections = self.connections.write().await;
        connections.insert(session_id.clone(), connection);
        drop(connections);

        // Start message processing loop
        self.start_message_loop(session_id).await?;

        Ok(())
    }

    /// Start the message processing loop for a connection - fixed match exhaustiveness
    async fn start_message_loop(&self, session_id: String) -> Result<()> {
        let connections = self.connections.clone();
        let outbound_tx = self.outbound_tx.clone();

        tokio::spawn(async move {
            loop {
                // Get connection
                let mut connections_guard = connections.write().await;
                let connection = match connections_guard.get_mut(&session_id) {
                    Some(conn) => conn,
                    None => {
                        warn!("Connection not found for session: {}", session_id);
                        break;
                    }
                };

                // Check if we have a stream
                let mut stream = match connection.stream.take() {
                    Some(stream) => stream,
                    None => {
                        error!("No stream available for session: {}", session_id);
                        break;
                    }
                };

                drop(connections_guard);

                // Process messages - fixed match to be exhaustive
                match stream.next().await {
                    Some(Ok(Message::Text(text))) => {
                        debug!("Received text message: {}", text);
                        if let Err(e) = Self::handle_text_message(&text, &outbound_tx).await {
                            error!("Error handling text message: {}", e);
                        }
                    }
                    Some(Ok(Message::Binary(data))) => {
                        debug!("Received binary message: {} bytes", data.len());
                        if let Err(e) = Self::handle_binary_message(&data, &outbound_tx).await {
                            error!("Error handling binary message: {}", e);
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        debug!("Received ping, sending pong");
                        if let Err(e) = stream.send(Message::Pong(data)).await {
                            error!("Error sending pong: {}", e);
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        debug!("Received pong");
                        // Update last ping time
                        let mut connections_guard = connections.write().await;
                        if let Some(conn) = connections_guard.get_mut(&session_id) {
                            conn.last_ping = std::time::Instant::now();
                        }
                    }
                    Some(Ok(Message::Close(close_frame))) => {
                        info!("Received close message for session: {} ({:?})", session_id, close_frame);
                        break;
                    }
                    Some(Ok(Message::Frame(_))) => {
                        // Handle raw frame (usually not needed in application code)
                        debug!("Received raw frame for session: {}", session_id);
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error for session {}: {}", session_id, e);
                        break;
                    }
                    None => {
                        info!("WebSocket stream ended for session: {}", session_id);
                        break;
                    }
                }

                // Restore stream to connection
                let mut connections_guard = connections.write().await;
                if let Some(connection) = connections_guard.get_mut(&session_id) {
                    connection.stream = Some(stream);
                }
            }

            // Clean up connection
            let mut connections_guard = connections.write().await;
            connections_guard.remove(&session_id);
            info!("Connection cleanup completed for session: {}", session_id);
        });

        Ok(())
    }

    /// Handle incoming text messages (JSON)
    async fn handle_text_message(
        text: &str,
        outbound_tx: &mpsc::UnboundedSender<FinalverseMessage>,
    ) -> Result<()> {
        // Parse JSON message
        let message: FinalverseMessage = serde_json::from_str(text)
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON message: {}", e))?;

        debug!("Parsed message: {:?}", message);

        // Forward message to internal processing
        outbound_tx.send(message)
            .map_err(|e| anyhow::anyhow!("Failed to forward message: {}", e))?;

        Ok(())
    }

    /// Handle incoming binary messages
    async fn handle_binary_message(
        data: &[u8],
        _outbound_tx: &mpsc::UnboundedSender<FinalverseMessage>,
    ) -> Result<()> {
        // For now, just log binary messages
        // In the future, this could handle compressed data or binary assets
        debug!("Received binary data: {} bytes", data.len());
        Ok(())
    }

    /// Send a message to a specific session
    pub async fn send_message(&self, session_id: &str, message: &FinalverseMessage) -> Result<()> {
        let mut connections = self.connections.write().await;
        let connection = connections.get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        // Serialize message to JSON
        let json = serde_json::to_string(message)
            .map_err(|e| anyhow::anyhow!("Failed to serialize message: {}", e))?;

        // Send via WebSocket if stream is available
        if let Some(ref mut stream) = connection.stream {
            stream.send(Message::Text(json)).await
                .map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))?;
        } else {
            // Queue message if stream is not available
            connection.pending_messages.push(message.clone());
            warn!("Queued message for session {}, stream not available", session_id);
        }

        Ok(())
    }

    /// Broadcast a message to all connected sessions
    pub async fn broadcast_message(&self, message: &FinalverseMessage) -> Result<()> {
        let connections = self.connections.read().await;

        for (session_id, _) in connections.iter() {
            if let Err(e) = self.send_message(session_id, message).await {
                warn!("Failed to send message to session {}: {}", session_id, e);
            }
        }

        Ok(())
    }

    /// Disconnect a specific session
    pub async fn disconnect(&self, session_id: &str) -> Result<()> {
        let mut connections = self.connections.write().await;

        if let Some(mut connection) = connections.remove(session_id) {
            connection.state = ConnectionState::Disconnecting;

            // Send close message if stream is available
            if let Some(ref mut stream) = connection.stream {
                let _ = stream.send(Message::Close(None)).await;
            }

            info!("Disconnected session: {}", session_id);
        }

        Ok(())
    }

    /// Disconnect all sessions
    pub async fn disconnect_all(&self) -> Result<()> {
        let mut connections = self.connections.write().await;

        for (session_id, connection) in connections.iter_mut() {
            connection.state = ConnectionState::Disconnecting;

            if let Some(ref mut stream) = connection.stream {
                let _ = stream.send(Message::Close(None)).await;
            }

            info!("Disconnected session: {}", session_id);
        }

        connections.clear();
        info!("All sessions disconnected");

        Ok(())
    }

    /// Get connection state for a session
    pub async fn get_connection_state(&self, session_id: &str) -> Option<ConnectionState> {
        let connections = self.connections.read().await;
        connections.get(session_id).map(|conn| conn.state.clone())
    }

    /// Get count of active connections
    pub async fn active_connections_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    /// Get list of all session IDs
    pub async fn get_session_ids(&self) -> Vec<String> {
        let connections = self.connections.read().await;
        connections.keys().cloned().collect()
    }

    /// Start periodic ping to maintain connection health
    pub async fn start_ping_loop(&self) -> Result<()> {
        let connections = self.connections.clone();
        let ping_interval = self.config.ping_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(ping_interval);

            loop {
                interval.tick().await;

                let mut connections_guard = connections.write().await;
                let mut sessions_to_remove = Vec::new();

                for (session_id, connection) in connections_guard.iter_mut() {
                    // Check if connection is stale
                    if connection.last_ping.elapsed() > ping_interval * 2 {
                        warn!("Connection stale for session: {}", session_id);
                        sessions_to_remove.push(session_id.clone());
                        continue;
                    }

                    // Send ping
                    if let Some(ref mut stream) = connection.stream {
                        if let Err(e) = stream.send(Message::Ping(vec![])).await {
                            error!("Failed to send ping to session {}: {}", session_id, e);
                            sessions_to_remove.push(session_id.clone());
                        }
                    }
                }

                // Remove stale connections
                for session_id in sessions_to_remove {
                    connections_guard.remove(&session_id);
                    info!("Removed stale connection: {}", session_id);
                }
            }
        });

        Ok(())
    }
}

impl Default for FinalverseWebSocket {
    fn default() -> Self {
        Self::new().expect("Failed to create default WebSocket manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_creation() {
        let ws = FinalverseWebSocket::new();
        assert!(ws.is_ok());

        let ws = ws.unwrap();
        assert_eq!(ws.active_connections_count().await, 0);
    }

    #[test]
    fn test_connection_state() {
        let state = ConnectionState::Connected;
        assert_eq!(state, ConnectionState::Connected);
        assert_ne!(state, ConnectionState::Disconnected);
    }

    #[test]
    fn test_websocket_config() {
        let config = WebSocketConfig::default();
        assert_eq!(config.max_message_size, 1024 * 1024);
        assert!(config.enable_compression);
        assert_eq!(config.ping_interval, std::time::Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_session_management() {
        let ws = FinalverseWebSocket::new().unwrap();

        // Initially no sessions
        assert_eq!(ws.get_session_ids().await.len(), 0);

        // Test getting state of non-existent session
        assert!(ws.get_connection_state("nonexistent").await.is_none());
    }

    #[tokio::test]
    async fn test_message_serialization() {
        let message = FinalverseMessage::Chat {
            message: "Hello, WebSocket!".to_string(),
            channel: "test".to_string(),
        };

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("Hello, WebSocket!"));
        assert!(json.contains("test"));

        let deserialized: FinalverseMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            FinalverseMessage::Chat { message: msg, channel: ch } => {
                assert_eq!(msg, "Hello, WebSocket!");
                assert_eq!(ch, "test");
            }
            _ => panic!("Wrong message type after deserialization"),
        }
    }

    #[test]
    fn test_connection_state_transitions() {
        let mut state = ConnectionState::Connecting;
        assert_eq!(state, ConnectionState::Connecting);

        state = ConnectionState::Connected;
        assert_eq!(state, ConnectionState::Connected);

        state = ConnectionState::Disconnecting;
        assert_eq!(state, ConnectionState::Disconnecting);

        state = ConnectionState::Disconnected;
        assert_eq!(state, ConnectionState::Disconnected);
    }
}