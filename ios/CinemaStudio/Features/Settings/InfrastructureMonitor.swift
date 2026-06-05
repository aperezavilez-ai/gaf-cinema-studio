import Foundation

/// Infrastructure endpoints — scaffold; fetches live status when online.
enum InfrastructureConfig {
    static let vercelBaseURL = URL(string: "https://gaf-cinema-studio.vercel.app")!
    static let githubRepo = "aperezavilez-ai/gaf-cinema-studio"
    static let statusPath = "/api/status"
}

struct DeploymentConnections: Decodable {
    struct Service: Decodable {
        let status: String
        let repo: String?
        let region: String?
        let note: String?
    }

    let github: Service
    let vercel: Service
    let supabase: Service
}

struct DeploymentStatus: Decodable {
    let connections: DeploymentConnections
}

@MainActor
final class InfrastructureMonitor: ObservableObject {
    @Published var githubStatus = "linked"
    @Published var vercelStatus = "checking…"
    @Published var supabaseStatus = "pending"
    @Published var note = ""

    func refresh() async {
        githubStatus = "linked"
        let url = InfrastructureConfig.vercelBaseURL
            .appendingPathComponent("api/status")
        do {
            let (data, _) = try await URLSession.shared.data(from: url)
            let status = try JSONDecoder().decode(DeploymentStatus.self, from: data)
            vercelStatus = status.connections.vercel.status
            supabaseStatus = status.connections.supabase.status
            note = status.connections.supabase.note ?? ""
        } catch {
            vercelStatus = "offline"
            note = "Could not reach Vercel — check network"
        }
    }
}
