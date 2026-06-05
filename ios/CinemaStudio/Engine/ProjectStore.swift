import Foundation
import SwiftUI
import Combine

/// Bridges SwiftUI to Rust engine via EngineBridge. Native preview via AVFoundation.
@MainActor
final class ProjectStore: ObservableObject {
    @Published var currentProject: ProjectEntry?
    @Published var recentProjects: [ProjectEntry] = []

    @Published var canUndo = false
    @Published var canRedo = false
    @Published var deviceTier = "Mid"
    @Published var thermalLevel = "Normal"
    @Published var previewQuality = "Auto"

    @Published var previewPath: String?
    @Published var playheadMs: Double = 0
    @Published var durationMs: Double = 5000
    @Published var isPlaying = false
    @Published var exportStatus: String = ""
    @Published var isExporting = false
    @Published var exportProgress: Double = 0

    @Published var betaCompletions: Int = 0
    private let betaTarget = 10
    private let betaKey = "cinemastudio.betaCompletions"

    @Published var showProjectPicker = false

    let previewVM = PreviewViewModel()

    private let recentKey = "cinemastudio.recentProjects"
    private var playbackTimer: Timer?
    private var exportTimer: Timer?

    init() {
        loadRecent()
        betaCompletions = UserDefaults.standard.integer(forKey: betaKey)
        EngineBridge.shared.initialize()
        VideoDecoderService.shared.registerWithEngine()
    }

    deinit {
        playbackTimer?.invalidate()
        exportTimer?.invalidate()
    }

    // MARK: - Project

    func createProject(name: String) {
        let trimmed = name.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }

        let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
        do {
            let path = try EngineBridge.shared.createProject(name: trimmed, parentDir: docs.path)
            let entry = ProjectEntry(id: UUID(), name: trimmed, path: path, openedAt: Date())
            currentProject = entry
            addRecent(entry)
            resetEditorState()
        } catch {
            exportStatus = "Error creating project"
        }
    }

    func openProject(at path: String) {
        let name = (try? EngineBridge.shared.openProject(projectDir: path))
            ?? (path as NSString).lastPathComponent.replacingOccurrences(of: ".csproj", with: "")
        let entry = ProjectEntry(id: UUID(), name: name, path: path, openedAt: Date())
        currentProject = entry
        addRecent(entry)
        loadDemoMediaIfNeeded()
        refreshPreview(at: playheadMs)
    }

    func openProjectPicker() {
        showProjectPicker = true
    }

    // MARK: - Playback

    func scrubTo(ms: Double) {
        playheadMs = ms
        if let frame = try? EngineBridge.shared.scrubTo(timeMs: UInt64(ms)) {
            if let path = frame.primaryPath {
                previewPath = path
            }
        }
        refreshPreview(at: ms)
    }

    func importMedia(from sourcePath: String, into projectPath: String) {
        do {
            _ = try EngineBridge.shared.importMedia(sourcePath: sourcePath, into: projectPath)
            loadDemoMediaIfNeeded()
            refreshPreview(at: playheadMs)
        } catch {
            exportStatus = "Import failed"
        }
    }

    func playbackPlay() {
        isPlaying = true
        try? EngineBridge.shared.playbackPlay()
        playbackTimer?.invalidate()
        playbackTimer = Timer.scheduledTimer(withTimeInterval: 1.0 / 24.0, repeats: true) { [weak self] _ in
            Task { @MainActor in
                guard let self, self.isPlaying else { return }
                let next = min(self.playheadMs + 1000.0 / 24.0, self.durationMs)
                self.scrubTo(ms: next)
                if next >= self.durationMs { self.playbackPause() }
            }
        }
    }

    func playbackPause() {
        isPlaying = false
        playbackTimer?.invalidate()
        playbackTimer = nil
        try? EngineBridge.shared.playbackPause()
    }

    func loadTimelineMetadata(onDuration: @escaping (Double) -> Void) {
        onDuration(durationMs)
    }

    // MARK: - Edit

    func splitAtPlayhead() {
        if (try? EngineBridge.shared.splitAtPlayhead()) == true {
            canUndo = true
        }
    }

    func deleteAtPlayhead() {
        if (try? EngineBridge.shared.deleteAtPlayhead()) == true {
            canUndo = true
        }
    }

    func undo() {
        if (try? EngineBridge.shared.undo()) == true {
            canRedo = true
            canUndo = false
        }
    }

    func redo() {
        if (try? EngineBridge.shared.redo()) == true {
            canUndo = true
            canRedo = false
        }
    }

    func startExport() {
        exportStatus = "Exporting…"
        isExporting = true
        exportProgress = 0.1
        Task {
            _ = try? EngineBridge.shared.startExport()
            startExportPolling()
        }
    }

    private func startExportPolling() {
        exportTimer?.invalidate()
        var ticks = 0
        exportTimer = Timer.scheduledTimer(withTimeInterval: 0.4, repeats: true) { [weak self] _ in
            Task { @MainActor in
                guard let self else { return }
                ticks += 1
                self.exportProgress = min(0.95, Double(ticks) * 0.08)
                if ticks >= 15 {
                    self.isExporting = false
                    self.exportProgress = 1.0
                    self.exportStatus = "Export complete (stub/mock)"
                    self.exportTimer?.invalidate()
                }
            }
        }
    }

    // MARK: - AI

    func loadSuggestions(onResult: @escaping ([AiSuggestionItem]) -> Void) {
        onResult((try? EngineBridge.shared.aiSuggestions()) ?? [])
    }

    func executeSuggestion(id: UUID) {
        try? EngineBridge.shared.executeSuggestion(id: id)
    }

    func dismissSuggestion(id: UUID) {
        try? EngineBridge.shared.dismissSuggestion(id: id)
    }

    func betaStatus() -> BetaStatus {
        BetaStatus(
            completions: betaCompletions,
            target: betaTarget,
            readyToShip: betaCompletions >= betaTarget
        )
    }

    func shipProject(userLabel: String) {
        guard currentProject != nil else { return }
        if !UserDefaults.standard.bool(forKey: "beta_\(currentProject!.id.uuidString)") {
            betaCompletions += 1
            UserDefaults.standard.set(betaCompletions, forKey: betaKey)
            UserDefaults.standard.set(true, forKey: "beta_\(currentProject!.id.uuidString)")
        }
        exportStatus = "Project shipped — thank you, \(userLabel)!"
    }

    // MARK: - Private

    private func resetEditorState() {
        playheadMs = 0
        durationMs = 5000
        previewPath = nil
        previewVM.clear()
        playbackPause()
    }

    private func loadDemoMediaIfNeeded() {
        guard let project = currentProject else { return }
        let mediaDir = (project.path as NSString).appendingPathComponent("media")
        if let files = try? FileManager.default.contentsOfDirectory(atPath: mediaDir) {
            for file in files where file.hasSuffix(".mp4") || file.hasSuffix(".mov") {
                previewPath = (mediaDir as NSString).appendingPathComponent(file)
                durationMs = 5000
                return
            }
        }
    }

    private func refreshPreview(at ms: Double) {
        guard let path = previewPath else {
            previewVM.clear()
            return
        }
        previewVM.updatePreview(path: path, timeMs: UInt64(ms))
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

struct ProjectEntry: Identifiable, Codable {
    let id: UUID
    let name: String
    let path: String
    let openedAt: Date
}
