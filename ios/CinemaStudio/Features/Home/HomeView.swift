import SwiftUI

/// Local project browser — New / Open .csproj folders.
struct HomeView: View {
    @EnvironmentObject var store: ProjectStore
    @State private var newProjectName = ""
    @State private var showCreateSheet = false

    var body: some View {
        ZStack {
            Color.black.ignoresSafeArea()

            VStack(spacing: 32) {
                VStack(spacing: 8) {
                    Text("CINEMASTUDIO")
                        .font(.system(size: 28, weight: .light, design: .default))
                        .tracking(6)
                        .foregroundStyle(.white)

                    Text("Mobile cinematic studio")
                        .font(.subheadline)
                        .foregroundStyle(.white.opacity(0.5))
                }
                .padding(.top, 48)

                if let project = store.currentProject {
                    NavigationLink(destination: EditorView()) {
                        ProjectBanner(name: project.name, path: project.path)
                    }
                }

                VStack(spacing: 16) {
                    Button(action: { showCreateSheet = true }) {
                        Label("New Project", systemImage: "plus.rectangle.on.rectangle")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(CinematicButtonStyle())

                    Button(action: { store.openProjectPicker() }) {
                        Label("Open Project", systemImage: "folder")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(CinematicButtonStyle(outlined: true))

                    NavigationLink(destination: SettingsView()) {
                        Label("Settings", systemImage: "gearshape")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(CinematicButtonStyle(outlined: true))
                }
                .padding(.horizontal, 32)

                if !store.recentProjects.isEmpty {
                    RecentProjectsList(projects: store.recentProjects) { path in
                        store.openProject(at: path)
                    }
                }

                Spacer()
            }
        }
        .sheet(isPresented: $showCreateSheet) {
            CreateProjectSheet(name: $newProjectName) {
                store.createProject(name: newProjectName)
                showCreateSheet = false
            }
        }
        .sheet(isPresented: $store.showProjectPicker) {
            ProjectDocumentPicker { url in
                store.openProject(at: url.path)
                store.showProjectPicker = false
            }
        }
    }
}

struct CinematicButtonStyle: ButtonStyle {
    var outlined: Bool = false

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.headline)
            .padding(.vertical, 16)
            .background(outlined ? Color.clear : Color.white.opacity(configuration.isPressed ? 0.8 : 1))
            .foregroundStyle(outlined ? .white : .black)
            .overlay(
                RoundedRectangle(cornerRadius: 4)
                    .stroke(Color.white.opacity(outlined ? 0.4 : 0), lineWidth: 1)
            )
            .cornerRadius(4)
            .scaleEffect(configuration.isPressed ? 0.98 : 1)
    }
}

struct ProjectBanner: View {
    let name: String
    let path: String

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(name)
                .font(.title3)
                .foregroundStyle(.white)
            Text(path)
                .font(.caption)
                .foregroundStyle(.white.opacity(0.4))
                .lineLimit(1)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding()
        .background(Color.white.opacity(0.06))
        .cornerRadius(8)
        .padding(.horizontal, 32)
    }
}

struct RecentProjectsList: View {
    let projects: [ProjectEntry]
    let onSelect: (String) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("RECENT")
                .font(.caption)
                .tracking(2)
                .foregroundStyle(.white.opacity(0.4))
                .padding(.horizontal, 32)

            ForEach(projects) { project in
                Button(action: { onSelect(project.path) }) {
                    HStack {
                        Text(project.name)
                            .foregroundStyle(.white)
                        Spacer()
                        Image(systemName: "chevron.right")
                            .foregroundStyle(.white.opacity(0.3))
                    }
                    .padding(.horizontal, 32)
                    .padding(.vertical, 12)
                }
            }
        }
    }
}

struct CreateProjectSheet: View {
    @Binding var name: String
    let onCreate: () -> Void

    var body: some View {
        NavigationStack {
            Form {
                TextField("Project name", text: $name)
            }
            .navigationTitle("New Project")
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Create", action: onCreate)
                        .disabled(name.trimmingCharacters(in: .whitespaces).isEmpty)
                }
            }
        }
        .presentationDetents([.medium])
    }
}

#Preview {
    HomeView()
        .environmentObject(ProjectStore())
}
