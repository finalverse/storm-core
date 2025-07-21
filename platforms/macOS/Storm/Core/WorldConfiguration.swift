// File: platforms/macOS/Storm/Core/WorldConfiguration.swift
// Description: World configuration and connection management for Storm client
// Handles OpenSim, Finalverse, and other virtual world protocols

import Foundation
import os

/// Configuration for connecting to virtual worlds
struct WorldConfiguration: Codable, Identifiable, Hashable {
    let id = UUID()
    let name: String
    let url: String
    let protocol: ProtocolType
    let credentials: Credentials?
    let settings: ConnectionSettings
    let metadata: WorldMetadata
    
    init(name: String, url: String, protocol: ProtocolType, credentials: Credentials? = nil, settings: ConnectionSettings = ConnectionSettings(), metadata: WorldMetadata = WorldMetadata()) {
        self.name = name
        self.url = url
        self.protocol = protocol
        self.credentials = credentials
        self.settings = settings
        self.metadata = metadata
    }
    
    // MARK: - Convenience Initializers
    
    /// Create OpenSim world configuration
    static func opensim(name: String, loginURI: String, username: String? = nil, password: String? = nil) -> WorldConfiguration {
        let credentials = username.flatMap { user in
            password.map { pass in
                Credentials(username: user, password: pass, sessionToken: nil, userAgent: "Storm/1.0")
            }
        }
        
        let settings = ConnectionSettings(
            timeout: 30,
            autoReconnect: true,
            maxReconnectAttempts: 3,
            useCompression: true,
            enableEncryption: false,
            protocolVersion: "LLUDP/1.0"
        )
        
        let metadata = WorldMetadata(
            description: "OpenSim Virtual World",
            maxAvatars: 100,
            gridType: .opensim,
            features: [.physics, .scripting, .voice, .media],
            region: "Main"
        )
        
        return WorldConfiguration(
            name: name,
            url: loginURI,
            protocol: .opensim,
            credentials: credentials,
            settings: settings,
            metadata: metadata
        )
    }
    
    /// Create Finalverse world configuration
    static func finalverse(name: String, serverURL: String, apiKey: String? = nil) -> WorldConfiguration {
        let credentials = apiKey.map { key in
            Credentials(username: "", password: "", sessionToken: key, userAgent: "Storm/1.0")
        }
        
        let settings = ConnectionSettings(
            timeout: 15,
            autoReconnect: true,
            maxReconnectAttempts: 5,
            useCompression: true,
            enableEncryption: true,
            protocolVersion: "WebSocket/1.0"
        )
        
        let metadata = WorldMetadata(
            description: "Finalverse AI-Enhanced World",
            maxAvatars: 1000,
            gridType: .finalverse,
            features: [.ai, .narratives, .physics, .procedural, .social],
            region: "Nexus"
        )
        
        return WorldConfiguration(
            name: name,
            url: serverURL,
            protocol: .finalverse,
            credentials: credentials,
            settings: settings,
            metadata: metadata
        )
    }
    
    /// Create local world configuration
    static func local(name: String) -> WorldConfiguration {
        let settings = ConnectionSettings(
            timeout: 1,
            autoReconnect: false,
            maxReconnectAttempts: 0,
            useCompression: false,
            enableEncryption: false,
            protocolVersion: "Local/1.0"
        )
        
        let metadata = WorldMetadata(
            description: "Local Storm World",
            maxAvatars: 10,
            gridType: .local,
            features: [.physics, .ai, .procedural],
            region: "Sandbox"
        )
        
        return WorldConfiguration(
            name: name,
            url: "local://sandbox",
            protocol: .local,
            credentials: nil,
            settings: settings,
            metadata: metadata
        )
    }
    
    // MARK: - Validation
    
    var isValid: Bool {
        return !name.isEmpty &&
               !url.isEmpty &&
               isValidURL &&
               hasRequiredCredentials
    }
    
    private var isValidURL: Bool {
        switch protocol {
        case .opensim:
            return url.hasPrefix("http://") || url.hasPrefix("https://")
        case .finalverse:
            return url.hasPrefix("ws://") || url.hasPrefix("wss://") || url.hasPrefix("https://")
        case .local:
            return url.hasPrefix("local://")
        }
    }
    
    private var hasRequiredCredentials: Bool {
        switch protocol {
        case .opensim:
            return credentials?.username.isEmpty == false &&
                   credentials?.password.isEmpty == false
        case .finalverse:
            return credentials?.sessionToken?.isEmpty == false ||
                   metadata.features.contains(.publicAccess)
        case .local:
            return true // No credentials required for local
        }
    }
    
    // MARK: - Connection Info
    
    var displayInfo: String {
        switch protocol {
        case .opensim:
            return "OpenSim Grid: \(name)"
        case .finalverse:
            return "Finalverse: \(name)"
        case .local:
            return "Local World: \(name)"
        }
    }
    
    var statusDescription: String {
        return isValid ? "Ready to connect" : "Configuration incomplete"
    }
}

/// Authentication credentials for world connections
struct Credentials: Codable, Hashable {
    let username: String
    let password: String
    let sessionToken: String?
    let userAgent: String?
    
    // Security: Don't include password in description
    var safeDescription: String {
        return "User: \(username), Has Token: \(sessionToken != nil)"
    }
}

/// Connection-specific settings and preferences
struct ConnectionSettings: Codable, Hashable {
    let timeout: TimeInterval
    let autoReconnect: Bool
    let maxReconnectAttempts: Int
    let useCompression: Bool
    let enableEncryption: Bool
    let protocolVersion: String
    let customHeaders: [String: String]
    let localPort: Int?
    
    init(timeout: TimeInterval = 30,
         autoReconnect: Bool = true,
         maxReconnectAttempts: Int = 3,
         useCompression: Bool = true,
         enableEncryption: Bool = true,
         protocolVersion: String = "1.0",
         customHeaders: [String: String] = [:],
         localPort: Int? = nil) {
        self.timeout = timeout
        self.autoReconnect = autoReconnect
        self.maxReconnectAttempts = maxReconnectAttempts
        self.useCompression = useCompression
        self.enableEncryption = enableEncryption
        self.protocolVersion = protocolVersion
        self.customHeaders = customHeaders
        self.localPort = localPort
    }
}

/// Metadata about the virtual world
struct WorldMetadata: Codable, Hashable {
    let description: String
    let maxAvatars: Int
    let gridType: GridType
    let features: Set<WorldFeature>
    let region: String
    let version: String
    let lastUpdated: Date
    let tags: [String]
    let maturityRating: MaturityRating
    
    init(description: String = "",
         maxAvatars: Int = 100,
         gridType: GridType = .opensim,
         features: Set<WorldFeature> = [],
         region: String = "Main",
         version: String = "1.0",
         lastUpdated: Date = Date(),
         tags: [String] = [],
         maturityRating: MaturityRating = .general) {
        self.description = description
        self.maxAvatars = maxAvatars
        self.gridType = gridType
        self.features = features
        self.region = region
        self.version = version
        self.lastUpdated = lastUpdated
        self.tags = tags
        self.maturityRating = maturityRating
    }
}

// MARK: - Enums

/// Supported virtual world protocols
enum ProtocolType: String, CaseIterable, Codable {
    case opensim = "OpenSim"
    case finalverse = "Finalverse"
    case local = "Local"
    
    var displayName: String {
        return rawValue
    }
    
    var defaultPort: Int {
        switch self {
        case .opensim: return 9000
        case .finalverse: return 443
        case .local: return 0
        }
    }
    
    var icon: String {
        switch self {
        case .opensim: return "globe"
        case .finalverse: return "brain.head.profile"
        case .local: return "house"
        }
    }
    
    var requiresCredentials: Bool {
        switch self {
        case .opensim: return true
        case .finalverse: return false // Can have public access
        case .local: return false
        }
    }
}

/// Type of virtual world grid
enum GridType: String, CaseIterable, Codable {
    case opensim = "OpenSim"
    case secondlife = "SecondLife"
    case finalverse = "Finalverse"
    case local = "Local"
    case custom = "Custom"
    
    var supportedFeatures: Set<WorldFeature> {
        switch self {
        case .opensim:
            return [.physics, .scripting, .voice, .media, .teleport]
        case .secondlife:
            return [.physics, .scripting, .voice, .media, .teleport, .economy]
        case .finalverse:
            return [.ai, .narratives, .physics, .procedural, .social, .economy]
        case .local:
            return [.physics, .ai, .procedural]
        case .custom:
            return [] // Will be determined dynamically
        }
    }
}

/// Features available in virtual worlds
enum WorldFeature: String, CaseIterable, Codable, Hashable {
    case physics = "Physics"
    case scripting = "Scripting"
    case voice = "Voice"
    case media = "Media"
    case teleport = "Teleport"
    case economy = "Economy"
    case ai = "AI"
    case narratives = "Narratives"
    case procedural = "Procedural"
    case social = "Social"
    case publicAccess = "Public Access"
    case customization = "Customization"
    case building = "Building"
    case events = "Events"
    
    var icon: String {
        switch self {
        case .physics: return "cube.box"
        case .scripting: return "chevron.left.forwardslash.chevron.right"
        case .voice: return "mic"
        case .media: return "play.rectangle"
        case .teleport: return "location"
        case .economy: return "dollarsign.circle"
        case .ai: return "brain"
        case .narratives: return "book"
        case .procedural: return "wand.and.rays"
        case .social: return "person.2"
        case .publicAccess: return "globe"
        case .customization: return "slider.horizontal.3"
        case .building: return "hammer"
        case .events: return "calendar"
        }
    }
}

/// Content maturity rating
enum MaturityRating: String, CaseIterable, Codable {
    case general = "General"
    case mature = "Mature"
    case adult = "Adult"
    
    var description: String {
        switch self {
        case .general: return "Suitable for all ages"
        case .mature: return "May contain mature content"
        case .adult: return "Adult content - 18+ only"
        }
    }
    
    var color: String {
        switch self {
        case .general: return "green"
        case .mature: return "yellow"
        case .adult: return "red"
        }
    }
}

// MARK: - World Discovery and Management

/// World discovery service for finding available worlds
class WorldDiscoveryService: ObservableObject {
    @Published var discoveredWorlds: [WorldConfiguration] = []
    @Published var favoriteWorlds: [WorldConfiguration] = []
    @Published var recentWorlds: [WorldConfiguration] = []
    @Published var isDiscovering = false
    
    private let logger = Logger(subsystem: "com.storm.client", category: "WorldDiscovery")
    private let userDefaults = UserDefaults.standard
    
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
            WorldConfiguration.opensim(name: "Kitely", loginURI: "https://login.kitely.com")
        ]
        
        await MainActor.run {
            discoveredWorlds.append(contentsOf: sampleGrids)
        }
    }
    
    private func discoverFinalverseServers() async {
        // In a real implementation, this would query Finalverse server directory
        let sampleServers = [
            WorldConfiguration.finalverse(name: "Finalverse Alpha", serverURL: "wss://alpha.finalverse.com"),
            WorldConfiguration.finalverse(name: "Finalverse Beta", serverURL: "wss://beta.finalverse.com")
        ]
        
        await MainActor.run {
            discoveredWorlds.append(contentsOf: sampleServers)
        }
    }
    
    private func discoverLocalWorlds() async {
        // Discover local Storm worlds
        let localWorld = WorldConfiguration.local(name: "Storm Sandbox")
        
        await MainActor.run {
            discoveredWorlds.append(localWorld)
        }
    }
    
    // MARK: - Favorites Management
    
    func addToFavorites(_ world: WorldConfiguration) {
        if !favoriteWorlds.contains(world) {
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
}

// MARK: - Connection State Management

/// Manages the state of world connections
class WorldConnectionManager: ObservableObject {
    @Published var activeConnections: [UUID: ConnectionState] = [:]
    @Published var connectionHistory: [ConnectionHistoryEntry] = []
    
    private let logger = Logger(subsystem: "com.storm.client", category: "ConnectionManager")
    
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
        
        logger.info("Added connection to \(world.name)")
    }
    
    func updateConnectionStatus(_ worldId: UUID, status: ConnectionStatus) {
        activeConnections[worldId]?.status = status
        logger.info("Updated connection status for world \(worldId): \(status)")
    }
    
    func removeConnection(_ worldId: UUID) {
        if let connection = activeConnections[worldId] {
            // Update history with final duration
            if let historyIndex = connectionHistory.firstIndex(where: { $0.world.id == worldId }) {
                let duration = Date().timeIntervalSince(connection.connectedAt)
                connectionHistory[historyIndex].duration = duration
            }
            
            activeConnections.removeValue(forKey: worldId)
            logger.info("Removed connection for world \(worldId)")
        }
    }
    
    var hasActiveConnections: Bool {
        return !activeConnections.isEmpty
    }
    
    func getConnection(for worldId: UUID) -> ConnectionState? {
        return activeConnections[worldId]
    }
}

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
}

/// Connection status
enum ConnectionStatus: String, CaseIterable {
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

/// Connection statistics
struct ConnectionStatistics {
    var bytesReceived: UInt64 = 0
    var bytesSent: UInt64 = 0
    var packetsReceived: UInt32 = 0
    var packetsSent: UInt32 = 0
    var averageLatency: TimeInterval = 0
    var reconnectCount: Int = 0
    var lastError: String?
    
    var totalBytes: UInt64 {
        return bytesReceived + bytesSent
    }
    
    var totalPackets: UInt32 {
        return packetsReceived + packetsSent
    }
}

/// Connection history entry
struct ConnectionHistoryEntry: Identifiable {
    let id = UUID()
    let world: WorldConfiguration
    let connectedAt: Date
    var duration: TimeInterval
    let success: Bool
    let errorMessage: String?
    
    init(world: WorldConfiguration, connectedAt: Date, duration: TimeInterval, success: Bool, errorMessage: String? = nil) {
        self.world = world
        self.connectedAt = connectedAt
        self.duration = duration
        self.success = success
        self.errorMessage = errorMessage
    }
    
    var formattedDuration: String {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.hour, .minute, .second]
        formatter.unitsStyle = .abbreviated
        return formatter.string(from: duration) ?? "0s"
    }
}
