use cinemastudio_engine::{default_project_settings, ProjectStateManager, Mutation};
use std::fs;
use tempfile::TempDir;

#[test]
fn integration_full_project_lifecycle() {
    let tmp = TempDir::new().unwrap();
    let mut manager = ProjectStateManager::new();

    // Create
    manager
        .create_project("Integration Film", tmp.path(), default_project_settings())
        .unwrap();

    let path = manager.project_path().unwrap().to_path_buf();

    // Import media
    let clip = tmp.path().join("scene01.mp4");
    fs::write(&clip, b"scene content").unwrap();
    manager.import_media(&clip).unwrap();

    // Mutate + save
    manager
        .apply(Mutation::RenameProject {
            name: "Director's Cut".into(),
        })
        .unwrap();
    manager.save().unwrap();

    // Autosave
    let snap = manager.tick_autosave().unwrap();
    assert!(snap.is_some());

    // Reopen
    let mut manager2 = ProjectStateManager::new();
    manager2.open_project(&path).unwrap();
    assert_eq!(manager2.state().unwrap().metadata.name, "Director's Cut");
    assert_eq!(manager2.state().unwrap().media.len(), 1);
}

#[test]
fn integration_recovery_pipeline() {
    let tmp = TempDir::new().unwrap();
    let mut manager = ProjectStateManager::new();
    manager
        .create_project("Recovery Film", tmp.path(), default_project_settings())
        .unwrap();
    manager.save().unwrap();

    let path = manager.project_path().unwrap().to_path_buf();

    // Simulate crash: corrupt JSON
    fs::write(path.join("project.json"), b"{ BAD JSON").unwrap();

    let mut recovered = ProjectStateManager::new();
    recovered.open_project(&path).unwrap();
    assert_eq!(recovered.state().unwrap().metadata.name, "Recovery Film");
}
