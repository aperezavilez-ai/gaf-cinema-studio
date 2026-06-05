import Foundation

/// Single entry point for Rust engine calls.
/// When UniFFI bindings are generated, set `useNativeEngine = true` and implement stubs below.
@MainActor
final class EngineBridge {
    static let shared = EngineBridge()

    /// Flip to true after `scripts/generate_bindings.ps1` produces Swift bindings.
    var useNativeEngine = false

    private init() {}

    func initialize(dataRoot: String? = nil) {
        guard useNativeEngine else { return }
        // csEngineInit(dataRoot: dataRoot)
    }

    func createProject(name: String, parentDir: String) throws -> String {
        if useNativeEngine {
            // return try csCreateProject(name: name, parentDir: parentDir)
            fatalError("UniFFI bindings not linked")
        }
        let path = (parentDir as NSString).appendingPathComponent(
            "\(name.replacingOccurrences(of: " ", with: "_")).csproj"
        )
        try FileManager.default.createDirectory(atPath: path, withIntermediateDirectories: true)
        return path
    }

    func openProject(projectDir: String) throws -> String {
        if useNativeEngine {
            // return try csOpenProject(projectDir: projectDir)
            fatalError("UniFFI bindings not linked")
        }
        return (projectDir as NSString).lastPathComponent.replacingOccurrences(of: ".csproj", with: "")
    }

    func importMedia(sourcePath: String) throws -> UUID {
        if useNativeEngine {
            // let id = try csImportMedia(sourcePath: sourcePath)
            // return UUID(uuidString: id)!
            fatalError("UniFFI bindings not linked")
        }
        return UUID()
    }

    func scrubTo(timeMs: UInt64) throws -> FrameCompositionDTO {
        if useNativeEngine {
            // let json = try csScrubTo(timeMs: timeMs)
            // return try JSONDecoder().decode(FrameCompositionDTO.self, from: Data(json.utf8))
            fatalError("UniFFI bindings not linked")
        }
        return FrameCompositionDTO(timeMs: timeMs, videoLayerCount: 1, primaryPath: nil, usesProxy: false)
    }

    func playbackPlay() throws {
        if useNativeEngine { /* try csPlaybackPlay() */ }
    }

    func playbackPause() throws {
        if useNativeEngine { /* try csPlaybackPause() */ }
    }

    func aiSuggestions() throws -> [AiSuggestionItem] {
        if useNativeEngine {
            // let json = try csAiSuggestions()
            fatalError("UniFFI bindings not linked")
        }
        return []
    }

    func setDeviceHints(json: String) throws {
        if useNativeEngine { /* try csSetDeviceHints(json: json) */ }
    }

    func bridgeStatus() -> String {
        if useNativeEngine {
            // return (try? csBridgeStatus()) ?? "{}"
            return "{\"decodeCallbackRegistered\":false}"
        }
        return "{\"mode\":\"mock\"}"
    }

    /// Register AVFoundation decode — call from VideoDecoderService when wired.
    func registerNativeDecoder() {
        // Future: pass callback handle to Rust via UniFFI callback interface
        if useNativeEngine {
            // try? csSetDecoderBackend(name: "avfoundation")
        }
    }
}

struct FrameCompositionDTO: Codable {
    let timeMs: UInt64
    let videoLayerCount: UInt32
    let primaryPath: String?
    let usesProxy: Bool
}
