import SwiftUI

/// Timeline + preview + minimal edit toolbar — wired to ProjectStore + AVFoundation preview.
struct EditorView: View {
    @EnvironmentObject var store: ProjectStore

    var body: some View {
        ZStack {
            Color.black.ignoresSafeArea()

            VStack(spacing: 0) {
                PreviewFrameView(
                    image: store.previewVM.previewImage,
                    isPlaying: store.isPlaying,
                    isDecoding: store.previewVM.isDecoding,
                    error: store.previewVM.lastError
                )
                .frame(maxHeight: .infinity)

                DeviceStatusBar(
                    tier: store.deviceTier,
                    thermal: store.thermalLevel,
                    previewQuality: store.previewQuality
                )

                GuidancePanel()
                    .padding(.vertical, 8)

                EditToolbar(
                    canUndo: store.canUndo,
                    canRedo: store.canRedo,
                    onSplit: { store.splitAtPlayhead() },
                    onDelete: { store.deleteAtPlayhead() },
                    onUndo: { store.undo() },
                    onRedo: { store.redo() },
                    onExport: { store.startExport() }
                )
                .padding(.horizontal, 16)
                .padding(.vertical, 8)

                MediaImportPicker()
                    .padding(.horizontal, 16)
                    .environmentObject(store)

                if !store.exportStatus.isEmpty {
                    Text(store.exportStatus)
                        .font(.caption)
                        .foregroundStyle(.white.opacity(0.5))
                }

                TransportBar(
                    playheadMs: Binding(
                        get: { store.playheadMs },
                        set: { store.scrubTo(ms: $0) }
                    ),
                    durationMs: store.durationMs,
                    isPlaying: Binding(
                        get: { store.isPlaying },
                        set: { $0 ? store.playbackPlay() : store.playbackPause() }
                    ),
                    onScrub: { store.scrubTo(ms: $0) },
                    onPlay: { store.playbackPlay() },
                    onPause: { store.playbackPause() }
                )
                .padding(.horizontal, 16)
                .padding(.bottom, 8)

                TimelineStrip(
                    playheadMs: Binding(
                        get: { store.playheadMs },
                        set: { store.scrubTo(ms: $0) }
                    ),
                    durationMs: store.durationMs,
                    onScrub: { store.scrubTo(ms: $0) }
                )
                .frame(height: CinemaTheme.trackHeight)
                .padding(.bottom, 16)
            }
        }
        .navigationTitle(store.currentProject?.name ?? "Editor")
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            store.loadTimelineMetadata { store.durationMs = $0 }
            store.scrubTo(ms: store.playheadMs)
        }
        .onDisappear { store.playbackPause() }
    }
}

struct EditToolbar: View {
    let canUndo: Bool
    let canRedo: Bool
    let onSplit: () -> Void
    let onDelete: () -> Void
    let onUndo: () -> Void
    let onRedo: () -> Void
    let onExport: () -> Void

    var body: some View {
        HStack(spacing: 24) {
            ToolButton(icon: "scissors", label: "Split", action: onSplit)
            ToolButton(icon: "trash", label: "Delete", action: onDelete)
            Spacer()
            ToolButton(icon: "arrow.uturn.backward", label: "Undo", action: onUndo)
                .opacity(canUndo ? 1 : 0.3)
                .disabled(!canUndo)
            ToolButton(icon: "arrow.uturn.forward", label: "Redo", action: onRedo)
                .opacity(canRedo ? 1 : 0.3)
                .disabled(!canRedo)
            ToolButton(icon: "square.and.arrow.up", label: "Export", action: onExport)
        }
    }
}

struct ToolButton: View {
    let icon: String
    let label: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            VStack(spacing: 4) {
                Image(systemName: icon)
                    .font(.body)
                Text(label)
                    .font(.system(size: 9))
                    .tracking(1)
            }
            .foregroundStyle(.white.opacity(0.8))
        }
    }
}

struct TransportBar: View {
    @Binding var playheadMs: Double
    let durationMs: Double
    @Binding var isPlaying: Bool
    let onScrub: (Double) -> Void
    let onPlay: () -> Void
    let onPause: () -> Void

    var body: some View {
        HStack(spacing: 16) {
            Button(action: { isPlaying ? onPause() : onPlay() }) {
                Image(systemName: isPlaying ? "pause.fill" : "play.fill")
                    .font(.title2).foregroundStyle(.white)
            }
            Text(formatTime(playheadMs))
                .font(.system(.caption, design: .monospaced))
                .foregroundStyle(.white.opacity(0.6))
            Slider(value: $playheadMs, in: 0...max(durationMs, 1)) { editing in
                if editing { onScrub(playheadMs) }
            }.tint(.white)
            Text(formatTime(durationMs))
                .font(.system(.caption, design: .monospaced))
                .foregroundStyle(.white.opacity(0.6))
        }
    }

    private func formatTime(_ ms: Double) -> String {
        let s = Int(ms) / 1000
        return String(format: "%d:%02d", s / 60, s % 60)
    }
}

struct TimelineStrip: View {
    @Binding var playheadMs: Double
    let durationMs: Double
    let onScrub: (Double) -> Void

    var body: some View {
        GeometryReader { geo in
            ZStack(alignment: .leading) {
                RoundedRectangle(cornerRadius: 4)
                    .fill(Color(white: 0.12))
                RoundedRectangle(cornerRadius: 4)
                    .fill(Color.white.opacity(0.15))
                    .frame(width: geo.size.width * 0.6)
                    .padding(.leading, 4)
                if durationMs > 0 {
                    Rectangle().fill(Color.white).frame(width: 2)
                        .offset(x: geo.size.width * (playheadMs / durationMs))
                }
            }
            .gesture(DragGesture(minimumDistance: 0).onChanged { v in
                let ratio = max(0, min(1, v.location.x / geo.size.width))
                playheadMs = ratio * durationMs
                onScrub(playheadMs)
            })
        }
        .padding(.horizontal, 16)
    }
}

#Preview {
    NavigationStack {
        EditorView().environmentObject(ProjectStore())
    }
}
