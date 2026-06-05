//! MVP release readiness — aggregates gates 6.3, 6.4, 11, 12 for ship decision.

use serde::{Deserialize, Serialize};

use crate::beta::BetaRegistry;
use crate::error::Result;
use crate::render_pipeline::ffmpeg_available;
use crate::telemetry::TelemetryService;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MvpReadinessReport {
    pub ready_to_ship: bool,
    pub version: String,
    pub beta_completions: usize,
    pub beta_target: u32,
    pub beta_gate_met: bool,
    pub crash_rate: f64,
    pub crash_gate_met: bool,
    pub ffmpeg_available: bool,
    pub blockers: Vec<String>,
    pub recommendations: Vec<String>,
}

pub fn evaluate(
    beta: &BetaRegistry,
    crash_rate: f64,
    version: &str,
) -> MvpReadinessReport {
    let beta_gate = beta.gate_met();
    let crash_gate = crash_rate < 0.01;
    let ffmpeg = ffmpeg_available();

    let mut blockers = Vec::new();
    let mut recommendations = Vec::new();

    if !beta_gate {
        blockers.push(format!(
            "Beta gate: {}/{} projects complete (need 10)",
            beta.count(),
            beta.target
        ));
    }
    if !crash_gate {
        blockers.push(format!(
            "Crash rate {:.2}% exceeds 1% threshold",
            crash_rate * 100.0
        ));
    }
    if !ffmpeg {
        recommendations.push("Install FFmpeg for production H.264 export on server/desktop CI".into());
    }

    recommendations.push("Upload to TestFlight / Play Internal Testing for human beta cohort".into());
    recommendations.push("Link Rust engine (CINEMASTUDIO_ENGINE_LINKED) for on-device state sync".into());

    let ready = beta_gate && crash_gate;

    MvpReadinessReport {
        ready_to_ship: ready,
        version: version.to_string(),
        beta_completions: beta.count(),
        beta_target: beta.target,
        beta_gate_met: beta_gate,
        crash_rate,
        crash_gate_met: crash_gate,
        ffmpeg_available: ffmpeg,
        blockers,
        recommendations,
    }
}

pub fn evaluate_from_services(
    beta: &BetaRegistry,
    telemetry: &TelemetryService,
    version: &str,
) -> Result<MvpReadinessReport> {
    let rate = telemetry.crash_rate()?;
    Ok(evaluate(beta, rate, version))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beta::BetaCompletion;
    use chrono::Utc;
    use uuid::Uuid;

    fn full_beta_registry() -> BetaRegistry {
        let mut reg = BetaRegistry::new();
        for i in 0..10 {
            reg.completions.push(BetaCompletion {
                project_id: Uuid::new_v4(),
                project_name: format!("Film {i}"),
                user_label: format!("beta_{i}"),
                completed_at: Utc::now(),
            });
        }
        reg
    }

    #[test]
    fn ready_when_beta_and_crash_gates_pass() {
        let report = evaluate(&full_beta_registry(), 0.005, "1.0.0");
        assert!(report.beta_gate_met);
        assert!(report.crash_gate_met);
        assert!(report.ready_to_ship);
        assert!(report.blockers.is_empty());
    }

    #[test]
    fn blocked_when_beta_incomplete() {
        let report = evaluate(&BetaRegistry::new(), 0.0, "1.0.0");
        assert!(!report.ready_to_ship);
        assert!(!report.blockers.is_empty());
    }
}
