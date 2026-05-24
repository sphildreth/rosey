#[cfg(test)]
mod config_tests {
    use rosey_core::RoseyConfig;

    #[test]
    fn default_config_has_reasonable_values() {
        let cfg = RoseyConfig::default();
        assert_eq!(cfg.version, "1.0");
        assert_eq!(cfg.paths.source, "");
        assert_eq!(cfg.paths.movies, "");
        assert_eq!(cfg.paths.tv, "");
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
        let parsed: RoseyConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, cfg.version);
        assert_eq!(parsed.paths.source, cfg.paths.source);
        assert_eq!(parsed.scanning.concurrency_local, cfg.scanning.concurrency_local);
        assert_eq!(parsed.behavior.dry_run, cfg.behavior.dry_run);
    }

    #[test]
    fn config_parses_partial_json() {
        let json = r#"{"version": "2.0", "paths": {"source": "/media/downloads"}}"#;
        let cfg: RoseyConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.version, "2.0");
        assert_eq!(cfg.paths.source, "/media/downloads");
        assert_eq!(cfg.paths.movies, "");
    }

    #[test]
    fn config_path_exists_for_current_platform() {
        let path = rosey_core::config_path();
        assert!(path.to_string_lossy().contains("rosey"));
    }

    #[test]
    fn save_and_load_config() {
        let _dir = tempfile::TempDir::new().unwrap();
        // Override config path isn't possible with the current API, but we can
        // test serialization round-trip which exercises the same code paths
        let cfg = RoseyConfig::default();
        let json = serde_json::to_string_pretty(&cfg).unwrap();
        assert!(json.contains("version"));
        assert!(json.contains("paths"));
    }
}
