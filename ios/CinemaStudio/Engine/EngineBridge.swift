import Foundation

/// Facade — selects mock or native Rust backend via compile flag `CINEMASTUDIO_ENGINE_LINKED`.
@MainActor
final class EngineBridge {
    static let shared = EngineBridge()

    private let backend: EngineBackend

    var useNativeEngine: Bool {
        #if CINEMASTUDIO_ENGINE_LINKED
        true
        #else
        false
        #endif
    }

    private init() {
        #if CINEMASTUDIO_ENGINE_LINKED
        backend = NativeEngineBackend()
        #else
        backend = MockEngineBackend()
        #endif
    }

    func initialize(dataRoot: String? = nil) {
        try? backend.initialize(dataRoot: dataRoot)
    }

    func createProject(name: String, parentDir: String) throws -> String {
        try backend.createProject(name: name, parentDir: parentDir)
    }

    func openProject(projectDir: String) throws -> String {
        try backend.openProject(projectDir: projectDir)
    }

    func saveProject(at path: String) throws {
        try backend.saveProject(at: path)
    }

    func importMedia(sourcePath: String, into projectPath: String) throws -> UUID {
        try backend.importMedia(sourcePath: sourcePath, into: projectPath)
    }

    func scrubTo(timeMs: UInt64) throws -> FrameCompositionDTO {
        try backend.scrubTo(timeMs: timeMs)
    }

    func playbackPlay() throws { try backend.playbackPlay() }
    func playbackPause() throws { try backend.playbackPause() }
    func splitAtPlayhead() throws -> Bool { try backend.splitAtPlayhead() }
    func deleteAtPlayhead() throws -> Bool { try backend.deleteAtPlayhead() }
    func undo() throws -> Bool { try backend.undo() }
    func redo() throws -> Bool { try backend.redo() }
    func startExport(width: Int = 1920, height: Int = 1080) throws -> UUID {
        try backend.startExport(width: width, height: height)
    }

    func aiSuggestions() throws -> [AiSuggestionItem] { try backend.aiSuggestions() }
    func executeSuggestion(id: UUID) throws { try backend.executeSuggestion(id: id) }
    func dismissSuggestion(id: UUID) throws { try backend.dismissSuggestion(id: id) }
    func setDeviceHints(json: String) throws { try backend.setDeviceHints(json: json) }
    func registerNativeDecoder() { backend.registerNativeDecoder() }
    func bridgeStatus() -> String { backend.bridgeStatus() }
}

struct FrameCompositionDTO: Codable {
    let timeMs: UInt64
    let videoLayerCount: UInt32
    let primaryPath: String?
    let usesProxy: Bool
}

struct AiSuggestionItem: Identifiable {
    let id: UUID
    let message: String
    let priority: String
    let actionLabel: String
    let isActionable: Bool
}
