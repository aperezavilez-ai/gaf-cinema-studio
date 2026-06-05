import AVFoundation
import Foundation
import ImageIO
import UIKit

/// AVFoundation decode — native preview frames until Rust callback is wired.
@MainActor
final class VideoDecoderService {
    static let shared = VideoDecoderService()

    private var cache: [String: Data] = [:]

    private init() {}

    func decodeFrame(path: String, timeMs: UInt64, width: Int = 1920, height: Int = 1080) async throws -> UIImage {
        let key = "\(path)@\(timeMs)"
        if let cached = cache[key], let img = UIImage(data: cached) {
            return img
        }

        let url = URL(fileURLWithPath: path)
        guard FileManager.default.fileExists(atPath: path) else {
            throw DecodeError.fileNotFound
        }

        let asset = AVURLAsset(url: url)
        let generator = AVAssetImageGenerator(asset: asset)
        generator.appliesPreferredTrackTransform = true
        generator.requestedTimeToleranceBefore = .zero
        generator.requestedTimeToleranceAfter = CMTime(value: 50, timescale: 1000)
        generator.maximumSize = CGSize(width: width, height: height)

        let cmTime = CMTime(value: Int64(timeMs), timescale: 1000)
        let cgImage = try generator.copyCGImage(at: cmTime, actualTime: nil)

        guard let png = pngData(from: cgImage) else {
            throw DecodeError.encodeFailed
        }
        cache[key] = png
        guard let image = UIImage(data: png) else {
            throw DecodeError.encodeFailed
        }
        return image
    }

    func registerWithEngine() {
        EngineBridge.shared.registerNativeDecoder()
    }

    func clearCache() {
        cache.removeAll()
    }

    private func pngData(from image: CGImage) -> Data? {
        let data = NSMutableData()
        guard let dest = CGImageDestinationCreateWithData(data, "public.png" as CFString, 1, nil) else {
            return nil
        }
        CGImageDestinationAddImage(dest, image, nil)
        guard CGImageDestinationFinalize(dest) else { return nil }
        return data as Data
    }

    enum DecodeError: LocalizedError {
        case fileNotFound
        case encodeFailed

        var errorDescription: String? {
            switch self {
            case .fileNotFound: return "Video file not found"
            case .encodeFailed: return "Failed to encode preview frame"
            }
        }
    }
}
