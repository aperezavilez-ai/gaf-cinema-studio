import SwiftUI

/// Device tier indicator — Phase 5.
struct DeviceStatusBar: View {
    let tier: String
    let thermal: String
    let previewQuality: String

    var body: some View {
        HStack(spacing: 16) {
            StatusChip(icon: "iphone", label: tier)
            StatusChip(icon: "thermometer.medium", label: thermal)
            StatusChip(icon: "sparkles.rectangle.stack", label: previewQuality)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 6)
    }
}

struct StatusChip: View {
    let icon: String
    let label: String

    var body: some View {
        HStack(spacing: 4) {
            Image(systemName: icon)
                .font(.system(size: 10))
            Text(label.uppercased())
                .font(.system(size: 9, weight: .medium))
                .tracking(1)
        }
        .foregroundStyle(.white.opacity(0.45))
    }
}

#Preview {
    DeviceStatusBar(tier: "Mid", thermal: "Normal", previewQuality: "Medium")
        .background(Color.black)
}
