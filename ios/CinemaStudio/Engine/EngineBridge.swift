import Foundation

/// Single entry point for Rust engine calls.
/// Mock mode: local file ops + AVFoundation preview. Native: UniFFI `cs_*` when linked.
@MainActor
final class EngineBridge {
    static let shared = EngineBridge()

    var useNativeEngine = false

    private init() {}

    func initialize(dataRoot: String? = nil) {
        guard useNativeEngine else { return }
        // csEngineInit(dataRoot: dataRoot)
    }

    // MARK: - Project

    func createProject(name: String, parentDir: String) throws -> String {
        if useNativeEngine { fatalError("UniFFI bindings not linked") }
        let safe = name.replacingOccurrences(of: " ", with: "_")
        let path = (parentDir as NSString).appendingPathComponent("\(safe).csproj")
        try FileManager.default.createDirectory(atPath: path, withIntermediateDirectories: true)
        try writeMinimalProjectJson(at: path, name: name)
        return path
    }

    func openProject(projectDir: String) throws -> String {
        if useNativeEngine { fatalError("UniFFI bindings not linked") }
        return (projectDir as NSString).lastPathComponent.replacingOccurrences(of: ".csproj", with: "")
    }

    func saveProject(at path: String) throws {
        if useNativeEngine { return }
        guard FileManager.default.fileExists(atPath: path) else { return }
        // Mock: touch project.json timestamp
        let jsonPath = (path as NSString).appendingPathComponent("project.json")
        if FileManager.default.fileExists(atPath: jsonPath) {
            try FileManager.default.setAttributes([.modificationDate: Date()], ofItemAtPath: jsonPath)
        }
    }

    // MARK: - Media + timeline

    func importMedia(sourcePath: String, into projectPath: String) throws -> UUID {
        if useNativeEngine { fatalError("UniFFI bindings not linked") }
        let mediaDir = (projectPath as NSString).appendingPathComponent("media")
        try FileManager.default.createDirectory(atPath: mediaDir, withIntermediateDirectories: true)
        let fileName = (sourcePath as NSString).lastPathComponent
        let dest = (mediaDir as NSString).appendingPathComponent(fileName)
        if sourcePath != dest {
            if FileManager.default.fileExists(atPath: dest) {
                try FileManager.default.removeItem(atPath: dest)
            }
            try FileManager.default.copyItem(atPath: sourcePath, toPath: dest)
        }
        return UUID()
    }

    func scrubTo(timeMs: UInt64) throws -> FrameCompositionDTO {
        if useNativeEngine { fatalError("UniFFI bindings not linked") }
        return FrameCompositionDTO(
            timeMs: timeMs,
            videoLayerCount: 1,
            primaryPath: nil,
            usesProxy: false
        )
    }

    // MARK: - Playback

    func playbackPlay() throws {
        if useNativeEngine { return }
    }

    func playbackPause() throws {
        if useNativeEngine { return }
    }

    // MARK: - Edit

    func splitAtPlayhead() throws -> Bool { !useNativeEngine }
    func deleteAtPlayhead() throws -> Bool { !useNativeEngine }
    func undo() throws -> Bool { !useNativeEngine }
    func redo() throws -> Bool { !useNativeEngine }

    // MARK: - Export

    func startExport(width: Int = 1920, height: Int = 1080) throws -> UUID {
        if useNativeEngine { fatalError("UniFFI bindings not linked") }
        return UUID()
    }

    // MARK: - AI

    func aiSuggestions() throws -> [AiSuggestionItem] {
        if useNativeEngine { fatalError("UniFFI bindings not linked") }
        return [
            AiSuggestionItem(
                id: UUID(),
                message: "Importa clips y colócalos en la timeline para un rough cut automático.",
                priority: "high",
                actionLabel: "Ejecutar",
                isActionable: true
            )
        ]
    }

    func executeSuggestion(id: UUID) throws {}
    func dismissSuggestion(id: UUID) throws {}

    // MARK: - Device

    func setDeviceHints(json: String) throws {
        if useNativeEngine { return }
    }

    func registerNativeDecoder() {
        if useNativeEngine {
            // try? csSetDecoderBackend(name: "avfoundation")
        }
    }

    func bridgeStatus() -> String {
        if useNativeEngine {
            return "{\"mode\":\"native\",\"decodeCallbackRegistered\":false}"
        }
        return "{\"mode\":\"mock\",\"avFoundationPreview\":true}"
    }

    private func writeMinimalProjectJson(at projectPath: String, name: String) throws {
        let jsonPath = (projectPath as NSString).appendingPathComponent("project.json")
        let payload: [String: Any] = [
            "schemaVersion": 1,
            "metadata": ["name": name, "createdAt": ISO8601DateFormatter().string(from: Date())],
        ]
        let data = try JSONSerialization.data(withJSONObject: payload, options: [.prettyPrinted, .sortedKeys])
        try data.write(to: URL(fileURLWithPath: jsonPath))
    }
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
