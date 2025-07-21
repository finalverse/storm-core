// File: Core/WorldConnectionManager.swift
// Description: Manages world connection states and statistics
// Tracks active connections, history, and provides connection monitoring

import Foundation
import Combine
import os

class WorldConnectionManager: ObservableObject {
    @Published var activeConnections: [UUID: ConnectionState] = [:]
    @Published var connectionHistory: [ConnectionHistoryEntry] = []
    @Published var isConnecting: Bool = false
    
    private let logger = Logger(subsystem: "com.storm.client", category: "ConnectionManager")
    private let userDefaults = UserDefaults.standard
    private var cancellables = Set<AnyCancellable>()
    
    init() {
        loadConnectionHistory()
        setupPeriodicUpdates()
    }
    
    // MARK: - Connection Management
    
    func addConnection(_ world: WorldConfiguration, connectionId: UUID) {
        let state = ConnectionState(
            world: world,
            connectionId: connectionId,
            status: .connecting,
            connectedAt: Date()
        )
        
        activeConnections[world.id] = state
        
        let historyEntry = ConnectionHistoryEntry(
            world: world,
            connectedAt: Date(),
            duration: 0,
            success: true
        )
        
        connectionHistory.insert(historyEntry, at: 0)
        saveConnectionHistory()
        
        logger.info("Added connection to \(world.name)")
        
        // Post notification
        NotificationCenter.default.post(
            name: .worldConnectionAdded,
            object: state
        )
    }
    
    func updateConnectionStatus(_ worldId: UUID, status: ConnectionStatus) {
        guard activeConnections[worldId] != nil else { return }
        
        activeConnections[worldId]?.status = status
        activeConnections[worldId]?.lastActivity = Date()
        
        logger.info("Updated connection status for world \(worldId): \(status)")
        
        // Update history entry
        if let historyIndex = connectionHistory.firstIndex(where: { $0.world.id == worldId }) {
            connectionHistory[historyIndex].status = status
        }
        
        // Post notification
        NotificationCenter.default.post(
            name: .worldConnectionStatusChanged,
            object: activeConnections[worldId]
        )
    }
    
    func updateConnectionStatistics(_ worldId: UUID, statistics: ConnectionStatistics) {
        activeConnections[worldId]?.statistics = statistics
        activeConnections[worldId]?.lastActivity = Date()
    }
    
    func removeConnection(_ worldId: UUID) {
        guard let connection = activeConnections[worldId] else { return }
        
        // Update history with final duration
        if let historyIndex = connectionHistory.firstIndex(where: { $0.world.id == worldId }) {
            let duration = Date().timeIntervalSince(connection.connectedAt)
            connectionHistory[historyIndex].duration = duration
            connectionHistory[historyIndex].disconnectedAt = Date()
        }
        
        activeConnections.removeValue(forKey: worldId)
        saveConnectionHistory()
        
        logger.info("Removed connection for world \(worldId)")
        
        // Post notification
        NotificationCenter.default.post(
            name: .worldConnectionRemoved,
            object: connection
        )
    }
    
    // MARK: - Connection Queries
    
    var hasActiveConnections: Bool {
        return !activeConnections.isEmpty
    }
    
    var connectedWorldsCount: Int {
        return activeConnections.values.filter { $0.isActive }.count
    }
    
    func getConnection(for worldId: UUID) -> ConnectionState? {
        return activeConnections[worldId]
    }
    
    func getActiveConnections() -> [ConnectionState] {
        return Array(activeConnections.values).filter { $0.isActive }
    }
    
    func getConnectionsByStatus(_ status: ConnectionStatus) -> [ConnectionState] {
        return Array(activeConnections.values).filter { $0.status == status }
    }
    
    func getConnectionsForProtocol(_ protocol: ProtocolType) -> [ConnectionState] {
        return Array(activeConnections.values).filter { $0.world.protocol == `protocol` }
    }
    
    // MARK: - Statistics and Monitoring
    
    func getOverallStatistics() -> OverallConnectionStatistics {
        let connections = Array(activeConnections.values)
        
        var totalBytesReceived: UInt64 = 0
        var totalBytesSent: UInt64 = 0
        var totalPacketsReceived: UInt32 = 0
        var totalPacketsSent: UInt32 = 0
        var latencies: [TimeInterval] = []
        
        for connection in connections {
            totalBytesReceived += connection.statistics.bytesReceived
            totalBytesSent += connection.statistics.bytesSent
            totalPacketsReceived += connection.statistics.packetsReceived
            totalPacketsSent += connection.statistics.packetsSent
            
            if connection.statistics.averageLatency > 0 {
                latencies.append(connection.statistics.averageLatency)
            }
        }
        
        let averageLatency = latencies.isEmpty ? 0 : latencies.reduce(0, +) / Double(latencies.count)
        
        return OverallConnectionStatistics(
            activeConnections: connections.count,
            totalBytesReceived: totalBytesReceived,
            totalBytesSent: totalBytesSent,
            totalPacketsReceived: totalPacketsReceived,
            totalPacketsSent: totalPacketsSent,
            averageLatency: averageLatency,
            connectionUptime: connections.map { $0.duration }.reduce(0, +)
        )
    }
    
    func getConnectionHealth(_ worldId: UUID) -> ConnectionHealth {
        guard let connection = activeConnections[worldId] else {
            return ConnectionHealth.disconnected
        }
        
        let timeSinceLastActivity = Date().timeIntervalSince(connection.lastActivity)
        let latency = connection.statistics.averageLatency
        
        // Determine health based on various factors
        if !connection.isActive {
            return .disconnected
        }
        
        if timeSinceLastActivity > 60 { // No activity for 1 minute
            return .poor
        }
        
        if latency > 0.5 { // High latency
            return .poor
        } else if latency > 0.2 {
            return .fair
        } else if latency > 0.1 {
            return .good
        } else {
            return .excellent
        }
    }
    
    // MARK: - Connection History Management
    
    private func loadConnectionHistory() {
        if let data = userDefaults.data(forKey: "connectionHistory"),
           let history = try? JSONDecoder().decode([ConnectionHistoryEntry].self, from: data) {
            connectionHistory = history
            logger.info("Loaded \(history.count) connection history entries")
        }
    }
    
    private func saveConnectionHistory() {
        // Keep only last 100 entries
        if connectionHistory.count > 100 {
            connectionHistory = Array(connectionHistory.prefix(100))
        }
        
        if let data = try? JSONEncoder().encode(connectionHistory) {
            userDefaults.set(data, forKey: "connectionHistory")
        }
    }
    
    func clearConnectionHistory() {
        connectionHistory.removeAll()
        saveConnectionHistory()
        logger.info("Cleared connection history")
    }
    
    // MARK: - Periodic Updates
    
    private func setupPeriodicUpdates() {
        // Update connection statistics every 5 seconds
        Timer.publish(every: 5.0, on: .main, in: .common)
            .autoconnect()
            .sink { [weak self] _ in
                self?.updateAllConnectionStatistics()
            }
            .store(in: &cancellables)
        
        // Check for stale connections every 30 seconds
        Timer.publish(every: 30.0, on: .main, in: .common)
            .autoconnect()
            .sink { [weak self] _ in
                self?.checkForStaleConnections()
            }
            .store(in: &cancellables)
    }
    
    private func updateAllConnectionStatistics() {
        for (worldId, connection) in activeConnections {
            // Update last activity if connection is active
            if connection.isActive {
                activeConnections[worldId]?.lastActivity = Date()
            }
        }
    }
    
    private func checkForStaleConnections() {
        let staleThreshold: TimeInterval = 300 // 5 minutes
        let now = Date()
        
        for (worldId, connection) in activeConnections {
            let timeSinceLastActivity = now.timeIntervalSince(connection.lastActivity)
            
            if timeSinceLastActivity > staleThreshold {
                logger.warning("Connection to \(connection.world.name) appears stale")
                updateConnectionStatus(worldId, status: .error)
            }
        }
    }
    
    // MARK: - Connection Recovery
    
    func attemptReconnection(_ worldId: UUID) async throws {
        guard let connection = activeConnections[worldId] else {
            throw ConnectionError.connectionNotFound
        }
        
        logger.info("Attempting to reconnect to \(connection.world.name)")
        
        updateConnectionStatus(worldId, status: .reconnecting)
        
        // Increment reconnect counter
        activeConnections[worldId]?.statistics.reconnectCount += 1
        
        // Simulate reconnection delay
        try await Task.sleep(nanoseconds: 2_000_000_000) // 2 seconds
        
        // In a real implementation, this would trigger actual reconnection logic
        // For now, we'll assume success
        updateConnectionStatus(worldId, status: .connected)
        
        logger.info("Successfully reconnected to \(connection.world.name)")
    }
    
    func disconnectAll() async {
        logger.info("Disconnecting from all worlds")
        
        let connectionIds = Array(activeConnections.keys)
        
        for worldId in connectionIds {
            updateConnectionStatus(worldId, status: .disconnected)
            removeConnection(worldId)
        }
        
        logger.info("Disconnected from all worlds")
    }
    
    // MARK: - Export and Analysis
    
    func exportConnectionData() -> ConnectionExportData {
        return ConnectionExportData(
            activeConnections: Array(activeConnections.values),
            connectionHistory: connectionHistory,
            overallStatistics: getOverallStatistics(),
            exportDate: Date()
        )
    }
    
    func generateConnectionReport() -> ConnectionReport {
        let now = Date()
        let last24Hours = now.addingTimeInterval(-24 * 60 * 60)
        let recentHistory = connectionHistory.filter { $0.connectedAt >= last24Hours }
        
        let successfulConnections = recentHistory.filter { $0.success }
        let failedConnections = recentHistory.filter { !$0.success }
        
        let totalDuration = recentHistory.reduce(0) { $0 + $1.duration }
        let averageDuration = recentHistory.isEmpty ? 0 : totalDuration / Double(recentHistory.count)
        
        let protocolStats = Dictionary(grouping: recentHistory) { $0.world.protocol }
            .mapValues { connections in
                ProtocolStatistics(
                    connectionCount: connections.count,
                    successRate: Double(connections.filter { $0.success }.count) / Double(connections.count),
                    averageDuration: connections.reduce(0) { $0 + $1.duration } / Double(connections.count)
                )
            }
        
        return ConnectionReport(
            reportDate: now,
            totalConnections: recentHistory.count,
            successfulConnections: successfulConnections.count,
            failedConnections: failedConnections.count,
            averageConnectionDuration: averageDuration,
            protocolStatistics: protocolStats,
            currentlyActive: activeConnections.count
        )
    }
}

// MARK: - Supporting Types

/// State of a world connection
struct ConnectionState {
    let world: WorldConfiguration
    let connectionId: UUID
    var status: ConnectionStatus
    let connectedAt: Date
    var lastActivity: Date = Date()
    var statistics: ConnectionStatistics = ConnectionStatistics()
    
    var duration: TimeInterval {
        return Date().timeIntervalSince(connectedAt)
    }
    
    var isActive: Bool {
        return status == .connected
    }
    
    var healthStatus: ConnectionHealth {
        let timeSinceActivity = Date().timeIntervalSince(lastActivity)
        
        switch status {
        case .disconnected:
            return .disconnected
        case .connecting, .reconnecting:
            return .connecting
        case .connected:
            if timeSinceActivity > 60 {
                return .poor
            } else if statistics.averageLatency > 0.3 {
                return .poor
            } else if statistics.averageLatency > 0.15 {
                return .fair
            } else if statistics.averageLatency > 0.05 {
                return .good
            } else {
                return .excellent
            }
        case .error:
            return .error
        }
    }
}

/// Connection status enumeration
enum ConnectionStatus: String, CaseIterable, Codable {
    case disconnected = "Disconnected"
    case connecting = "Connecting"
    case connected = "Connected"
    case reconnecting = "Reconnecting"
    case error = "Error"
    
    var color: String {
        switch self {
        case .disconnected: return "gray"
        case .connecting: return "yellow"
        case .connected: return "green"
        case .reconnecting: return "orange"
        case .error: return "red"
        }
    }
    
    var icon: String {
        switch self {
        case .disconnected: return "circle"
        case .connecting: return "circle.dashed"
        case .connected: return "circle.fill"
        case .reconnecting: return "arrow.clockwise.circle"
        case .error: return "exclamationmark.circle"
        }
    }
}

/// Connection health indicator
enum ConnectionHealth: String, CaseIterable {
    case disconnected = "Disconnected"
    case connecting = "Connecting"
    case poor = "Poor"
    case fair = "Fair"
    case good = "Good"
    case excellent = "Excellent"
    case error = "Error"
    
    var color: String {
        switch self {
        case .disconnected: return "gray"
        case .connecting: return "blue"
        case .poor: return "red"
        case .fair: return "orange"
        case .good: return "yellow"
        case .excellent: return "green"
        case .error: return "purple"
        }
    }
    
    var description: String {
        switch self {
        case .disconnected: return "Not connected"
        case .connecting: return "Establishing connection"
        case .poor: return "High latency or packet loss"
        case .fair: return "Moderate performance"
        case .good: return "Good connection quality"
        case .excellent: return "Optimal performance"
        case .error: return "Connection error"
        }
    }
}

/// Connection statistics tracking
struct ConnectionStatistics: Codable {
    var bytesReceived: UInt64 = 0
    var bytesSent: UInt64 = 0
    var packetsReceived: UInt32 = 0
    var packetsSent: UInt32 = 0
    var averageLatency: TimeInterval = 0
    var reconnectCount: Int = 0
    var lastError: String?
    var peakLatency: TimeInterval = 0
    var minLatency: TimeInterval = 0
    var packetLossRate: Double = 0
    
    var totalBytes: UInt64 {
        return bytesReceived + bytesSent
    }
    
    var totalPackets: UInt32 {
        return packetsReceived + packetsSent
    }
    
    var formattedDataTransfer: String {
        let formatter = ByteCountFormatter()
        formatter.allowedUnits = [.useKB, .useMB, .useGB]
        formatter.countStyle = .file
        return formatter.string(fromByteCount: Int64(totalBytes))
    }
    
    var formattedLatency: String {
        return String(format: "%.0f ms", averageLatency * 1000)
    }
}

/// Connection history entry
struct ConnectionHistoryEntry: Identifiable, Codable {
    let id = UUID()
    let world: WorldConfiguration
    let connectedAt: Date
    var disconnectedAt: Date?
    var duration: TimeInterval
    let success: Bool
    var status: ConnectionStatus = .connected
    let errorMessage: String?
    let statistics: ConnectionStatistics?
    
    init(world: WorldConfiguration,
         connectedAt: Date,
         duration: TimeInterval,
         success: Bool,
         errorMessage: String? = nil,
         statistics: ConnectionStatistics? = nil) {
        self.world = world
        self.connectedAt = connectedAt
        self.duration = duration
        self.success = success
        self.errorMessage = errorMessage
        self.statistics = statistics
    }
    
    var formattedDuration: String {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.hour, .minute, .second]
        formatter.unitsStyle = .abbreviated
        return formatter.string(from: duration) ?? "0s"
    }
    
    var formattedDate: String {
        let formatter = DateFormatter()
        formatter.dateStyle = .short
        formatter.timeStyle = .short
        return formatter.string(from: connectedAt)
    }
}

/// Overall connection statistics
struct OverallConnectionStatistics {
    let activeConnections: Int
    let totalBytesReceived: UInt64
    let totalBytesSent: UInt64
    let totalPacketsReceived: UInt32
    let totalPacketsSent: UInt32
    let averageLatency: TimeInterval
    let connectionUptime: TimeInterval
    
    var totalDataTransfer: UInt64 {
        return totalBytesReceived + totalBytesSent
    }
    
    var formattedDataTransfer: String {
        let formatter = ByteCountFormatter()
        formatter.allowedUnits = [.useKB, .useMB, .useGB]
        formatter.countStyle = .file
        return formatter.string(fromByteCount: Int64(totalDataTransfer))
    }
    
    var formattedUptime: String {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.day, .hour, .minute]
        formatter.unitsStyle = .full
        return formatter.string(from: connectionUptime) ?? "0 minutes"
    }
}

/// Protocol-specific statistics
struct ProtocolStatistics {
    let connectionCount: Int
    let successRate: Double
    let averageDuration: TimeInterval
    
    var formattedSuccessRate: String {
        return String(format: "%.1f%%", successRate * 100)
    }
    
    var formattedAverageDuration: String {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.hour, .minute, .second]
        formatter.unitsStyle = .abbreviated
        return formatter.string(from: averageDuration) ?? "0s"
    }
}

/// Connection export data structure
struct ConnectionExportData: Codable {
    let activeConnections: [ConnectionState]
    let connectionHistory: [ConnectionHistoryEntry]
    let overallStatistics: OverallConnectionStatistics
    let exportDate: Date
}

/// Connection report for analysis
struct ConnectionReport {
    let reportDate: Date
    let totalConnections: Int
    let successfulConnections: Int
    let failedConnections: Int
    let averageConnectionDuration: TimeInterval
    let protocolStatistics: [ProtocolType: ProtocolStatistics]
    let currentlyActive: Int
    
    var successRate: Double {
        guard totalConnections > 0 else { return 0 }
        return Double(successfulConnections) / Double(totalConnections)
    }
    
    var formattedSuccessRate: String {
        return String(format: "%.1f%%", successRate * 100)
    }
}

/// Connection errors
enum ConnectionError: LocalizedError {
    case connectionNotFound
    case connectionAlreadyExists
    case invalidConfiguration
    case networkError(String)
    case authenticationFailed
    case protocolError(String)
    case timeout
    
    var errorDescription: String? {
        switch self {
        case .connectionNotFound:
            return "Connection not found"
        case .connectionAlreadyExists:
            return "Connection already exists"
        case .invalidConfiguration:
            return "Invalid connection configuration"
        case .networkError(let message):
            return "Network error: \(message)"
        case .authenticationFailed:
            return "Authentication failed"
        case .protocolError(let message):
            return "Protocol error: \(message)"
        case .timeout:
            return "Connection timeout"
        }
    }
}

// MARK: - Notification Extensions

extension Notification.Name {
    static let worldConnectionAdded = Notification.Name("worldConnectionAdded")
    static let worldConnectionRemoved = Notification.Name("worldConnectionRemoved")
    static let worldConnectionStatusChanged = Notification.Name("worldConnectionStatusChanged")
    static let worldConnectionStatisticsUpdated = Notification.Name("worldConnectionStatisticsUpdated")
}

// MARK: - WorldDiscoveryService

class WorldDiscoveryService: ObservableObject {
    @Published var discoveredWorlds: [WorldConfiguration] = []
    @Published var favoriteWorlds: [WorldConfiguration] = []
    @Published var recentWorlds: [WorldConfiguration] = []
    @Published var isDiscovering = false
    
    private let logger = Logger(subsystem: "com.storm.client", category: "WorldDiscovery")
    private let userDefaults = UserDefaults.standard
    private let networkingClient = NetworkingClient.shared
    
    init() {
        loadSavedWorlds()
        loadDefaultWorlds()
    }
    
    // MARK: - Discovery
    
    func discoverWorlds() async {
        await MainActor.run {
            isDiscovering = true
        }
        
        logger.info("Starting world discovery")
        
        // Discover OpenSim grids
        await discoverOpenSimGrids()
        
        // Discover Finalverse servers
        await discoverFinalverseServers()
        
        // Discover local networks
        await discoverLocalWorlds()
        
        await MainActor.run {
            isDiscovering = false
        }
        
        logger.info("World discovery completed. Found \(discoveredWorlds.count) worlds")
    }
    
    private func discoverOpenSimGrids() async {
        // In a real implementation, this would query OpenSim grid lists
        let sampleGrids = [
            WorldConfiguration.opensim(name: "OSGrid", loginURI: "http://login.osgrid.org"),
            WorldConfiguration.opensim(name: "Metropolis Grid", loginURI: "http://hg.metropolis.de:8002"),
            WorldConfiguration.opensim(name: "Kitely", loginURI: "https://login.kitely.com"),
            WorldConfiguration.opensim(name: "InWorldz", loginURI: "http://inworldz.com:8002"),
            WorldConfiguration.opensim(name: "3rd Rock Grid", loginURI: "http://grid.3rdrockgrid.com:8002")
        ]
        
        await MainActor.run {
            discoveredWorlds.append(contentsOf: sampleGrids)
        }
    }
    
    private func discoverFinalverseServers() async {
        // In a real implementation, this would query Finalverse server directory
        let sampleServers = [
            WorldConfiguration.finalverse(name: "Finalverse Alpha", serverURL: "wss://alpha.finalverse.com"),
            WorldConfiguration.finalverse(name: "Finalverse Beta", serverURL: "wss://beta.finalverse.com"),
            WorldConfiguration.finalverse(name: "Finalverse Creative", serverURL: "wss://creative.finalverse.com")
        ]
        
        await MainActor.run {
            discoveredWorlds.append(contentsOf: sampleServers)
        }
    }
    
    private func discoverLocalWorlds() async {
        // Discover local Storm worlds
        let localWorlds = [
            WorldConfiguration.local(name: "Storm Sandbox"),
            WorldConfiguration.local(name: "Creative Playground"),
            WorldConfiguration.local(name: "AI Testing Lab")
        ]
        
        await MainActor.run {
            discoveredWorlds.append(contentsOf: localWorlds)
        }
    }
    
    // MARK: - Favorites Management
    
    func addToFavorites(_ world: WorldConfiguration) {
        if !favoriteWorlds.contains(where: { $0.id == world.id }) {
            favoriteWorlds.append(world)
            saveFavoriteWorlds()
            logger.info("Added \(world.name) to favorites")
        }
    }
    
    func removeFromFavorites(_ world: WorldConfiguration) {
        favoriteWorlds.removeAll { $0.id == world.id }
        saveFavoriteWorlds()
        logger.info("Removed \(world.name) from favorites")
    }
    
    func isFavorite(_ world: WorldConfiguration) -> Bool {
        return favoriteWorlds.contains { $0.id == world.id }
    }
    
    // MARK: - Recent Worlds
    
    func addToRecent(_ world: WorldConfiguration) {
        // Remove if already exists
        recentWorlds.removeAll { $0.id == world.id }
        
        // Add to beginning
        recentWorlds.insert(world, at: 0)
        
        // Keep only last 10
        if recentWorlds.count > 10 {
            recentWorlds = Array(recentWorlds.prefix(10))
        }
        
        saveRecentWorlds()
        logger.info("Added \(world.name) to recent worlds")
    }
    
    // MARK: - Search and Filter
    
    func searchWorlds(_ query: String) -> [WorldConfiguration] {
        guard !query.isEmpty else { return discoveredWorlds }
        
        let lowercaseQuery = query.lowercased()
        return discoveredWorlds.filter { world in
            world.name.lowercased().contains(lowercaseQuery) ||
            world.metadata.description.lowercased().contains(lowercaseQuery) ||
            world.metadata.tags.contains { $0.lowercased().contains(lowercaseQuery) }
        }
    }
    
    func filterWorlds(by protocol: ProtocolType) -> [WorldConfiguration] {
        return discoveredWorlds.filter { $0.protocol == protocol }
    }
    
    func filterWorlds(by feature: WorldFeature) -> [WorldConfiguration] {
        return discoveredWorlds.filter { $0.metadata.features.contains(feature) }
    }
    
    func filterWorlds(by maturity: MaturityRating) -> [WorldConfiguration] {
        return discoveredWorlds.filter { $0.metadata.maturityRating == maturity }
    }
    
    // MARK: - Persistence
    
    private func loadSavedWorlds() {
        loadFavoriteWorlds()
        loadRecentWorlds()
    }
    
    private func loadDefaultWorlds() {
        // Add some default worlds for immediate use
        let defaultWorlds = [
            WorldConfiguration.local(name: "Local Sandbox"),
            WorldConfiguration.opensim(name: "OSGrid", loginURI: "http://login.osgrid.org"),
            WorldConfiguration.finalverse(name: "Finalverse Demo", serverURL: "wss://demo.finalverse.com")
        ]
        
        discoveredWorlds.append(contentsOf: defaultWorlds)
    }
    
    private func loadFavoriteWorlds() {
        if let data = userDefaults.data(forKey: "favoriteWorlds"),
           let worlds = try? JSONDecoder().decode([WorldConfiguration].self, from: data) {
            favoriteWorlds = worlds
        }
    }
    
    private func saveFavoriteWorlds() {
        if let data = try? JSONEncoder().encode(favoriteWorlds) {
            userDefaults.set(data, forKey: "favoriteWorlds")
        }
    }
    
    private func loadRecentWorlds() {
        if let data = userDefaults.data(forKey: "recentWorlds"),
           let worlds = try? JSONDecoder().decode([WorldConfiguration].self, from: data) {
            recentWorlds = worlds
        }
    }
    
    private func saveRecentWorlds() {
        if let data = try? JSONEncoder().encode(recentWorlds) {
            userDefaults.set(data, forKey: "recentWorlds")
        }
    }
}
