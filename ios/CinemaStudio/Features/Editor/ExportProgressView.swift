import SwiftUI

/// Export progress overlay — polls export status from store.
struct ExportProgressView: View {
    @EnvironmentObject var store: ProjectStore

    var body: some View {
        if store.isExporting || !store.exportStatus.isEmpty {
            VStack(spacing: 6) {
                if store.isExporting {
                    ProgressView(value: store.exportProgress)
                        .tint(.white)
                }
                Text(store.exportStatus)
                    .font(.caption)
                    .foregroundStyle(.white.opacity(0.6))
            }
            .padding(.horizontal, 16)
        }
    }
}
