use ntix_rs::paths::local_app_data_path;

#[test]
fn local_app_data_resolves_and_errors_based_on_localappdata() {
    unsafe {
        let base = format!("C:\\Users\\tester\\AppData\\Local-{}", std::process::id());
        std::env::set_var("LOCALAPPDATA", &base);
        let resolved = local_app_data_path().expect("should resolve with LOCALAPPDATA set");
        assert_eq!(resolved, std::path::PathBuf::from(&base).join("ntix"));

        std::env::remove_var("LOCALAPPDATA");
        assert!(local_app_data_path().is_err());
    }
}
