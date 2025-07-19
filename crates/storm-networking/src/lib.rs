// File: crates/storm-networking/src/lib.rs
// Async networking layer for StormCore
// Handles multiple protocols and connection management

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};
use anyhow::Result;

pub mod packet;
pub mod connection;
pub mod protocol;

pub use packet::*;
pub use connection::*;
pub use protocol::*;

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub max_connections: usize,
    pub connection_timeout_ms: u64,
    pub packet_buffer_size: usize,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
}

/// Network manager - coordinates all network operations
pub struct NetworkManager {
    config: NetworkConfig,
    connections: Arc<RwLock<HashMap<ConnectionId, Connection>>>,
    listeners: Arc<Mutex<Vec<Listener>>>,
    packet_handlers: Arc<RwLock<HashMap<ProtocolType, Box<dyn PacketHandler>>>>,

    // Channels for communication
    incoming_packets: mpsc::UnboundedSender<IncomingPacket>,
    outgoing_packets: mpsc::UnboundedReceiver<OutgoingPacket>,
}

/// Connection identifier
pub type ConnectionId = uuid::Uuid;

/// Protocol types supported by the network layer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtocolType {
    LLUDP,     // OpenSim/MutSea UDP protocol
    WebSocket, // Finalverse WebSocket protocol
    QUIC,      // Future high-performance protocol
}

/// Network listener for incoming connections
pub enum Listener {
    Tcp(TcpListener),
    Udp(UdpSocket),
    WebSocket(TcpListener),
}

/// Connection wrapper for different transport types
pub enum Connection {
    Tcp(TcpConnection),
    Udp(UdpConnection),
    WebSocket(WebSocketConnection),
}

/// TCP connection wrapper
pub struct TcpConnection {
    id: ConnectionId,
    stream: tokio::net::TcpStream,
    remote_addr: SocketAddr,
    last_activity: std::time::Instant,
}

/// UDP connection wrapper
pub struct UdpConnection {
    id: ConnectionId,
    socket: Arc<UdpSocket>,
    remote_addr: SocketAddr,
    last_activity: std::time::Instant,
}

/// WebSocket connection wrapper
pub struct WebSocketConnection {
    id: ConnectionId,
    stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    remote_addr: SocketAddr,
    last_activity: std::time::Instant,
}

/// Incoming packet from network
#[derive(Debug)]
pub struct IncomingPacket {
    pub connection_id: ConnectionId,
    pub protocol: ProtocolType,
    pub data: Vec<u8>,
    pub timestamp: std::time::Instant,
}

/// Outgoing packet to network
#[derive(Debug)]
pub struct OutgoingPacket {
    pub connection_id: ConnectionId,
    pub protocol: ProtocolType,
    pub data: Vec<u8>,
    pub priority: PacketPriority,
}

/// Packet priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PacketPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Trait for handling protocol-specific packets
pub trait PacketHandler: Send + Sync {
    fn handle_packet(&self, packet: &IncomingPacket) -> Result<Vec<OutgoingPacket>>;
    fn protocol_type(&self) -> ProtocolType;
}

impl NetworkManager {
    pub async fn new(config: &NetworkConfig) -> Result<Self> {
        info!("Initializing network manager");

        let (incoming_tx, _incoming_rx) = mpsc::unbounded_channel();
        let (_outgoing_tx, outgoing_rx) = mpsc::unbounded_channel();

        let manager = Self {
            config: config.clone(),
            connections: Arc::new(RwLock::new(HashMap::new())),
            listeners: Arc::new(Mutex::new(Vec::new())),
            packet_handlers: Arc::new(RwLock::new(HashMap::new())),
            incoming_packets: incoming_tx,
            outgoing_packets: outgoing_rx,
        };

        info!("Network manager initialized successfully");
        Ok(manager)
    }

    /// Start listening on a specific address and protocol
    pub async fn start_listener(&self, addr: SocketAddr, protocol: ProtocolType) -> Result<()> {
        info!("Starting listener for {:?} on {}", protocol, addr);

        let listener = match protocol {
            ProtocolType::LLUDP => {
                let socket = UdpSocket::bind(addr).await?;
                Listener::Udp(socket)
            }
            ProtocolType::WebSocket => {
                let tcp_listener = TcpListener::bind(addr).await?;
                Listener::WebSocket(tcp_listener)
            }
            ProtocolType::QUIC => {
                // QUIC implementation would go here
                return Err(anyhow::anyhow!("QUIC not implemented yet"));
            }
        };

        let mut listeners = self.listeners.lock().await;
        listeners.push(listener);

        Ok(())
    }

    /// Connect to a remote address
    pub async fn connect(&self, addr: SocketAddr, protocol: ProtocolType) -> Result<ConnectionId> {
        info!("Connecting to {} via {:?}", addr, protocol);

        let connection_id = ConnectionId::new_v4();

        let connection = match protocol {
            ProtocolType::LLUDP => {
                let socket = UdpSocket::bind("0.0.0.0:0").await?;
                socket.connect(addr).await?;
                Connection::Udp(UdpConnection {
                    id: connection_id,
                    socket: Arc::new(socket),
                    remote_addr: addr,
                    last_activity: std::time::Instant::now(),
                })
            }
            ProtocolType::WebSocket => {
                let (ws_stream, _) = tokio_tungstenite::connect_async(
                    format!("ws://{}", addr)
                ).await?;

                Connection::WebSocket(WebSocketConnection {
                    id: connection_id,
                    stream: ws_stream,
                    remote_addr: addr,
                    last_activity: std::time::Instant::now(),
                })
            }
            ProtocolType::QUIC => {
                return Err(anyhow::anyhow!("QUIC not implemented yet"));
            }
        };

        let mut connections = self.connections.write().await;
        connections.insert(connection_id, connection);

        info!("Connected to {} with ID: {}", addr, connection_id);
        Ok(connection_id)
    }

    /// Send a packet to a specific connection
    pub async fn send_packet(&self, connection_id: ConnectionId, data: Vec<u8>, priority: PacketPriority) -> Result<()> {
        let connections = self.connections.read().await;

        if let Some(connection) = connections.get(&connection_id) {
            match connection {
                Connection::Udp(udp_conn) => {
                    udp_conn.socket.send(&data).await?;
                }
                Connection::WebSocket(ws_conn) => {
                    // WebSocket sending would be implemented here
                    // This is a simplified placeholder
                }
                Connection::Tcp(tcp_conn) => {
                    // TCP sending would be implemented here
                }
            }
        } else {
            return Err(anyhow::anyhow!("Connection not found: {}", connection_id));
        }

        Ok(())
    }

    /// Register a packet handler for a protocol
    pub async fn register_packet_handler<H: PacketHandler + 'static>(&self, handler: H) {
        let mut handlers = self.packet_handlers.write().await;
        handlers.insert(handler.protocol_type(), Box::new(handler));
    }

    /// Update network manager (called from main loop)
    pub async fn update(&self) -> Result<()> {
        // Process outgoing packets
        // Handle connection timeouts
        // Clean up dead connections
        self.cleanup_dead_connections().await?;
        Ok(())
    }

    /// Shutdown network manager
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down network manager");

        // Close all connections
        let mut connections = self.connections.write().await;
        connections.clear();

        // Close all listeners
        let mut listeners = self.listeners.lock().await;
        listeners.clear();

        info!("Network manager shutdown complete");
        Ok(())
    }

    async fn cleanup_dead_connections(&self) -> Result<()> {
        let mut connections = self.connections.write().await;
        let timeout = std::time::Duration::from_millis(self.config.connection_timeout_ms);
        let now = std::time::Instant::now();

        connections.retain(|_id, connection| {
            let last_activity = match connection {
                Connection::Tcp(tcp) => tcp.last_activity,
                Connection::Udp(udp) => udp.last_activity,
                Connection::WebSocket(ws) => ws.last_activity,
            };

            now.duration_since(last_activity) < timeout
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_manager_creation() {
        let config = NetworkConfig {
            max_connections: 100,
            connection_timeout_ms: 5000,
            packet_buffer_size: 8192,
            compression_enabled: true,
            encryption_enabled: true,
        };

        let manager = NetworkManager::new(&config).await;
        assert!(manager.is_ok());
    }
}