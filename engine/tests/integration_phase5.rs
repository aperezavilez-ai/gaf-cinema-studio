//! Phase 5 integration: device tiers, thermal adapt, scheduler

use cinemastudio_engine::{
    default_project_settings, parse_hints_json, DeviceTier, PreviewQuality, ProjectStateManager,
    ThermalLevel,
};
use std::fs;
use tempfile::TempDir;

#[test]
fn phase5_low_mid_high_policies() {
    let low = parse_hints_json(r#"{"totalRamMb":3072,"availableRamMb":2048,"gpuTier":"basic"}"#).unwrap();
    let mid = parse_hints_json(r#"{"totalRamMb":6144,"availableRamMb":4096,"gpuTier":"mid"}"#).unwrap();
    let high = parse_hints_json(r#"{"totalRamMb":12288,"availableRamMb":8000,"gpuTier":"flagship"}"#).unwrap();

    assert_eq!(low.tier, DeviceTier::Low);
    assert_eq!(mid.tier, DeviceTier::Mid);
    assert_eq!(high.tier, DeviceTier::High);

    let tmp = TempDir::new().unwrap();
    let mut manager = ProjectStateManager::new();
    manager.create_project("Perf", tmp.path(), default_project_settings()).unwrap();

    manager.set_device_profile(low.clone()).unwrap();
    assert_eq!(manager.quality_policy().max_proxy_height, 720);

    manager.set_device_profile(mid).unwrap();
    assert_eq!(manager.quality_policy().max_proxy_height, 1080);

    manager.set_device_profile(high).unwrap();
    assert!(manager.quality_policy().max_playback_fps >= 30.0);
}

#[test]
fn phase5_thermal_throttle_degrades() {
    let tmp = TempDir::new().unwrap();
    let mut manager = ProjectStateManager::new();
    manager.create_project("Thermal", tmp.path(), default_project_settings()).unwrap();

    manager.set_thermal_level(ThermalLevel::Critical).unwrap();
    assert!(!manager.quality_policy().allow_export);
    assert_eq!(
        manager.state().unwrap().render_state.preview_quality,
        PreviewQuality::Low
    );
}

#[test]
fn phase5_crash_report_opt_in() {
    let tmp = TempDir::new().unwrap();
    let mut manager = ProjectStateManager::new();
    manager.create_project("Crash", tmp.path(), default_project_settings()).unwrap();

    assert!(manager.record_crash("test", "unit").unwrap().is_none());

    manager.set_crash_reporting(true);
    let path = manager.record_crash("simulated", "test").unwrap();
    assert!(path.is_some());
    assert!(path.unwrap().exists());
}

#[test]
fn phase5_session_metrics_stable() {
    let tmp = TempDir::new().unwrap();
    let clip = tmp.path().join("c.mp4");
    fs::write(&clip, b"v").unwrap();

    let mut manager = ProjectStateManager::new();
    manager.create_project("Metrics", tmp.path(), default_project_settings()).unwrap();
    manager.import_media(&clip).unwrap();

    for _ in 0..10 {
        let _ = manager.playback_tick();
    }

    let report = manager.performance_report();
    assert!(report.get("deviceTier").is_some());
}
