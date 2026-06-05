//! Phase 13: Infrastructure scaffold — cloud providers, build-first connect-later.

use cinemastudio_engine::{
    default_project_settings, CloudBackendKind, CloudService, ProjectStateManager,
};
use tempfile::TempDir;

#[test]
fn phase13_local_backend_default() {
    std::env::remove_var("CINEMASTUDIO_CLOUD_BACKEND");
    let tmp = TempDir::new().unwrap();
    let cloud = CloudService::new(tmp.path());
    let status = cloud.backend_status();
    assert_eq!(status.kind, CloudBackendKind::Local);
    assert!(status.wired);
}

#[test]
fn phase13_supabase_backend_not_wired() {
    std::env::set_var("CINEMASTUDIO_CLOUD_BACKEND", "supabase");
    let tmp = TempDir::new().unwrap();
    let cloud = CloudService::new(tmp.path());
    let status = cloud.backend_status();
    assert_eq!(status.kind, CloudBackendKind::Supabase);
    assert!(!status.wired);
    assert!(cloud.login("a@b.com", "x").is_err());
    std::env::remove_var("CINEMASTUDIO_CLOUD_BACKEND");
}

#[test]
fn phase13_manager_cloud_login_local() {
    let tmp = TempDir::new().unwrap();
    std::env::remove_var("CINEMASTUDIO_CLOUD_BACKEND");
    let mut manager = ProjectStateManager::with_data_root(tmp.path().to_path_buf());
    manager
        .create_project("Cloud", tmp.path(), default_project_settings())
        .unwrap();
    manager.cloud_login("user@cinemastudio.dev", "pass").unwrap();
    assert!(manager.auth_session().unwrap().logged_in);
}
