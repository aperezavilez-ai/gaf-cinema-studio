//! Rule-based suggestion engine — 100% offline, context from ProjectState.

use chrono::Utc;
use uuid::Uuid;

use crate::ai_orchestrator::actions::{
    ACTION_ADVANCE_TO_EDIT, ACTION_ADVANCE_TO_EXPORT, ACTION_APPLY_DEFAULT_FADES,
    ACTION_ORGANIZE_BY_ORDER, ACTION_ROUGH_CUT, ACTION_START_EXPORT,
};
use crate::project_state::types::{
    AiPriority, AiSuggestion, AiSuggestionType, MediaStatus, ProjectState, TrackType,
    WorkflowPhase,
};

const LONG_CLIP_MS: u64 = 60_000;

pub fn generate_suggestions(state: &ProjectState) -> Vec<AiSuggestion> {
    let mut suggestions = Vec::new();

    rule_no_media(state, &mut suggestions);
    rule_media_not_on_timeline(state, &mut suggestions);
    rule_no_audio(state, &mut suggestions);
    rule_long_clips(state, &mut suggestions);
    rule_ready_to_edit(state, &mut suggestions);
    rule_ready_to_export(state, &mut suggestions);
    rule_missing_proxy(state, &mut suggestions);
    rule_clips_without_fades(state, &mut suggestions);

    suggestions
}

fn rule_no_media(state: &ProjectState, out: &mut Vec<AiSuggestion>) {
    if !state.media.is_empty() {
        return;
    }
    out.push(make(
        AiSuggestionType::Workflow,
        AiPriority::High,
        "Importa tu primer clip para comenzar la producción.",
        "hint_import",
        serde_json::json!({}),
    ));
}

fn rule_media_not_on_timeline(state: &ProjectState, out: &mut Vec<AiSuggestion>) {
    let on_timeline: std::collections::HashSet<_> = state
        .timeline
        .tracks
        .iter()
        .flat_map(|t| t.clips.iter().map(|c| c.media_id))
        .collect();

    let pending: usize = state
        .media
        .iter()
        .filter(|m| m.status == MediaStatus::Ready && !on_timeline.contains(&m.id))
        .count();

    if pending == 0 {
        return;
    }

    out.push(make(
        AiSuggestionType::Organization,
        AiPriority::High,
        &format!(
            "Tienes {pending} clip(s) sin colocar en la timeline. ¿Crear un rough cut automático?",
        ),
        ACTION_ROUGH_CUT,
        serde_json::json!({ "pendingCount": pending }),
    ));
}

fn rule_no_audio(state: &ProjectState, out: &mut Vec<AiSuggestion>) {
    let has_video = state.timeline.tracks.iter().any(|t| {
        t.track_type == TrackType::Video && !t.clips.is_empty()
    });
    let has_audio = state.timeline.tracks.iter().any(|t| {
        t.track_type == TrackType::Audio && !t.clips.is_empty()
    });

    if has_video && !has_audio {
        out.push(make(
            AiSuggestionType::Audio,
            AiPriority::Medium,
            "Tu proyecto no tiene pista de audio. Considera importar audio o música.",
            "hint_audio",
            serde_json::json!({}),
        ));
    }
}

fn rule_long_clips(state: &ProjectState, out: &mut Vec<AiSuggestion>) {
    for track in &state.timeline.tracks {
        for clip in &track.clips {
            if clip.duration_ms > LONG_CLIP_MS {
                out.push(make(
                    AiSuggestionType::Edit,
                    AiPriority::Low,
                    &format!(
                        "El clip \"{}\" dura {:.0}s. Considera dividirlo para un ritmo más cinematográfico.",
                        clip.label,
                        clip.duration_ms as f64 / 1000.0
                    ),
                    "hint_split",
                    serde_json::json!({ "clipId": clip.id.to_string() }),
                ));
                break;
            }
        }
    }
}

fn rule_ready_to_edit(state: &ProjectState, out: &mut Vec<AiSuggestion>) {
    let clip_count: usize = state.timeline.tracks.iter().map(|t| t.clips.len()).sum();

    if clip_count == 0 {
        return;
    }

    if state.workflow_state.phase == WorkflowPhase::Import
        || state.workflow_state.phase == WorkflowPhase::Organize
    {
        out.push(make(
            AiSuggestionType::Workflow,
            AiPriority::Medium,
            &format!(
                "Timeline con {clip_count} clip(s) lista. Avanzar a fase de edición.",
            ),
            ACTION_ADVANCE_TO_EDIT,
            serde_json::json!({ "clipCount": clip_count }),
        ));
    }
}

fn rule_ready_to_export(state: &ProjectState, out: &mut Vec<AiSuggestion>) {
    let has_clips = state.timeline.tracks.iter().any(|t| !t.clips.is_empty());

    if !has_clips {
        return;
    }

    if matches!(
        state.workflow_state.phase,
        WorkflowPhase::Edit | WorkflowPhase::Review
    ) {
        out.push(make(
            AiSuggestionType::Export,
            AiPriority::High,
            "Tu proyecto está listo para exportar en 1080p.",
            ACTION_START_EXPORT,
            serde_json::json!({ "resolution": "1920x1080" }),
        ));
    }
}

fn rule_missing_proxy(state: &ProjectState, out: &mut Vec<AiSuggestion>) {
    let missing: usize = state
        .media
        .iter()
        .filter(|m| m.proxy_path.is_none() && m.status == MediaStatus::Ready)
        .count();

    if missing > 0 && state.timeline.tracks.iter().any(|t| !t.clips.is_empty()) {
        out.push(make(
            AiSuggestionType::Workflow,
            AiPriority::Low,
            &format!(
                "{missing} clip(s) aún sin proxy. La reproducción usará archivos originales.",
            ),
            "hint_proxy",
            serde_json::json!({ "count": missing }),
        ));
    }
}

fn rule_clips_without_fades(state: &ProjectState, out: &mut Vec<AiSuggestion>) {
    let no_fade: usize = state
        .timeline
        .tracks
        .iter()
        .flat_map(|t| &t.clips)
        .filter(|c| c.transitions.fade_in_ms == 0 && c.transitions.fade_out_ms == 0)
        .count();

    if no_fade >= 2 {
        out.push(make(
            AiSuggestionType::Edit,
            AiPriority::Low,
            &format!(
                "{no_fade} clips sin transiciones. Aplicar fades cinematográficos suaves?",
            ),
            ACTION_APPLY_DEFAULT_FADES,
            serde_json::json!({ "clipCount": no_fade }),
        ));
    }

    if state.timeline.tracks.iter().any(|t| t.clips.len() >= 3)
        && !out.iter().any(|s| s.action_id == ACTION_ORGANIZE_BY_ORDER)
    {
        out.push(make(
            AiSuggestionType::Organization,
            AiPriority::Medium,
            "Organizar clips en orden de importación para un flujo más claro.",
            ACTION_ORGANIZE_BY_ORDER,
            serde_json::json!({}),
        ));
    }
}

fn make(
    suggestion_type: AiSuggestionType,
    priority: AiPriority,
    message: &str,
    action_id: &str,
    payload: serde_json::Value,
) -> AiSuggestion {
    AiSuggestion {
        id: Uuid::new_v4(),
        suggestion_type,
        priority,
        message: message.to_string(),
        action_id: action_id.to_string(),
        payload,
        created_at: Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_state::types::{default_project_settings, ProjectState};

    #[test]
    fn suggests_import_when_empty() {
        let state = ProjectState::new("Empty", "/p", default_project_settings());
        let s = generate_suggestions(&state);
        assert!(s.iter().any(|x| x.action_id == "hint_import"));
    }

    #[test]
    fn suggests_rough_cut_when_media_off_timeline() {
        use chrono::Utc;
        let mut state = ProjectState::new("M", "/p", default_project_settings());
        state.media.push(crate::project_state::types::MediaAsset {
            id: Uuid::new_v4(),
            original_path: "/a.mp4".into(),
            proxy_path: None,
            thumbnail_path: None,
            file_name: "a.mp4".into(),
            mime_type: "video/mp4".into(),
            duration_ms: 3000,
            width: 1920,
            height: 1080,
            file_size_bytes: 1,
            status: MediaStatus::Ready,
            imported_at: Utc::now(),
            checksum: None,
        });
        let s = generate_suggestions(&state);
        assert!(s.iter().any(|x| x.action_id == ACTION_ROUGH_CUT));
    }

    #[test]
    fn no_generic_suggestions_when_complete() {
        let state = ProjectState::new("Empty", "/p", default_project_settings());
        for sug in generate_suggestions(&state) {
            assert!(!sug.message.is_empty());
            assert!(!sug.action_id.is_empty());
        }
    }
}
