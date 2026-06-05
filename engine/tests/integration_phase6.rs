//! Phase 6 integration: optional cloud, billing, telemetry, beta gates

use cinemastudio_engine::{
    default_project_settings, ProjectStateManager, SubscriptionTier,
};
use std::fs;
use tempfile::TempDir;
use uuid::Uuid;

#[test]
fn phase6_core_works_without_account() {
    let data = TempDir::new().unwrap();
    let project_root = TempDir::new().unwrap();
    let clip = project_root.path().join("clip.mp4");
    fs::write(&clip, b"video").unwrap();

    let mut manager = ProjectStateManager::with_data_root(data.path().to_path_buf());
    assert!(!manager.auth_session().unwrap().logged_in);

    manager
        .create_project("NoAccount", project_root.path(), default_project_settings())
        .unwrap();
    let asset = manager.import_media(&clip).unwrap();
    manager
        .add_clip_to_timeline(cinemastudio_engine::AddClipParams {
            media_id: asset.id,
            track_id: None,
            start_ms: None,
        })
        .unwrap();

    manager.save().unwrap();
    assert!(manager.state().unwrap().metadata.name == "NoAccount");
}

#[test]
fn phase6_cloud_backup_restore() {
    let data = TempDir::new().unwrap();
    let project_root = TempDir::new().unwrap();

    let mut manager = ProjectStateManager::with_data_root(data.path().to_path_buf());
    manager
        .create_project("Cloud", project_root.path(), default_project_settings())
        .unwrap();
    manager.save().unwrap();

    let record = manager.cloud_backup().unwrap();
    assert!(record.path.exists());
    assert!(record.size_bytes > 0);

    let restore_path = project_root.path().parent().unwrap().join("restored.csproj");
    manager
        .cloud_restore(&record.path, &restore_path)
        .unwrap();
    assert!(restore_path.join("project.json").exists());

    let mut manager2 = ProjectStateManager::with_data_root(data.path().to_path_buf());
    manager2.open_project(&restore_path).unwrap();
    assert_eq!(manager2.state().unwrap().metadata.name, "Cloud");
}

#[test]
fn phase6_auth_optional_login() {
    let data = TempDir::new().unwrap();
    let manager = ProjectStateManager::with_data_root(data.path().to_path_buf());

    let session = manager
        .cloud_login("beta@cinemastudio.dev", "secret")
        .unwrap();
    assert!(session.logged_in);
    assert!(manager.auth_session().unwrap().logged_in);

    manager.cloud_logout().unwrap();
    assert!(!manager.auth_session().unwrap().logged_in);
}

#[test]
fn phase6_billing_pro_stub() {
    let data = TempDir::new().unwrap();
    let manager = ProjectStateManager::with_data_root(data.path().to_path_buf());

    assert_eq!(
        manager.subscription_state().unwrap().tier,
        SubscriptionTier::Free
    );
    manager.activate_pro_subscription().unwrap();
    assert_eq!(
        manager.subscription_state().unwrap().tier,
        SubscriptionTier::Pro
    );
    manager.cancel_pro_subscription().unwrap();
    assert_eq!(
        manager.subscription_state().unwrap().tier,
        SubscriptionTier::Free
    );
}

#[test]
fn phase6_telemetry_crash_rate_gate() {
    let data = TempDir::new().unwrap();
    let mut manager = ProjectStateManager::with_data_root(data.path().to_path_buf());
    manager.set_telemetry(true).unwrap();

    for _ in 0..100 {
        manager.start_telemetry_session().unwrap();
        manager.end_telemetry_session(false).unwrap();
    }
    manager.start_telemetry_session().unwrap();
    manager.end_telemetry_session(true).unwrap();

    let rate = manager.telemetry_crash_rate().unwrap();
    assert!(rate < 0.01, "crash rate {rate} should be < 1%");
}

#[test]
fn phase6_beta_registry_gate() {
    let data = TempDir::new().unwrap();
    let project_root = TempDir::new().unwrap();

    let manager = ProjectStateManager::with_data_root(data.path().to_path_buf());

    for i in 0..10 {
        let mut m = ProjectStateManager::with_data_root(data.path().to_path_buf());
        let dir = project_root.path().join(format!("p{i}"));
        fs::create_dir_all(&dir).unwrap();
        m.create_project(&format!("Beta {i}"), &dir, default_project_settings())
            .unwrap();
        let reg = m.beta_mark_complete(&format!("beta_user_{i}")).unwrap();
        if i == 9 {
            assert!(reg.gate_met());
            assert_eq!(reg.count(), 10);
        }
    }

    let final_reg = manager.beta_registry().unwrap();
    assert!(final_reg.gate_met());
}

#[test]
fn phase6_backup_works_while_logged_out() {
    let data = TempDir::new().unwrap();
    let project_root = TempDir::new().unwrap();

    let mut manager = ProjectStateManager::with_data_root(data.path().to_path_buf());
    manager
        .create_project("LocalCloud", project_root.path(), default_project_settings())
        .unwrap();

    assert!(!manager.auth_session().unwrap().logged_in);
    let record = manager.cloud_backup().unwrap();
    assert!(record.backup_id != Uuid::nil());
}
