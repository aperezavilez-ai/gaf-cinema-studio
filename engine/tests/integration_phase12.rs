//! Phase 12: MVP release gates — beta cohort, crash rate, ship readiness

use cinemastudio_engine::{
    default_project_settings, evaluate_mvp_readiness, BetaRegistry, BetaCompletion,
    ProjectStateManager,
};
use chrono::Utc;
use tempfile::TempDir;
use uuid::Uuid;

fn full_beta_registry() -> BetaRegistry {
    let mut reg = BetaRegistry::new();
    for i in 0..10 {
        reg.completions.push(BetaCompletion {
            project_id: Uuid::new_v4(),
            project_name: format!("Beta Film {i}"),
            user_label: format!("tester_{i}"),
            completed_at: Utc::now(),
        });
    }
    reg
}

#[test]
fn phase12_mvp_ready_when_gates_pass() {
    let report = evaluate_mvp_readiness(&full_beta_registry(), 0.005, "1.0.0");
    assert!(report.beta_gate_met);
    assert!(report.crash_gate_met);
    assert!(report.ready_to_ship);
}

#[test]
fn phase12_blocked_without_beta_cohort() {
    let report = evaluate_mvp_readiness(&BetaRegistry::new(), 0.0, "1.0.0");
    assert!(!report.ready_to_ship);
    assert!(!report.blockers.is_empty());
}

#[test]
fn phase12_ship_project_marks_beta() {
    let tmp = TempDir::new().unwrap();
    let clip = tmp.path().join("clip.mp4");
    std::fs::write(&clip, b"video").unwrap();

    let mut manager = ProjectStateManager::with_data_root(tmp.path().to_path_buf());
    manager
        .create_project("Ship", tmp.path(), default_project_settings())
        .unwrap();
    manager.import_media(&clip).unwrap();

    let report = manager.ship_project("beta_tester_1").unwrap();
    assert_eq!(report.beta_completions, 1);
    assert_eq!(
        manager.state().unwrap().workflow_state.phase,
        cinemastudio_engine::project_state::types::WorkflowPhase::Complete
    );
}

#[test]
fn phase12_version_is_1_0_0() {
    assert_eq!(ProjectStateManager::MVP_VERSION, "1.0.0");
}
