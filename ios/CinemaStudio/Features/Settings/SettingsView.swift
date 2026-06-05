import SwiftUI

/// Account, cloud backup, billing, and telemetry — all optional (local-first).
struct SettingsView: View {
    @EnvironmentObject var store: ProjectStore
    @State private var loggedIn = false
    @State private var email = ""
    @State private var password = ""
    @State private var telemetryEnabled = false
    @State private var crashReporting = false
    @State private var isPro = false
    @State private var statusMessage = ""

    var body: some View {
        ZStack {
            Color.black.ignoresSafeArea()

            ScrollView {
                VStack(alignment: .leading, spacing: 24) {
                    Text("SETTINGS")
                        .font(.caption)
                        .tracking(3)
                        .foregroundStyle(.white.opacity(0.4))

                    accountSection
                    cloudSection
                    billingSection
                    privacySection

                    if !statusMessage.isEmpty {
                        Text(statusMessage)
                            .font(.caption)
                            .foregroundStyle(.green.opacity(0.8))
                    }
                }
                .padding(24)
            }
        }
        .navigationTitle("Settings")
        .navigationBarTitleDisplayMode(.inline)
    }

    private var accountSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            sectionHeader("Account", subtitle: "Optional — core works without login")

            if loggedIn {
                HStack {
                    Image(systemName: "person.crop.circle.fill")
                        .foregroundStyle(.white.opacity(0.6))
                    Text(email)
                        .foregroundStyle(.white)
                    Spacer()
                    Button("Sign out") {
                        loggedIn = false
                        statusMessage = "Signed out"
                    }
                    .font(.caption)
                    .foregroundStyle(.white.opacity(0.7))
                }
            } else {
                TextField("Email", text: $email)
                    .textContentType(.emailAddress)
                    .autocapitalization(.none)
                    .padding(12)
                    .background(Color.white.opacity(0.08))
                    .cornerRadius(6)

                SecureField("Password", text: $password)
                    .padding(12)
                    .background(Color.white.opacity(0.08))
                    .cornerRadius(6)

                Button("Sign in") {
                    loggedIn = true
                    statusMessage = "Signed in (stub — wire to Rust FFI)"
                }
                .buttonStyle(CinematicButtonStyle())
                .disabled(email.isEmpty)
            }
        }
    }

    private var cloudSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            sectionHeader("Cloud backup", subtitle: "Sync project bundles when signed in or locally")

            Button("Backup current project") {
                statusMessage = store.currentProject != nil
                    ? "Backup queued (stub)"
                    : "Open a project first"
            }
            .buttonStyle(CinematicButtonStyle(outlined: true))
            .disabled(store.currentProject == nil)

            Button("Restore from backup") {
                statusMessage = "Restore picker (stub)"
            }
            .buttonStyle(CinematicButtonStyle(outlined: true))
        }
    }

    private var billingSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            sectionHeader("CinemaStudio Pro", subtitle: "Stripe subscription — optional")

            HStack {
                Text(isPro ? "Pro active" : "Free tier")
                    .foregroundStyle(.white)
                Spacer()
                if isPro {
                    Button("Cancel") { isPro = false }
                        .font(.caption)
                } else {
                    Button("Upgrade") { isPro = true }
                        .font(.caption)
                }
            }
            .padding(12)
            .background(Color.white.opacity(0.06))
            .cornerRadius(6)
        }
    }

    private var privacySection: some View {
        VStack(alignment: .leading, spacing: 12) {
            sectionHeader("Privacy", subtitle: "Opt-in only")

            Toggle("Session telemetry", isOn: $telemetryEnabled)
                .tint(.white)
                .foregroundStyle(.white)

            Toggle("Crash reports", isOn: $crashReporting)
                .tint(.white)
                .foregroundStyle(.white)
        }
    }

    private func sectionHeader(_ title: String, subtitle: String) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title)
                .font(.headline)
                .foregroundStyle(.white)
            Text(subtitle)
                .font(.caption)
                .foregroundStyle(.white.opacity(0.45))
        }
    }
}

#Preview {
    NavigationStack {
        SettingsView()
            .environmentObject(ProjectStore())
    }
}
