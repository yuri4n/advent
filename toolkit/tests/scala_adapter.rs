#![cfg(unix)]

use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

#[test]
fn scala_scaffold_hides_benchmark_replay_and_preserves_existing_work() {
    let temp = tempfile::tempdir().unwrap();
    let adapter = Path::new(env!("CARGO_MANIFEST_DIR")).join("../scripts/languages/scala.sh");
    let create = || {
        Command::new("bash")
            .arg(&adapter)
            .args(["new", "2026", "1"])
            .env("AOC_ROOT", temp.path())
            .output()
            .unwrap()
    };
    assert!(create().status.success());
    let path = temp
        .path()
        .join("scala/src/main/scala/aoc/solutions/y2026/Day01.scala");
    let source = fs::read_to_string(&path).unwrap();
    assert!(source.contains("type Prepared = Stream[String, Sync]"));
    assert!(
        !source.contains("snapshot"),
        "replay is a runner implementation detail"
    );
    assert!(!create().status.success());
    assert_eq!(fs::read_to_string(path).unwrap(), source);
}

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
            .env("AOC_JAVA", "true")
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

#[test]
fn scala_build_reuses_unchanged_artifacts_and_invalidates_changed_inputs() {
    let temp = tempfile::Builder::new()
        .prefix("aoc cached build ")
        .tempdir()
        .unwrap();
    let project = temp.path().join("scala");
    fs::create_dir_all(project.join("src/main/resources")).unwrap();
    let source = project.join("Main.scala");
    fs::write(&source, "object Main").unwrap();
    let compiler = temp.path().join("scala-cli");
    fs::write(
        &compiler,
        r#"#!/bin/sh
set -eu
if [ "$1" = version ]; then printf '%s\n' "${COMPILER_VERSION:-1}"; exit 0; fi
printf 'build\n' >> "$SCALA_BUILDS"
while [ "$1" != --output ]; do shift; done
printf 'jar\n' > "$2"
exit "${BUILD_EXIT:-0}"
"#,
    )
    .unwrap();
    fs::set_permissions(&compiler, fs::Permissions::from_mode(0o755)).unwrap();
    let calls = temp.path().join("calls");
    let adapter = Path::new(env!("CARGO_MANIFEST_DIR")).join("../scripts/languages/scala.sh");
    let build = |version: &str, exit: &str| {
        Command::new("bash")
            .arg(&adapter)
            .args(["build", "2025"])
            .env("AOC_ROOT", temp.path())
            .env("SCALA_CLI", &compiler)
            .env("AOC_JAVA", "true")
            .env("SCALA_BUILDS", &calls)
            .env("COMPILER_VERSION", version)
            .env("BUILD_EXIT", exit)
            .output()
            .unwrap()
    };
    let count = || fs::read_to_string(&calls).unwrap().lines().count();
    assert!(build("1", "0").status.success());
    assert!(build("1", "0").status.success());
    assert_eq!(count(), 1, "unchanged sources must reuse the jar");
    fs::write(&source, "object Changed").unwrap();
    assert!(build("1", "0").status.success());
    assert_eq!(count(), 2);
    fs::write(project.join("Extra.scala"), "object Extra").unwrap();
    assert!(build("1", "0").status.success());
    fs::remove_file(project.join("Extra.scala")).unwrap();
    assert!(build("1", "0").status.success());
    assert_eq!(count(), 4, "added and deleted sources invalidate the build");
    fs::write(
        project.join("src/main/resources/settings.txt"),
        "new resource",
    )
    .unwrap();
    assert!(build("1", "0").status.success());
    assert!(build("2", "0").status.success());
    assert_eq!(count(), 6, "resources and toolchains invalidate the build");
    assert!(!build("3", "7").status.success());
    assert!(build("2", "0").status.success());
    assert_eq!(
        count(),
        8,
        "failed builds must invalidate their partial output"
    );
    fs::remove_file(temp.path().join(".aoc/build/scala/advent.jar")).unwrap();
    assert!(build("2", "0").status.success());
    assert_eq!(count(), 9);
}

#[test]
fn scala_editor_setup_records_an_executable_launcher_and_native_project() {
    let temp = tempfile::Builder::new()
        .prefix("aoc editor setup ")
        .tempdir()
        .unwrap();
    fs::create_dir(temp.path().join("scala")).unwrap();
    let compiler = temp.path().join("scala-cli wrapper");
    fs::write(
        &compiler,
        "#!/bin/sh\nprintf '%s\\n' \"$PWD\" \"$@\" > \"$SCALA_ARGS\"\nexit \"${SCALA_EXIT:-0}\"\n",
    )
    .unwrap();
    fs::set_permissions(&compiler, fs::Permissions::from_mode(0o755)).unwrap();
    let arguments = temp.path().join("arguments");
    let adapter = Path::new(env!("CARGO_MANIFEST_DIR")).join("../scripts/languages/scala.sh");

    for code in [0, 17] {
        let output = Command::new("bash")
            .arg(&adapter)
            .arg("editor-setup")
            .current_dir(temp.path())
            .env("AOC_ROOT", temp.path())
            .env("SCALA_CLI", "./scala-cli wrapper")
            .env("SCALA_ARGS", &arguments)
            .env("SCALA_EXIT", code.to_string())
            .env("AOC_OFFLINE", "1")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(code), "{:?}", output);
        let actual = fs::read_to_string(&arguments).unwrap();
        let args: Vec<_> = actual.lines().collect();
        assert_eq!(args[0], temp.path().join("scala").to_str().unwrap());
        assert_eq!(args[1..3], ["--power", "--initial-launcher-path"]);
        assert_eq!(
            Path::new(args[3]).canonicalize().unwrap(),
            compiler.canonicalize().unwrap()
        );
        assert_eq!(
            args[4..],
            [
                "setup-ide",
                ".",
                "--server=false",
                "--coursier-validate-checksums",
                "--offline"
            ]
        );
    }
}
