use std::fs;
use std::path::Path;
use std::path::PathBuf;

use ntix_rs::lock::lock_file::LockFile;

mod common;

fn temp_lock_path(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(common::unique_tag(&format!("lock_{tag}")));
    fs::create_dir_all(&dir).unwrap();
    dir.join("test.lock")
}

fn cleanup(dir: &Path) {
    if let Some(parent) = dir.parent() {
        let _ = fs::remove_dir_all(parent);
    }
}

#[test]
fn lock_file_create_and_drop_removes_file() {
    let path = temp_lock_path("create");
    {
        let lock = LockFile::acquire_at(path.clone()).unwrap();
        assert!(path.is_file());
        drop(lock);
    }
    assert!(!path.exists());
    cleanup(&path);
}

#[test]
fn lock_file_existing_stale_file_is_overwritten() {
    let path = temp_lock_path("stale");
    fs::write(&path, "1234@9999999999").unwrap();

    {
        let lock = LockFile::acquire_at(path.clone()).unwrap();
        assert!(path.is_file());
        drop(lock);
    }
    assert!(!path.exists());
    cleanup(&path);
}

#[test]
fn lock_file_second_lock_throws() {
    let path = temp_lock_path("second");
    let lock1 = LockFile::acquire_at(path.clone()).unwrap();
    let second = LockFile::acquire_at(path.clone());
    assert!(second.is_err());
    drop(lock1);
    cleanup(&path);
}

#[test]
fn lock_file_dropped_lock_can_be_reacquired() {
    let path = temp_lock_path("reacquire");
    let lock1 = LockFile::acquire_at(path.clone()).unwrap();
    drop(lock1);

    let lock2 = LockFile::acquire_at(path.clone()).unwrap();
    assert!(path.is_file());
    drop(lock2);
    cleanup(&path);
}

#[test]
fn get_default_lock_path_requires_localappdata() {
    // On a machine without LOCALAPPDATA (e.g. non-Windows), this returns Err.
    match std::env::var("LOCALAPPDATA") {
        Ok(_) => {
            let path = ntix_rs::lock::lock_file::get_default_lock_path().unwrap();
            let s = path.to_string_lossy();
            assert!(s.contains("ntix"));
            assert!(s.ends_with("apply.lock"));
        }
        Err(_) => {
            assert!(ntix_rs::lock::lock_file::get_default_lock_path().is_err());
        }
    }
}
