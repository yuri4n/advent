#![cfg(unix)]

use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

#[test]
fn scala_build_handles_an_unset_or_enabled_offline_option() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir(temp.path().join("scala")).unwrap();
    let compiler = temp.path().join("scala-cli");
    fs::write(
        &compiler,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$SCALA_ARGS\"\n",
    )
    .unwrap();
    fs::set_permissions(&compiler, fs::Permissions::from_mode(0o755)).unwrap();
    let arguments = temp.path().join("arguments");
    let adapter = Path::new(env!("CARGO_MANIFEST_DIR")).join("../scripts/languages/scala.sh");

    for offline in [false, true] {
        let mut command = Command::new("bash");
        command
            .arg(&adapter)
            .args(["build", "2024"])
            .env("AOC_ROOT", temp.path())
            .env("SCALA_CLI", &compiler)
            .env("SCALA_ARGS", &arguments)
            .env("AOC_JAVA", "/usr/bin/true")
            .env_remove("AOC_OFFLINE");
        if offline {
            command.env("AOC_OFFLINE", "1");
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "offline={offline}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let arguments = fs::read_to_string(&arguments).unwrap();
        assert!(!arguments.lines().any(str::is_empty));
        assert_eq!(arguments.lines().any(|arg| arg == "--offline"), offline);
        assert!(arguments
            .lines()
            .any(|arg| arg == "--coursier-validate-checksums"));
    }
}
