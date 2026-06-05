use crate::error::{CinemaError, Result};
use crate::project_state::types::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub field: String,
    pub message: String,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        !self
            .issues
            .iter()
            .any(|i| i.severity == ValidationSeverity::Error)
    }

    pub fn errors(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Error)
    }
}

pub fn validate_project_state(state: &ProjectState) -> ValidationReport {
    let mut issues = Vec::new();

    if state.schema_version != SCHEMA_VERSION {
        issues.push(ValidationIssue {
            field: "schema_version".into(),
            message: format!(
                "expected schema version {}, got {}",
                SCHEMA_VERSION, state.schema_version
            ),
            severity: ValidationSeverity::Error,
        });
    }

    if state.metadata.name.trim().is_empty() {
        issues.push(ValidationIssue {
            field: "metadata.name".into(),
            message: "project name cannot be empty".into(),
            severity: ValidationSeverity::Error,
        });
    }

    if state.metadata.resolution.width < 640 || state.metadata.resolution.height < 360 {
        issues.push(ValidationIssue {
            field: "metadata.resolution".into(),
            message: "resolution below minimum (640x360)".into(),
            severity: ValidationSeverity::Error,
        });
    }

    if state.storage_state.project_root.trim().is_empty() {
        issues.push(ValidationIssue {
            field: "storage_state.project_root".into(),
            message: "project root cannot be empty".into(),
            severity: ValidationSeverity::Error,
        });
    }

    let media_ids: std::collections::HashSet<_> = state.media.iter().map(|m| m.id).collect();
    if media_ids.len() != state.media.len() {
        issues.push(ValidationIssue {
            field: "media".into(),
            message: "duplicate media IDs detected".into(),
            severity: ValidationSeverity::Error,
        });
    }

    for track in &state.timeline.tracks {
        for clip in &track.clips {
            if !media_ids.contains(&clip.media_id) {
                issues.push(ValidationIssue {
                    field: format!("timeline.tracks.{}.clips.{}", track.id, clip.id),
                    message: format!("clip references unknown media_id {}", clip.media_id),
                    severity: ValidationSeverity::Error,
                });
            }

            if clip.source_out_ms <= clip.source_in_ms {
                issues.push(ValidationIssue {
                    field: format!("timeline.tracks.{}.clips.{}", track.id, clip.id),
                    message: "source_out_ms must be greater than source_in_ms".into(),
                    severity: ValidationSeverity::Error,
                });
            }

            if clip.duration_ms == 0 {
                issues.push(ValidationIssue {
                    field: format!("timeline.tracks.{}.clips.{}", track.id, clip.id),
                    message: "clip duration must be greater than 0".into(),
                    severity: ValidationSeverity::Error,
                });
            }
        }
    }

    if state.timeline.playhead_ms > state.timeline.duration_ms && state.timeline.duration_ms > 0 {
        issues.push(ValidationIssue {
            field: "timeline.playhead_ms".into(),
            message: "playhead exceeds timeline duration".into(),
            severity: ValidationSeverity::Warning,
        });
    }

    ValidationReport { issues }
}

pub fn ensure_valid(state: &ProjectState) -> Result<()> {
    let report = validate_project_state(state);
    if report.is_valid() {
        Ok(())
    } else {
        let messages: Vec<String> = report
            .errors()
            .map(|e| format!("{}: {}", e.field, e.message))
            .collect();
        Err(CinemaError::Validation(messages.join("; ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn valid_empty_project_passes() {
        let state = ProjectState::new(
            "Test Film",
            "/projects/test.csproj",
            default_project_settings(),
        );
        let report = validate_project_state(&state);
        assert!(report.is_valid());
    }

    #[test]
    fn empty_name_fails() {
        let mut state = ProjectState::new("", "/projects/test.csproj", default_project_settings());
        state.metadata.name = "   ".into();
        let report = validate_project_state(&state);
        assert!(!report.is_valid());
    }

    #[test]
    fn orphan_clip_reference_fails() {
        let mut state = ProjectState::new(
            "Test",
            "/projects/test.csproj",
            default_project_settings(),
        );
        state.timeline.tracks[0].clips.push(Clip {
            id: Uuid::new_v4(),
            media_id: Uuid::new_v4(),
            start_ms: 0,
            duration_ms: 1000,
            source_in_ms: 0,
            source_out_ms: 1000,
            label: String::new(),
            transitions: ClipTransitions::default(),
        });
        let report = validate_project_state(&state);
        assert!(!report.is_valid());
    }
}
