import Foundation
import SwiftUI
import UIKit

/// Coordinates preview frame decode at playhead — native AVFoundation path.
@MainActor
final class PreviewViewModel: ObservableObject {
    @Published var previewImage: UIImage?
    @Published var isDecoding = false
    @Published var lastError: String?

    private var decodeTask: Task<Void, Never>?

    func updatePreview(path: String?, timeMs: UInt64) {
        decodeTask?.cancel()
        guard let path, !path.isEmpty else {
            previewImage = nil
            return
        }

        decodeTask = Task {
            isDecoding = true
            lastError = nil
            defer { isDecoding = false }

            do {
                let image = try await VideoDecoderService.shared.decodeFrame(
                    path: path,
                    timeMs: timeMs
                )
                if !Task.isCancelled {
                    previewImage = image
                }
            } catch {
                if !Task.isCancelled {
                    lastError = error.localizedDescription
                    previewImage = nil
                }
            }
        }
    }

    func clear() {
        decodeTask?.cancel()
        previewImage = nil
        lastError = nil
    }
}
