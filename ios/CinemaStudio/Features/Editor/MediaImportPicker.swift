import PhotosUI
import SwiftUI

/// Import video from photo library into current project.
struct MediaImportPicker: View {
    @EnvironmentObject var store: ProjectStore
    @State private var pickerItem: PhotosPickerItem?

    var body: some View {
        PhotosPicker(selection: $pickerItem, matching: .videos) {
            Label("Import media", systemImage: "square.and.arrow.down")
                .frame(maxWidth: .infinity)
        }
        .buttonStyle(CinematicButtonStyle(outlined: true))
        .onChange(of: pickerItem) { _, item in
            guard let item else { return }
            Task { await importVideo(item) }
        }
    }

    private func importVideo(_ item: PhotosPickerItem) async {
        guard let project = store.currentProject else { return }
        guard let movie = try? await item.loadTransferable(type: VideoFile.self) else { return }
        store.importMedia(from: movie.url.path, into: project.path)
    }
}

private struct VideoFile: Transferable {
    let url: URL

    static var transferRepresentation: some TransferRepresentation {
        FileRepresentation(contentType: .movie) { video in
            SentTransferredFile(video.url)
        } importing: { received in
            let dest = FileManager.default.temporaryDirectory.appendingPathComponent(received.file.lastPathComponent)
            if FileManager.default.fileExists(atPath: dest.path) {
                try FileManager.default.removeItem(at: dest)
            }
            try FileManager.default.copyItem(at: received.file, to: dest)
            return VideoFile(url: dest)
        }
    }
}
