import SwiftUI

/// Cinematic design tokens — premium, minimal, dark-first.
enum CinemaTheme {
    static let background = Color.black
    static let surface = Color(white: 0.08)
    static let surfaceElevated = Color(white: 0.12)
    static let accent = Color.white
    static let textPrimary = Color.white
    static let textSecondary = Color.white.opacity(0.5)
    static let textMuted = Color.white.opacity(0.25)
    static let trackHeight: CGFloat = 80
    static let previewAspect: CGFloat = 16 / 9
}

struct CinemaSurface: ViewModifier {
    func body(content: Content) -> some View {
        content
            .background(CinemaTheme.surfaceElevated)
            .cornerRadius(8)
    }
}

extension View {
    func cinemaSurface() -> some View {
        modifier(CinemaSurface())
    }
}
