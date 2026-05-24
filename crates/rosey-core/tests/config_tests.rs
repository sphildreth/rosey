#[cfg(test)]
mod config_tests {
    use rosey_core::RoseyConfig;
    use std::ffi::OsString;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    #[cfg(not(windows))]
    fn config_env_key() -> &'static str {
        "XDG_CONFIG_HOME"
    }

    #[cfg(windows)]
    fn config_env_key() -> &'static str {
        "APPDATA"
    }

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match self.previous.take() {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    fn set_env_var(key: &'static str, value: &std::path::Path) -> EnvVarGuard {
        let previous = std::env::var_os(key);
        std::env::set_var(key, value);
        EnvVarGuard { key, previous }
    }

    #[test]
    fn default_config_has_reasonable_values() {
        let cfg = RoseyConfig::default();
        assert_eq!(cfg.version, "1.0");
        assert_eq!(cfg.paths.source, "");
        assert_eq!(cfg.paths.movies, "");
        assert_eq!(cfg.paths.tv, "");
        assert_eq!(cfg.ui.theme, "system");
        assert_eq!(cfg.ui.window.width, 1200);
        assert_eq!(cfg.ui.window.height, 800);
        assert!(!cfg.ui.window.maximized);
        assert_eq!(cfg.ui.splitters.main, vec![300, 900]);
        assert_eq!(cfg.ui.splitters.vertical, vec![600, 200]);
        assert!(cfg.behavior.dry_run);
        assert!(cfg.behavior.auto_select_green);
        assert_eq!(cfg.behavior.conflict_policy, "ask");
        assert_eq!(cfg.scanning.concurrency_local, 8);
        assert_eq!(cfg.scanning.concurrency_network, 2);
        assert!(!cfg.scanning.follow_symlinks);
        assert!(!cfg.scanning.enforce_one_media_per_folder);
        assert_eq!(cfg.identification.confidence_thresholds.green, 70);
        assert_eq!(cfg.identification.confidence_thresholds.yellow, 40);
        assert!(cfg.identification.prefer_nfo_ids);
        assert_eq!(cfg.providers.cache_ttl_days, 30);
        assert_eq!(cfg.logging.level, "INFO");
        assert!(cfg.logging.redact_secrets);
    }

    #[test]
    fn config_serialization_round_trip() {
        let cfg = RoseyConfig::default();
        let json = serde_json::to_string_pretty(&cfg).unwrap();
        assert!(json.contains("\"ui\""));
        let parsed: RoseyConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, cfg.version);
        assert_eq!(parsed.paths.source, cfg.paths.source);
        assert_eq!(parsed.ui.theme, cfg.ui.theme);
        assert_eq!(parsed.scanning.concurrency_local, cfg.scanning.concurrency_local);
        assert_eq!(parsed.behavior.dry_run, cfg.behavior.dry_run);
    }

    #[test]
    fn config_parses_partial_json() {
        let json = r#"{
            "version": "2.0",
            "paths": {"source": "/media/downloads"},
            "ui": {
                "theme": "dark",
                "window": {"width": 1440, "maximized": true},
                "splitters": {"main": [400, 800]}
            }
        }"#;
        let cfg: RoseyConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.version, "2.0");
        assert_eq!(cfg.paths.source, "/media/downloads");
        assert_eq!(cfg.paths.movies, "");
        assert_eq!(cfg.ui.theme, "dark");
        assert_eq!(cfg.ui.window.width, 1440);
        assert_eq!(cfg.ui.window.height, 800);
        assert!(cfg.ui.window.maximized);
        assert_eq!(cfg.ui.splitters.main, vec![400, 800]);
        assert_eq!(cfg.ui.splitters.vertical, vec![600, 200]);
    }

    #[test]
    fn config_path_exists_for_current_platform() {
        let path = rosey_core::config_path();
        assert!(path.to_string_lossy().contains("rosey"));
    }

    #[test]
    fn load_config_reads_existing_file() {
        let _guard = env_lock();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let key = config_env_key();
        let _env = set_env_var(key, temp_dir.path());

        let config_dir = temp_dir.path().join("rosey");
        std::fs::create_dir_all(&config_dir).unwrap();
        let config_file = config_dir.join("rosey.json");
        let json = r#"{
            "version": "2.0",
            "paths": {"source": "/media/downloads", "movies": "/media/movies", "tv": "/media/tv"},
            "behavior": {"dry_run": false, "conflict_policy": "replace"},
            "scanning": {"concurrency_local": 16},
            "identification": {"confidence_thresholds": {"green": 80, "yellow": 55}}
        }"#;
        std::fs::write(&config_file, json).unwrap();

        let cfg = rosey_core::load_config();
        assert_eq!(cfg.version, "2.0");
        assert_eq!(cfg.paths.source, "/media/downloads");
        assert_eq!(cfg.paths.movies, "/media/movies");
        assert_eq!(cfg.paths.tv, "/media/tv");
        assert!(!cfg.behavior.dry_run);
        assert_eq!(cfg.behavior.conflict_policy, "replace");
        assert_eq!(cfg.scanning.concurrency_local, 16);
        assert_eq!(cfg.identification.confidence_thresholds.green, 80);
        assert_eq!(cfg.identification.confidence_thresholds.yellow, 55);
    }
}
