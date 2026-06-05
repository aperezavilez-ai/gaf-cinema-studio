import Foundation

/// Engine operations contract — mock or native Rust backend.
protocol EngineBackend: Sendable {
    func initialize(dataRoot: String?) throws
    func createProject(name: String, parentDir: String) throws -> String
    func openProject(projectDir: String) throws -> String
    func saveProject(at path: String) throws
    func importMedia(sourcePath: String, into projectPath: String) throws -> UUID
    func scrubTo(timeMs: UInt64) throws -> FrameCompositionDTO
    func playbackPlay() throws
    func playbackPause() throws
    func splitAtPlayhead() throws -> Bool
    func deleteAtPlayhead() throws -> Bool
    func undo() throws -> Bool
    func redo() throws -> Bool
    func startExport(width: Int, height: Int) throws -> UUID
    func aiSuggestions() throws -> [AiSuggestionItem]
    func executeSuggestion(id: UUID) throws
    func dismissSuggestion(id: UUID) throws
    func setDeviceHints(json: String) throws
    func registerNativeDecoder()
    func bridgeStatus() -> String
}

struct MockEngineBackend: EngineBackend {
    func initialize(dataRoot: String?) {}

    func createProject(name: String, parentDir: String) throws -> String {
        let safe = name.replacingOccurrences(of: " ", with: "_")
        let path = (parentDir as NSString).appendingPathComponent("\(safe).csproj")
        try FileManager.default.createDirectory(atPath: path, withIntermediateDirectories: true)
        try writeMinimalProjectJson(at: path, name: name)
        return path
    }

    func openProject(projectDir: String) throws -> String {
        (projectDir as NSString).lastPathComponent.replacingOccurrences(of: ".csproj", with: "")
    }

    func saveProject(at path: String) throws {
        let jsonPath = (path as NSString).appendingPathComponent("project.json")
        if FileManager.default.fileExists(atPath: jsonPath) {
            try FileManager.default.setAttributes([.modificationDate: Date()], ofItemAtPath: jsonPath)
        }
    }

    func importMedia(sourcePath: String, into projectPath: String) throws -> UUID {
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
        FrameCompositionDTO(timeMs: timeMs, videoLayerCount: 0, primaryPath: nil, usesProxy: false)
    }

    func playbackPlay() throws {}
    func playbackPause() throws {}
    func splitAtPlayhead() throws -> Bool { true }
    func deleteAtPlayhead() throws -> Bool { true }
    func undo() throws -> Bool { true }
    func redo() throws -> Bool { true }
    func startExport(width: Int, height: Int) throws -> UUID { UUID() }

    func aiSuggestions() throws -> [AiSuggestionItem] {
        [
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
    func setDeviceHints(json: String) throws {}
    func registerNativeDecoder() {}

    func bridgeStatus() -> String {
        "{\"mode\":\"mock\",\"avFoundationPreview\":true}"
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

#if CINEMASTUDIO_ENGINE_LINKED

struct NativeEngineBackend: EngineBackend {
    func initialize(dataRoot: String?) throws {
        CinemaStudioFFI.engineInit(dataRoot: dataRoot)
    }

    func createProject(name: String, parentDir: String) throws -> String {
        try name.withCString { n in
            try parentDir.withCString { p in
                try CinemaStudioFFI.requireString(cs_c_create_project(n, p))
            }
        }
    }

    func openProject(projectDir: String) throws -> String {
        try projectDir.withCString { dir in
            try CinemaStudioFFI.requireString(cs_c_open_project(dir))
        }
    }

    func saveProject(at path: String) throws {
        _ = path
        guard cs_c_save_project() == 0 else {
            throw EngineError.native("save failed")
        }
    }

    func importMedia(sourcePath: String, into projectPath: String) throws -> UUID {
        _ = projectPath
        let idStr = try sourcePath.withCString { path in
            try CinemaStudioFFI.requireString(cs_c_import_media(path))
        }
        guard let id = UUID(uuidString: idStr) else {
            throw EngineError.native("invalid media id")
        }
        return id
    }

    func scrubTo(timeMs: UInt64) throws -> FrameCompositionDTO {
        let json = try CinemaStudioFFI.requireString(cs_c_scrub_to(timeMs))
        return try JSONDecoder().decode(FrameCompositionDTO.self, from: Data(json.utf8))
    }

    func playbackPlay() throws {
        guard cs_c_playback_play() == 0 else { throw EngineError.native("playback play failed") }
    }

    func playbackPause() throws {
        guard cs_c_playback_pause() == 0 else { throw EngineError.native("playback pause failed") }
    }

    func splitAtPlayhead() throws -> Bool { false }
    func deleteAtPlayhead() throws -> Bool { false }
    func undo() throws -> Bool { cs_c_undo() == 1 }
    func redo() throws -> Bool { cs_c_redo() == 1 }
    func startExport(width: Int, height: Int) throws -> UUID { UUID() }

    func aiSuggestions() throws -> [AiSuggestionItem] {
        let json = try CinemaStudioFFI.requireString(cs_c_ai_suggestions())
        struct Raw: Decodable {
            let id: String
            let message: String
            let priority: String
            let actionLabel: String?
            let isActionable: Bool
        }
        let items = try JSONDecoder().decode([Raw].self, from: Data(json.utf8))
        return items.compactMap { raw in
            guard let id = UUID(uuidString: raw.id) else { return nil }
            return AiSuggestionItem(
                id: id,
                message: raw.message,
                priority: raw.priority,
                actionLabel: raw.actionLabel ?? "Ejecutar",
                isActionable: raw.isActionable
            )
        }
    }

    func executeSuggestion(id: UUID) throws {}
    func dismissSuggestion(id: UUID) throws {}
    func setDeviceHints(json: String) throws {}

    func registerNativeDecoder() {
        "avfoundation".withCString { cs_c_set_decoder_backend($0) }
    }

    func bridgeStatus() -> String {
        CinemaStudioFFI.takeString(cs_c_bridge_status())
    }
}

#endif
