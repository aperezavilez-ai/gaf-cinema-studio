import Foundation

/// Infrastructure endpoints — scaffold; fetches live status when online.
enum InfrastructureConfig {
    static let customBaseURL = URL(string: "https://gafcinemastudio.com")!
    static let customWwwURL = URL(string: "https://www.gafcinemastudio.com")!
    static let vercelFallbackURL = URL(string: "https://gaf-cinema-studio.vercel.app")!
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
        if await fetchStatus(from: InfrastructureConfig.customBaseURL) { return }
        if await fetchStatus(from: InfrastructureConfig.customWwwURL) {
            note = "Using www — configure apex DNS for gafcinemastudio.com"
            return
        }
        if await fetchStatus(from: InfrastructureConfig.vercelFallbackURL) {
            vercelStatus = "deployed (fallback)"
            note = "Custom domain unreachable — using vercel.app"
            return
        }
        vercelStatus = "offline"
        note = "Could not reach Vercel — check network"
    }

    private func fetchStatus(from base: URL) async -> Bool {
        let url = base.appendingPathComponent("api/status")
        do {
            let (data, _) = try await URLSession.shared.data(from: url)
            let status = try JSONDecoder().decode(DeploymentStatus.self, from: data)
            vercelStatus = status.connections.vercel.status
            supabaseStatus = status.connections.supabase.status
            if note.isEmpty {
                note = status.connections.supabase.note ?? ""
            }
            return true
        } catch {
            return false
        }
    }
}
