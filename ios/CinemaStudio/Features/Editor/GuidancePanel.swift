import SwiftUI

/// AI guidance panel — orchestrator suggestions, not chat (Rule #5).
struct GuidancePanel: View {
    @EnvironmentObject var store: ProjectStore
    @State private var suggestions: [AiSuggestionItem] = []

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "sparkles")
                    .foregroundStyle(.white.opacity(0.6))
                Text("GUÍA CINEMATOGRÁFICA")
                    .font(.caption)
                    .tracking(2)
                    .foregroundStyle(.white.opacity(0.5))
            }

            if suggestions.isEmpty {
                Text("Analizando proyecto…")
                    .font(.subheadline)
                    .foregroundStyle(.white.opacity(0.35))
            } else {
                ForEach(suggestions) { item in
                    SuggestionCard(item: item) {
                        store.executeSuggestion(id: item.id)
                    } onDismiss: {
                        store.dismissSuggestion(id: item.id)
                    }
                }
            }
        }
        .padding(16)
        .background(Color.white.opacity(0.04))
        .cornerRadius(8)
        .padding(.horizontal, 16)
        .onAppear { store.loadSuggestions { suggestions = $0 } }
    }
}

struct AiSuggestionItem: Identifiable {
    let id: UUID
    let message: String
    let priority: String
    let actionLabel: String
    let isActionable: Bool
}

struct SuggestionCard: View {
    let item: AiSuggestionItem
    let onExecute: () -> Void
    let onDismiss: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(item.message)
                .font(.subheadline)
                .foregroundStyle(.white.opacity(0.85))

            HStack {
                if item.isActionable {
                    Button("Ejecutar", action: onExecute)
                        .font(.caption.bold())
                        .foregroundStyle(.black)
                        .padding(.horizontal, 12)
                        .padding(.vertical, 6)
                        .background(Color.white)
                        .cornerRadius(4)
                }
                Spacer()
                Button(action: onDismiss) {
                    Image(systemName: "xmark")
                        .font(.caption)
                        .foregroundStyle(.white.opacity(0.4))
                }
            }
        }
        .padding(12)
        .background(Color.white.opacity(0.06))
        .cornerRadius(6)
    }
}

#Preview {
    GuidancePanel()
        .environmentObject(ProjectStore())
        .background(Color.black)
}
