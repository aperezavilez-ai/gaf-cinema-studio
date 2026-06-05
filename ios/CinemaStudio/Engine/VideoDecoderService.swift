import AVFoundation
import Foundation

/// AVFoundation decode service — wire to Rust `native_bridge` at integration.
/// Until UniFFI callbacks exist, EngineBridge uses mock frames.
@MainActor
final class VideoDecoderService {
    static let shared = VideoDecoderService()

    private init() {}

    func decodeFrame(path: String, timeMs: UInt64, width: Int, height: Int) async throws -> Data {
        let url = URL(fileURLWithPath: path)
        let asset = AVURLAsset(url: url)
        let generator = AVAssetImageGenerator(asset: asset)
        generator.appliesPreferredTrackTransform = true
        generator.maximumSize = CGSize(width: width, height: height)

        let cmTime = CMTime(value: Int64(timeMs), timescale: 1000)
        let cgImage = try generator.copyCGImage(at: cmTime, actualTime: nil)
        return pngData(from: cgImage) ?? Data()
    }

    func registerWithEngine() {
        EngineBridge.shared.registerNativeDecoder()
    }

    private func pngData(from image: CGImage) -> Data? {
        let data = NSMutableData()
        guard let dest = CGImageDestinationCreateWithData(data, "public.png" as CFString, 1, nil) else {
            return nil
        }
        CGImageDestinationAddImage(dest, image, nil)
        CGImageDestinationFinalize(dest)
        return data as Data
    }
}
