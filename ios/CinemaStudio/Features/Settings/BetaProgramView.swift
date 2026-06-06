import SwiftUI

/// Beta program status + ship project — Gate 12.
struct BetaProgramView: View {
    @EnvironmentObject var store: ProjectStore
    @State private var userLabel = ""
    @State private var statusMessage = ""
    @State private var completions = 0
    @State private var target = 10
    @State private var readyToShip = false

    var body: some View {
        ZStack {
            Color.black.ignoresSafeArea()

            ScrollView {
                VStack(alignment: .leading, spacing: 20) {
                    Text("BETA PROGRAM")
                        .font(.caption)
                        .tracking(3)
                        .foregroundStyle(.white.opacity(0.4))

                    Text("GAF Cinema Studio MVP 1.0.0")
                        .font(.title3)
                        .foregroundStyle(.white)

                    progressCard

                    VStack(alignment: .leading, spacing: 8) {
                        Text("Tester ID")
                            .font(.caption)
                            .foregroundStyle(.white.opacity(0.5))
                        TextField("beta_user_1", text: $userLabel)
                            .padding(12)
                            .background(Color.white.opacity(0.08))
                            .cornerRadius(6)
                            .foregroundStyle(.white)
                    }

                    Button("Mark project complete") {
                        store.shipProject(userLabel: userLabel.isEmpty ? "anonymous" : userLabel)
                        refreshStatus()
                        statusMessage = "Project marked complete"
                    }
                    .buttonStyle(CinematicButtonStyle())
                    .disabled(store.currentProject == nil)

                    if !statusMessage.isEmpty {
                        Text(statusMessage)
                            .font(.caption)
                            .foregroundStyle(.green.opacity(0.8))
                    }

                    if readyToShip {
                        Label("MVP ready to ship", systemImage: "checkmark.seal.fill")
                            .foregroundStyle(.green)
                    }
                }
                .padding(24)
            }
        }
        .navigationTitle("Beta Program")
        .navigationBarTitleDisplayMode(.inline)
        .onAppear { refreshStatus() }
    }

    private var progressCard: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("\(completions) / \(target) beta projects")
                .font(.headline)
                .foregroundStyle(.white)
            ProgressView(value: Double(completions), total: Double(target))
                .tint(.white)
            Text("Complete a full project: import → edit → export")
                .font(.caption)
                .foregroundStyle(.white.opacity(0.45))
        }
        .padding(16)
        .cinemaSurface()
    }

    private func refreshStatus() {
        let s = store.betaStatus()
        completions = s.completions
        target = s.target
        readyToShip = s.readyToShip
    }
}

#Preview {
    NavigationStack {
        BetaProgramView().environmentObject(ProjectStore())
    }
}
