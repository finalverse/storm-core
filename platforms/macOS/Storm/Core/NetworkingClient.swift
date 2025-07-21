// File: Core/NetworkingClient.swift
// Description: Network services for Storm client
// Handles HTTP requests, WebSocket connections, and network monitoring

import Foundation
import Network
import Combine
import os

class NetworkingClient: ObservableObject {
    static let shared = NetworkingClient()
    
    @Published var isConnected: Bool = false
    @Published var connectionType: NWInterface.InterfaceType?
    @Published var latency: TimeInterval = 0
    
    private let monitor = NWPathMonitor()
    private let monitorQueue = DispatchQueue(label: "NetworkMonitor")
    private let logger = Logger(subsystem: "com.storm.client", category: "Networking")
    
    private var urlSession: URLSession
    private var webSocketTasks: [String: URLSessionWebSocketTask] = [:]
    private var connectionHandlers: [String: (Bool) -> Void] = [:]
    
    private init() {
        // Configure URL session with custom configuration
        let config = URLSessionConfiguration.default
        config.timeoutIntervalForRequest = 30
        config.timeoutIntervalForResource = 60
        config.waitsForConnectivity = true
        config.requestCachePolicy = .reloadIgnoringLocalCacheData
        
        self.urlSession = URLSession(configuration: config)
        
        setupNetworkMonitoring()
    }
    
    deinit {
        monitor.cancel()
        closeAllWebSockets()
    }
    
    // MARK: - Network Monitoring
    
    private func setupNetworkMonitoring() {
        monitor.pathUpdateHandler = { [weak self] path in
            DispatchQueue.main.async {
                self?.isConnected = path.status == .satisfied
                self?.connectionType = path.availableInterfaces.first?.type
                
                if path.status == .satisfied {
                    self?.logger.info("Network connection established")
                    self?.measureLatency()
                } else {
                    self?.logger.warning("Network connection lost")
                }
            }
        }
        
        monitor.start(queue: monitorQueue)
    }
    
    private func measureLatency() {
        Task {
            let startTime = CACurrentMediaTime()
            
            do {
                let url = URL(string: "https://www.google.com")!
                var request = URLRequest(url: url)
                request.httpMethod = "HEAD"
                request.timeoutInterval = 5
                
                let _ = try await urlSession.data(for: request)
                
                await MainActor.run {
                    self.latency = CACurrentMediaTime() - startTime
                }
            } catch {
                await MainActor.run {
                    self.latency = 0
                }
            }
        }
    }
    
    // MARK: - HTTP Requests
    
    func fetchData(from url: URL) async throws -> Data {
        logger.info("Fetching data from: \(url)")
        
        let (data, response) = try await urlSession.data(from: url)
        
        guard let httpResponse = response as? HTTPURLResponse else {
            throw NetworkError.invalidResponse
        }
        
        guard 200...299 ~= httpResponse.statusCode else {
            throw NetworkError.httpError(httpResponse.statusCode)
        }
        
        logger.info("Successfully fetched \(data.count) bytes from \(url)")
        return data
    }
    
    func fetchData(completion: @escaping (Data?) -> Void) {
        guard let url = URL(string: "https://example.com/world-data") else {
            completion(nil)
            return
        }
        
        URLSession.shared.dataTask(with: url) { data, response, error in
            DispatchQueue.main.async {
                if let error = error {
                    self.logger.error("Failed to fetch data: \(error)")
                    completion(nil)
                } else {
                    completion(data)
                }
            }
        }.resume()
    }
    
    func post<T: Codable>(
        to url: URL,
        body: T,
        headers: [String: String] = [:]
    ) async throws -> Data {
        logger.info("POSTing data to: \(url)")
        
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        
        // Add custom headers
        for (key, value) in headers {
            request.setValue(value, forHTTPHeaderField: key)
        }
        
        // Encode body
        request.httpBody = try JSONEncoder().encode(body)
        
        let (data, response) = try await urlSession.data(for: request)
        
        guard let httpResponse = response as? HTTPURLResponse else {
            throw NetworkError.invalidResponse
        }
        
        guard 200...299 ~= httpResponse.statusCode else {
            throw NetworkError.httpError(httpResponse.statusCode)
        }
        
        return data
    }
    
    func testConnection(completion: @escaping (Bool) -> Void) {
        guard let url = URL(string: "https://httpbin.org/status/200") else {
            completion(false)
            return
        }
        
        var request = URLRequest(url: url)
        request.timeoutInterval = 5
        
        URLSession.shared.dataTask(with: request) { _, response, error in
            DispatchQueue.main.async {
                if let httpResponse = response as? HTTPURLResponse {
                    completion(httpResponse.statusCode == 200)
                } else {
                    completion(false)
                }
            }
        }.resume()
    }
    
    // MARK: - WebSocket Connections
    
    func connectWebSocket(
        to url: URL,
        connectionId: String,
        onConnect: @escaping (Bool) -> Void = { _ in },
        onMessage: @escaping (Data) -> Void = { _ in },
        onDisconnect: @escaping (URLSessionWebSocketTask.CloseCode?, Data?) -> Void = { _, _ in }
    ) {
        logger.info("Connecting WebSocket to: \(url)")
        
        // Close existing connection if any
        closeWebSocket(connectionId: connectionId)
        
        let task = urlSession.webSocketTask(with: url)
        webSocketTasks[connectionId] = task
        connectionHandlers[connectionId] = onConnect
        
        // Start receiving messages
        receiveWebSocketMessage(task: task, onMessage: onMessage, onDisconnect: onDisconnect)
        
        // Start the connection
        task.resume()
        
        // Check connection status after a delay
        DispatchQueue.main.asyncAfter(deadline: .now() + 2) { [weak self] in
            if task.state == .running {
                onConnect(true)
                self?.logger.info("WebSocket connected successfully")
            } else {
                onConnect(false)
                self?.logger.error("WebSocket connection failed")
            }
        }
    }
    
    private func receiveWebSocketMessage(
        task: URLSessionWebSocketTask,
        onMessage: @escaping (Data) -> Void,
        onDisconnect: @escaping (URLSessionWebSocketTask.CloseCode?, Data?) -> Void
    ) {
        task.receive { [weak self] result in
            switch result {
            case .success(let message):
                switch message {
                case .data(let data):
                    onMessage(data)
                case .string(let text):
                    if let data = text.data(using: .utf8) {
                        onMessage(data)
                    }
                @unknown default:
                    break
                }
                
                // Continue receiving
                self?.receiveWebSocketMessage(task: task, onMessage: onMessage, onDisconnect: onDisconnect)
                
            case .failure(let error):
                self?.logger.error("WebSocket receive error: \(error)")
                onDisconnect(nil, nil)
            }
        }
    }
    
    func sendWebSocketMessage(
        connectionId: String,
        message: Data
    ) async throws {
        guard let task = webSocketTasks[connectionId] else {
            throw NetworkError.connectionNotFound
        }
        
        try await task.send(.data(message))
    }
    
    func sendWebSocketMessage(
        connectionId: String,
        message: String
    ) async throws {
        guard let task = webSocketTasks[connectionId] else {
            throw NetworkError.connectionNotFound
        }
        
        try await task.send(.string(message))
    }
    
    func closeWebSocket(connectionId: String) {
        if let task = webSocketTasks[connectionId] {
            task.cancel(with: .normalClosure, reason: nil)
            webSocketTasks.removeValue(forKey: connectionId)
            connectionHandlers.removeValue(forKey: connectionId)
            logger.info("Closed WebSocket connection: \(connectionId)")
        }
    }
    
    func closeAllWebSockets() {
        for (connectionId, task) in webSocketTasks {
            task.cancel(with: .normalClosure, reason: nil)
            logger.info("Closed WebSocket connection: \(connectionId)")
        }
        webSocketTasks.removeAll()
        connectionHandlers.removeAll()
    }
    
    // MARK: - OpenSim Specific Methods
    
    func testOpenSimConnection(
        loginURI: String,
        username: String,
        password: String
    ) async throws -> OpenSimLoginResponse {
        guard let url = URL(string: loginURI) else {
            throw NetworkError.invalidURL
        }
        
        logger.info("Testing OpenSim connection to: \(loginURI)")
        
        let loginRequest = OpenSimLoginRequest(
            first: username.components(separatedBy: " ").first ?? "",
            last: username.components(separatedBy: " ").last ?? "",
            passwd: password,
            start: "last",
            channel: "Storm 1.0",
            version: "1.0.0",
            platform: "Mac",
            mac: "00:00:00:00:00:00"
        )
        
        let responseData = try await post(to: url, body: loginRequest)
        let response = try JSONDecoder().decode(OpenSimLoginResponse.self, from: responseData)
        
        if response.login == "true" {
            logger.info("OpenSim login successful")
        } else {
            logger.error("OpenSim login failed: \(response.message ?? "Unknown error")")
            throw NetworkError.authenticationFailed
        }
        
        return response
    }
    
    // MARK: - Finalverse Specific Methods
    
    func connectToFinalverse(
        serverURL: String,
        apiKey: String? = nil
    ) async throws -> FinalverseConnectionInfo {
        guard let url = URL(string: serverURL) else {
            throw NetworkError.invalidURL
        }
        
        logger.info("Connecting to Finalverse server: \(serverURL)")
        
        var headers: [String: String] = [:]
        if let apiKey = apiKey {
            headers["Authorization"] = "Bearer \(apiKey)"
        }
        
        let connectionRequest = FinalverseConnectionRequest(
            clientVersion: "Storm/1.0",
            supportedFeatures: ["ai", "narratives", "physics", "procedural"]
        )
        
        let responseData = try await post(to: url, body: connectionRequest, headers: headers)
        let response = try JSONDecoder().decode(FinalverseConnectionInfo.self, from: responseData)
        
        logger.info("Finalverse connection established")
        return response
    }
    
    // MARK: - Network Diagnostics
    
    func runNetworkDiagnostics() async -> NetworkDiagnostics {
        var diagnostics = NetworkDiagnostics()
        
        // Test basic connectivity
        diagnostics.isConnected = isConnected
        diagnostics.connectionType = connectionType?.displayName ?? "Unknown"
        
        // Measure latency to common servers
        diagnostics.googleLatency = await measureLatencyTo("https://www.google.com")
        diagnostics.cloudflareLatency = await measureLatencyTo("https://1.1.1.1")
        
        // Test specific service connectivity
        diagnostics.canReachOpenSim = await testServiceReachability("http://login.osgrid.org")
        diagnostics.canReachFinalverse = await testServiceReachability("https://api.finalverse.com")
        
        return diagnostics
    }
    
    private func measureLatencyTo(_ urlString: String) async -> TimeInterval {
        guard let url = URL(string: urlString) else { return 0 }
        
        let startTime = CACurrentMediaTime()
        
        do {
            var request = URLRequest(url: url)
            request.httpMethod = "HEAD"
            request.timeoutInterval = 5
            
            let _ = try await urlSession.data(for: request)
            return CACurrentMediaTime() - startTime
        } catch {
            return 0
        }
    }
    
    private func testServiceReachability(_ urlString: String) async -> Bool {
        guard let url = URL(string: urlString) else { return false }
        
        do {
            var request = URLRequest(url: url)
            request.httpMethod = "HEAD"
            request.timeoutInterval = 10
            
            let (_, response) = try await urlSession.data(for: request)
            
            if let httpResponse = response as? HTTPURLResponse {
                return httpResponse.statusCode < 500
            }
            
            return false
        } catch {
            return false
        }
    }
}

// MARK: - Supporting Types

enum NetworkError: LocalizedError {
    case invalidURL
    case invalidResponse
    case httpError(Int)
    case connectionNotFound
    case authenticationFailed
    case timeout
    case noConnection
    
    var errorDescription: String? {
        switch self {
        case .invalidURL:
            return "Invalid URL"
        case .invalidResponse:
            return "Invalid response from server"
        case .httpError(let code):
            return "HTTP error: \(code)"
        case .connectionNotFound:
            return "Connection not found"
        case .authenticationFailed:
            return "Authentication failed"
        case .timeout:
            return "Request timed out"
        case .noConnection:
            return "No network connection"
        }
    }
}

// MARK: - OpenSim Types

struct OpenSimLoginRequest: Codable {
    let first: String
    let last: String
    let passwd: String
    let start: String
    let channel: String
    let version: String
    let platform: String
    let mac: String
}

struct OpenSimLoginResponse: Codable {
    let login: String
    let message: String?
    let sessionId: String?
    let secureSessionId: String?
    let agentId: String?
    let circuitCode: Int?
    let regionX: Int?
    let regionY: Int?
    let simPort: Int?
    let simIP: String?
    let seedCapability: String?
    
    enum CodingKeys: String, CodingKey {
        case login
        case message
        case sessionId = "session_id"
        case secureSessionId = "secure_session_id"
        case agentId = "agent_id"
        case circuitCode = "circuit_code"
        case regionX = "region_x"
        case regionY = "region_y"
        case simPort = "sim_port"
        case simIP = "sim_ip"
        case seedCapability = "seed_capability"
    }
}

// MARK: - Finalverse Types

struct FinalverseConnectionRequest: Codable {
    let clientVersion: String
    let supportedFeatures: [String]
}

struct FinalverseConnectionInfo: Codable {
    let connectionId: String
    let serverVersion: String
    let availableFeatures: [String]
    let worldList: [FinalverseWorld]
    let userProfile: FinalverseUserProfile?
}

struct FinalverseWorld: Codable {
    let id: String
    let name: String
    let description: String
    let maxUsers: Int
    let currentUsers: Int
    let features: [String]
}

struct FinalverseUserProfile: Codable {
    let userId: String
    let username: String
    let avatarId: String?
    let preferences: [String: String]
}

// MARK: - Diagnostics

struct NetworkDiagnostics {
    var isConnected: Bool = false
    var connectionType: String = "Unknown"
    var googleLatency: TimeInterval = 0
    var cloudflareLatency: TimeInterval = 0
    var canReachOpenSim: Bool = false
    var canReachFinalverse: Bool = false
    
    var averageLatency: TimeInterval {
        let latencies = [googleLatency, cloudflareLatency].filter { $0 > 0 }
        return latencies.isEmpty ? 0 : latencies.reduce(0, +) / Double(latencies.count)
    }
    
    var connectionQuality: ConnectionQuality {
        guard isConnected else { return .none }
        
        let avgLatency = averageLatency
        if avgLatency == 0 { return .unknown }
        if avgLatency < 0.05 { return .excellent }
        if avgLatency < 0.1 { return .good }
        if avgLatency < 0.2 { return .fair }
        return .poor
    }
}

enum ConnectionQuality: String, CaseIterable {
    case none = "No Connection"
    case poor = "Poor"
    case fair = "Fair"
    case good = "Good"
    case excellent = "Excellent"
    case unknown = "Unknown"
    
    var color: String {
        switch self {
        case .none: return "gray"
        case .poor: return "red"
        case .fair: return "orange"
        case .good: return "yellow"
        case .excellent: return "green"
        case .unknown: return "blue"
        }
    }
}

// MARK: - Extensions

extension NWInterface.InterfaceType {
    var displayName: String {
        switch self {
        case .wifi: return "Wi-Fi"
        case .cellular: return "Cellular"
        case .wiredEthernet: return "Ethernet"
        case .loopback: return "Loopback"
        case .other: return "Other"
        @unknown default: return "Unknown"
        }
    }
}

// MARK: - Notification Extensions

extension Notification.Name {
    static let networkStatusChanged = Notification.Name("networkStatusChanged")
    static let webSocketConnected = Notification.Name("webSocketConnected")
    static let webSocketDisconnected = Notification.Name("webSocketDisconnected")
    static let webSocketMessageReceived = Notification.Name("webSocketMessageReceived")
}
