import SwiftUI

struct PreviewFrameView: View {
    let image: UIImage?
    let isPlaying: Bool
    let isDecoding: Bool
    let error: String?

    var body: some View {
        ZStack {
            Rectangle().fill(Color(white: 0.08))

            if let image {
                Image(uiImage: image)
                    .resizable()
                    .scaledToFit()
            } else {
                VStack(spacing: 12) {
                    if isDecoding {
                        ProgressView().tint(.white.opacity(0.5))
                    } else {
                        Image(systemName: isPlaying ? "play.fill" : "film")
                            .font(.system(size: 48, weight: .ultraLight))
                            .foregroundStyle(.white.opacity(0.3))
                    }
                    Text(statusLabel)
                        .font(.caption)
                        .tracking(3)
                        .foregroundStyle(.white.opacity(0.25))
                }
            }
        }
        .aspectRatio(CinemaTheme.previewAspect, contentMode: .fit)
        .padding(.horizontal, 16)
        .padding(.top, 8)
    }

    private var statusLabel: String {
        if let error { return error.uppercased() }
        if isDecoding { return "DECODING" }
        if isPlaying { return "PLAYING" }
        return image == nil ? "PREVIEW" : "FRAME"
    }
}

#Preview {
    PreviewFrameView(image: nil, isPlaying: false, isDecoding: false, error: nil)
        .background(Color.black)
}
