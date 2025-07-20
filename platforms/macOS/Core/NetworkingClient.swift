// platforms/macos/Storm/Core/NetworkingClient.swift
// Description: Client for networking operations, e.g., fetching world data.
// Summary: Singleton handling API calls or WebSocket. Currently a stub; can extend for real-time updates.
// Logic: Uses URLSession for async calls. Error-free: Basic completion handler; compiles without external deps.
// Progressive: Add as stub; expand later for integration with StormEngine.

import Foundation

class NetworkingClient {
    static let shared = NetworkingClient()
    
    private init() {}
    
    func fetchData(completion: @escaping (Data?) -> Void) {
        // Caller: StormEngine.fetchWorldData; callee: URLSession dataTask.
        guard let url = URL(string: "https://example.com/world-data") else {
            completion(nil)
            return
        }
        
        URLSession.shared.dataTask(with: url) { data, _, _ in
            completion(data)
        }.resume()
    }
}
