import SwiftUI

/// CinemaStudio iOS shell — Phase 1 scaffold.
/// Requires UniFFI-generated Swift bindings from `cinemastudio-engine` (feature `ffi`).
@main
struct CinemaStudioApp: App {
    @StateObject private var projectStore = ProjectStore()

    var body: some Scene {
        WindowGroup {
            NavigationStack {
                HomeView()
            }
            .environmentObject(projectStore)
            .preferredColorScheme(.dark)
        }
    }
}
