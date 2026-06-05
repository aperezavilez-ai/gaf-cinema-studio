import Foundation
import SwiftUI

struct ProjectEntry: Identifiable, Codable {
    let id: UUID
    let name: String
    let path: String
    let openedAt: Date
}

/// Bridges SwiftUI to Rust engine via UniFFI (Phase 1 scaffold).
/// Until FFI bindings are generated, uses mock behavior for UI development.
@MainActor
final class ProjectStore: ObservableObject {
    @Published var currentProject: ProjectEntry?
    @Published var recentProjects: [ProjectEntry] = []

    @Published var canUndo = false
    @Published var canRedo = false
    @Published var deviceTier = "Mid"
    @Published var thermalLevel = "Normal"
    @Published var previewQuality = "Auto"

    private let recentKey = "cinemastudio.recentProjects"

    init() {
        loadRecent()
        EngineBridge.shared.initialize()
    }

    func createProject(name: String) {
        let trimmed = name.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }

        let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
        let parent = docs.path

        do {
            let path = try EngineBridge.shared.createProject(name: trimmed, parentDir: parent)
            let entry = ProjectEntry(id: UUID(), name: trimmed, path: path, openedAt: Date())
            currentProject = entry
            addRecent(entry)
        } catch {
            // Fallback mock path if bridge fails
            let path = docs.appendingPathComponent("\(trimmed.replacingOccurrences(of: " ", with: "_")).csproj").path
            let entry = ProjectEntry(id: UUID(), name: trimmed, path: path, openedAt: Date())
            currentProject = entry
            addRecent(entry)
        }
    }

    func openProject(at path: String) {
        let name: String
        do {
            name = try EngineBridge.shared.openProject(projectDir: path)
        } catch {
            name = (path as NSString).lastPathComponent.replacingOccurrences(of: ".csproj", with: "")
        }
        let entry = ProjectEntry(id: UUID(), name: name, path: path, openedAt: Date())
        currentProject = entry
        addRecent(entry)
    }

    // Phase 2 playback stubs — wire to UniFFI
    func scrubTo(ms: Double) {
        // TODO: cs_scrub_to(timeMs: UInt64(ms))
    }

    func playbackPlay() {
        // TODO: cs_playback_play()
    }

    func playbackPause() {
        // TODO: cs_playback_pause()
    }

    func loadTimelineMetadata(onDuration: @escaping (Double) -> Void) {
        onDuration(5000)
    }

    func splitAtPlayhead() { /* TODO: cs_split_at_playhead() */ }
    func deleteAtPlayhead() { /* TODO: cs_delete_at_playhead() */ }
    func undo() { canUndo = false; canRedo = true }
    func redo() { canRedo = false; canUndo = true }
    func startExport() { /* TODO: cs_start_export() */ }

    func loadSuggestions(onResult: @escaping ([AiSuggestionItem]) -> Void) {
        // TODO: cs_ai_suggestions() via UniFFI
        onResult([
            AiSuggestionItem(
                id: UUID(),
                message: "Tienes clips sin colocar en la timeline. ¿Crear un rough cut automático?",
                priority: "high",
                actionLabel: "Ejecutar",
                isActionable: true
            )
        ])
    }

    func executeSuggestion(id: UUID) { /* TODO: cs_ai_execute(id) */ }
    func dismissSuggestion(id: UUID) { /* TODO: cs_ai_dismiss(id) */ }

    func openProjectPicker() {
        // Phase 1: UIDocumentPickerViewController for .csproj folders
    }

    private func addRecent(_ entry: ProjectEntry) {
        recentProjects.removeAll { $0.path == entry.path }
        recentProjects.insert(entry, at: 0)
        if recentProjects.count > 10 { recentProjects = Array(recentProjects.prefix(10)) }
        saveRecent()
    }

    private func loadRecent() {
        guard let data = UserDefaults.standard.data(forKey: recentKey),
              let decoded = try? JSONDecoder().decode([ProjectEntry].self, from: data) else { return }
        recentProjects = decoded
    }

    private func saveRecent() {
        guard let data = try? JSONEncoder().encode(recentProjects) else { return }
        UserDefaults.standard.set(data, forKey: recentKey)
    }
}
