use std::{ffi::OsString, path::PathBuf};

use super::resolve_log_dir;

#[test]
fn logging_creates_file_in_selected_directory() {
    if let Some(expected) = std::env::var_os("SHOJI_LOG_PATH_TEST_EXPECTED") {
        let expected = PathBuf::from(expected);
        assert_eq!(super::shoji_log_dir(), expected);
        let guard = super::init_logging(&super::CommonArgs::default()).unwrap();
        drop(guard);
        assert!(expected.join("latest.log").is_file());
        return;
    }

    let root = std::env::temp_dir().join(format!(
        "shoji-log-path-{}-{}",
        std::process::id(),
        super::startup_timestamp_millis()
    ));
    std::fs::create_dir(&root).unwrap();
    let state_home = root.join("state");
    for override_dir in [None, Some(root.join("override"))] {
        let expected = override_dir
            .clone()
            .unwrap_or_else(|| state_home.join("shoji_wm/logs"));
        let mut command = std::process::Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "log_path_tests::logging_creates_file_in_selected_directory",
                "--nocapture",
            ])
            .env("SHOJI_LOG_PATH_TEST_EXPECTED", &expected)
            .env("XDG_STATE_HOME", &state_home)
            .env("HOME", root.join("home"))
            .env_remove("SHOJI_LOG_DIR");
        if let Some(override_dir) = override_dir {
            command.env("SHOJI_LOG_DIR", override_dir);
        }
        assert!(command.status().unwrap().success());
        assert!(expected.join("latest.log").is_file());
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn session_logs_follow_xdg_and_explicit_override() {
    let cases = [
        (
            None,
            None,
            Some("/home/test"),
            "/home/test/.local/state/shoji_wm/logs",
        ),
        (
            None,
            Some("/state"),
            Some("/home/test"),
            "/state/shoji_wm/logs",
        ),
        (Some("/logs"), Some("/state"), Some("/home/test"), "/logs"),
        (
            Some(""),
            Some("/state"),
            Some("/home/test"),
            "/state/shoji_wm/logs",
        ),
        (
            None,
            Some(""),
            Some("/home/test"),
            "/home/test/.local/state/shoji_wm/logs",
        ),
        (
            None,
            Some("relative"),
            Some("/home/test"),
            "/home/test/.local/state/shoji_wm/logs",
        ),
        (
            Some("relative-logs"),
            None,
            Some("/home/test"),
            "relative-logs",
        ),
        (None, None, None, "./.local/state/shoji_wm/logs"),
    ];
    let actual: Vec<_> = cases
        .iter()
        .map(|(override_dir, state_home, home, _)| {
            resolve_log_dir(
                override_dir.map(OsString::from),
                state_home.map(OsString::from),
                home.map(OsString::from),
            )
        })
        .collect();
    let expected: Vec<_> = cases
        .iter()
        .map(|(_, _, _, expected)| PathBuf::from(expected))
        .collect();
    assert_eq!(actual, expected);
}
