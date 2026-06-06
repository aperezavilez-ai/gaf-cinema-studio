import Foundation

#if CINEMASTUDIO_ENGINE_LINKED

enum CinemaStudioFFI {
    static var isLinked: Bool { true }

    static func engineInit(dataRoot: String?) {
        if let root = dataRoot {
            root.withCString { cs_c_engine_init($0) }
        } else {
            _ = cs_c_engine_init(nil)
        }
    }

    static func takeString(_ ptr: UnsafeMutablePointer<CChar>?) -> String {
        guard let ptr else { return "" }
        let value = String(cString: ptr)
        cs_c_free_string(ptr)
        return value
    }

    static func requireString(_ ptr: UnsafeMutablePointer<CChar>?) throws -> String {
        let value = takeString(ptr)
        if value.hasPrefix("ERROR:") {
            throw EngineError.native(value.replacingOccurrences(of: "ERROR:", with: ""))
        }
        return value
    }
}

@_silgen_name("cs_c_free_string")
func cs_c_free_string(_ s: UnsafeMutablePointer<CChar>?)

@_silgen_name("cs_c_engine_init")
func cs_c_engine_init(_ dataRoot: UnsafePointer<CChar>?) -> Int32

@_silgen_name("cs_c_bridge_status")
func cs_c_bridge_status() -> UnsafeMutablePointer<CChar>?

@_silgen_name("cs_c_create_project")
func cs_c_create_project(_ name: UnsafePointer<CChar>, _ parent: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_silgen_name("cs_c_open_project")
func cs_c_open_project(_ dir: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_silgen_name("cs_c_save_project")
func cs_c_save_project() -> Int32

@_silgen_name("cs_c_import_media")
func cs_c_import_media(_ path: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_silgen_name("cs_c_scrub_to")
func cs_c_scrub_to(_ timeMs: UInt64) -> UnsafeMutablePointer<CChar>?

@_silgen_name("cs_c_playback_play")
func cs_c_playback_play() -> Int32

@_silgen_name("cs_c_playback_pause")
func cs_c_playback_pause() -> Int32

@_silgen_name("cs_c_playback_tick")
func cs_c_playback_tick() -> UnsafeMutablePointer<CChar>?

@_silgen_name("cs_c_split_at_playhead")
func cs_c_split_at_playhead() -> Int32

@_silgen_name("cs_c_delete_at_playhead")
func cs_c_delete_at_playhead() -> Int32

@_silgen_name("cs_c_timeline_info")
func cs_c_timeline_info() -> UnsafeMutablePointer<CChar>?

@_silgen_name("cs_c_add_clip")
func cs_c_add_clip(_ mediaId: UnsafePointer<CChar>, _ startMs: Int64) -> UnsafeMutablePointer<CChar>?

@_silgen_name("cs_c_start_export")
func cs_c_start_export(_ width: UInt32, _ height: UInt32, _ frameRate: Double) -> UnsafeMutablePointer<CChar>?

@_silgen_name("cs_c_export_status")
func cs_c_export_status() -> UnsafeMutablePointer<CChar>?

@_silgen_name("cs_c_undo")
func cs_c_undo() -> Int32

@_silgen_name("cs_c_redo")
func cs_c_redo() -> Int32

@_silgen_name("cs_c_can_undo")
func cs_c_can_undo() -> Int32

@_silgen_name("cs_c_can_redo")
func cs_c_can_redo() -> Int32

@_silgen_name("cs_c_ai_suggestions")
func cs_c_ai_suggestions() -> UnsafeMutablePointer<CChar>?

@_silgen_name("cs_c_set_decoder_backend")
func cs_c_set_decoder_backend(_ name: UnsafePointer<CChar>) -> Int32

#else

enum CinemaStudioFFI {
    static var isLinked: Bool { false }
}

#endif

enum EngineError: LocalizedError {
    case native(String)

    var errorDescription: String? {
        switch self {
        case .native(let msg): return msg
        }
    }
}
