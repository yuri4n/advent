use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
use tempfile::TempDir;

fn fixture() -> TempDir {
    let temp = tempfile::tempdir().unwrap();
    fs::write(
        temp.path().join("aoc.toml"),
        r#"version = 1
[[languages]]
name = "fixture"
extension = "txt"
adapter = "adapter.sh"
"#,
    )
    .unwrap();
    fs::write(
        temp.path().join("adapter.sh"),
        r#"#!/bin/sh
set -eu
case "$1" in
build) echo build >> "$AOC_ROOT/calls" ;;
run) echo run >> "$AOC_ROOT/calls"; cat; printf 'part=%s\n' "$4" ;;
test) test "$4" = 2; printf 'one selected sample passed\n' ;;
new) mkdir -p "$AOC_ROOT/new" ;;
esac
"#,
    )
    .unwrap();
    fs::create_dir_all(temp.path().join("data/y22/d01")).unwrap();
    fs::write(temp.path().join("data/y22/d01/input.txt"), "fixture\n").unwrap();
    temp
}
fn aoc(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aoc"))
        .arg("--root")
        .arg(root)
        .args(args)
        .env_remove("AOC_SESSION")
        .output()
        .unwrap()
}
fn assert_ok(out: &Output) {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
#[test]
fn runs_both_parts_with_central_input_and_one_build() {
    let dir = fixture();
    let out = aoc(dir.path(), &["run", "22", "1", "--lang", "fixture"]);
    assert_ok(&out);
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "Part 1: fixture\npart=1\nPart 2: fixture\npart=2\n"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("calls")).unwrap(),
        "build\nrun\nrun\n"
    );
}
#[test]
fn selected_samples_preserve_failure_status() {
    let dir = fixture();
    assert_ok(&aoc(
        dir.path(),
        &["test", "2022", "1", "--lang", "fixture", "--part", "2"],
    ));
    let out = aoc(
        dir.path(),
        &["test", "2022", "1", "--lang", "fixture", "--part", "1"],
    );
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("samples failed"));
}
#[test]
fn offline_cache_hit_and_miss_require_no_network_or_cookie() {
    let dir = fixture();
    assert_ok(&aoc(dir.path(), &["fetch", "22", "1", "--offline"]));
    assert!(!aoc(dir.path(), &["fetch", "22", "2", "--offline"])
        .status
        .success());
    fs::write(
        dir.path().join("data/y22/d01/puzzle.html"),
        "<article class='day-desc'><h2>Example</h2><p>Read this.</p></article>",
    )
    .unwrap();
    let out = aoc(dir.path(), &["puzzle", "22", "1", "--offline"]);
    assert_ok(&out);
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "Example\n\nRead this.\n"
    );
    assert!(!aoc(dir.path(), &["puzzle", "22", "2", "--offline"])
        .status
        .success());
}
#[test]
fn benchmark_builds_once_and_runs_exact_count() {
    let dir = fixture();
    let out = aoc(
        dir.path(),
        &[
            "bench",
            "22",
            "1",
            "--lang",
            "fixture",
            "--part",
            "2",
            "--warmup",
            "1",
            "--iterations",
            "3",
        ],
    );
    assert_ok(&out);
    assert_eq!(
        fs::read_to_string(dir.path().join("calls")).unwrap(),
        "build\nrun\nrun\nrun\nrun\n"
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("3 runs, 1 warmups"));
    assert!(!aoc(
        dir.path(),
        &["bench", "22", "1", "--lang", "fixture", "--iterations", "0"]
    )
    .status
    .success());
}
#[test]
fn custom_language_scaffolds_without_overwrite_and_routes_commands() {
    let dir = fixture();
    let out = aoc(
        dir.path(),
        &[
            "add-language",
            "plain",
            "--extension",
            "txt",
            "--run",
            "cat {file}",
            "--test",
            "test -f {file}",
            "--stub",
            "UNIMPLEMENTED\n",
        ],
    );
    assert_ok(&out);
    assert_ok(&aoc(dir.path(), &["new", "22", "1", "--lang", "plain"]));
    let source = dir.path().join("solutions/plain/y22/d01.txt");
    assert_eq!(fs::read_to_string(&source).unwrap(), "UNIMPLEMENTED\n");
    fs::write(&source, "user work\n").unwrap();
    assert!(!aoc(dir.path(), &["new", "22", "1", "--lang", "plain"])
        .status
        .success());
    assert_eq!(fs::read_to_string(&source).unwrap(), "user work\n");
    assert_ok(&aoc(dir.path(), &["test", "22", "1", "--lang", "plain"]));
    let out = aoc(
        dir.path(),
        &["run", "22", "1", "--lang", "plain", "--part", "1"],
    );
    assert_ok(&out);
    assert_eq!(String::from_utf8(out.stdout).unwrap(), "user work\n");
    assert!(!aoc(
        dir.path(),
        &[
            "add-language",
            "../escape",
            "--extension",
            "sh",
            "--run",
            "true",
            "--test",
            "true"
        ]
    )
    .status
    .success());
}
#[test]
fn failed_runner_does_not_produce_successful_timing_or_answer() {
    let dir = fixture();
    fs::write(
        dir.path().join("adapter.sh"),
        "#!/bin/sh\n[ \"$1\" = build ] && exit 0\necho 'unfinished' >&2\nexit 7\n",
    )
    .unwrap();
    let out = aoc(
        dir.path(),
        &[
            "run", "22", "1", "--lang", "fixture", "--part", "2", "--time",
        ],
    );
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("unfinished"));
}

#[test]
fn runner_may_successfully_use_only_part_of_large_input() {
    let dir = fixture();
    fs::write(
        dir.path().join("adapter.sh"),
        "#!/bin/sh\n[ \"$1\" = build ] && exit 0\nhead -n 1\n",
    )
    .unwrap();
    let mut input = String::from("first line\n");
    input.push_str(&"ignored\n".repeat(200000));
    fs::write(dir.path().join("data/y22/d01/input.txt"), input).unwrap();
    let out = aoc(
        dir.path(),
        &["run", "22", "1", "--lang", "fixture", "--part", "1"],
    );
    assert_ok(&out);
    assert_eq!(out.stdout, b"first line\n");
}

#[test]
fn custom_tests_run_the_declared_build_first() {
    let dir = fixture();
    assert_ok(&aoc(
        dir.path(),
        &[
            "add-language",
            "compiled",
            "--extension",
            "txt",
            "--run",
            "cat",
            "--test",
            "test -f built",
            "--build",
            "touch built",
        ],
    ));
    assert_ok(&aoc(dir.path(), &["test", "22", "1", "--lang", "compiled"]));
}

#[test]
fn watch_observes_external_input_and_does_not_loop_on_build_outputs() {
    use std::{
        io::Write,
        process::Stdio,
        thread,
        time::{Duration, Instant},
    };
    let dir = fixture();
    fs::write(dir.path().join("adapter.sh"), "#!/bin/sh\nset -eu\ncase \"$1\" in\nbuild) touch \"$AOC_ROOT/built\";;\nrun) cat; echo run >> \"$AOC_ROOT/.aoc/runs\";;\nesac\n").unwrap();
    fs::create_dir(dir.path().join(".aoc")).unwrap();
    let mut external = tempfile::NamedTempFile::new().unwrap();
    writeln!(external, "before").unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_aoc"))
        .arg("--root")
        .arg(dir.path())
        .args([
            "watch", "22", "1", "--lang", "fixture", "--part", "1", "--input",
        ])
        .arg(external.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let count = || {
        fs::read_to_string(dir.path().join(".aoc/runs"))
            .unwrap_or_default()
            .lines()
            .count()
    };
    let wait_for = |n| {
        let start = Instant::now();
        while count() < n && start.elapsed() < Duration::from_secs(6) {
            thread::sleep(Duration::from_millis(20));
        }
        count() >= n
    };
    let initial = wait_for(1);
    thread::sleep(Duration::from_millis(200));
    fs::write(external.path(), "changed external input\n").unwrap();
    let rerun = wait_for(2);
    thread::sleep(Duration::from_millis(1000));
    let total = count();
    let _ = child.kill();
    let _ = child.wait();
    assert!(
        initial && rerun,
        "watch failed to observe initial or external input"
    );
    assert_eq!(total, 2, "build outputs caused a watch feedback loop");
}

#[cfg(unix)]
#[test]
fn downloads_keep_secrets_off_argv_and_preserve_good_cache_on_failure() {
    use std::os::unix::fs::PermissionsExt;
    let dir = fixture();
    let bin = dir.path().join("bin");
    fs::create_dir(&bin).unwrap();
    let curl = bin.join("curl");
    fs::write(&curl, "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$MOCK_DIR/argv\"\ncat > \"$MOCK_DIR/config\"\nprintf '%s' \"$MOCK_BODY\"\nexit \"$MOCK_EXIT\"\n").unwrap();
    fs::set_permissions(&curl, fs::Permissions::from_mode(0o755)).unwrap();
    let execute = |args: &[&str], body: &str, code: &str, session: bool| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_aoc"));
        command
            .arg("--root")
            .arg(dir.path())
            .args(args)
            .env(
                "PATH",
                format!("{}:{}", bin.display(), std::env::var("PATH").unwrap()),
            )
            .env("MOCK_DIR", dir.path())
            .env("MOCK_BODY", body)
            .env("MOCK_EXIT", code)
            .env_remove("AOC_SESSION");
        if session {
            command.env("AOC_SESSION", "abc123");
        }
        command.output().unwrap()
    };
    let args = ["fetch", "22", "1", "--force"];
    let out = execute(&args, "fresh input\n", "0", true);
    assert_ok(&out);
    assert_eq!(
        fs::read_to_string(dir.path().join("data/y22/d01/input.txt")).unwrap(),
        "fresh input\n"
    );
    assert!(!fs::read_to_string(dir.path().join("argv"))
        .unwrap()
        .contains("abc123"));
    assert!(fs::read_to_string(dir.path().join("config"))
        .unwrap()
        .contains("Cookie: session=abc123"));
    assert!(!String::from_utf8_lossy(&out.stdout).contains("abc123"));
    assert!(!String::from_utf8_lossy(&out.stderr).contains("abc123"));
    for (body, code, session) in [
        ("bad server", "22", true),
        ("<html>login</html>", "0", true),
        ("unused", "0", false),
    ] {
        assert!(!execute(&args, body, code, session).status.success());
        assert_eq!(
            fs::read_to_string(dir.path().join("data/y22/d01/input.txt")).unwrap(),
            "fresh input\n"
        );
    }
    let out = execute(
        &["puzzle", "22", "2"],
        "<article class='day-desc'><h2>Cached puzzle</h2><p>A &amp; B.</p></article>",
        "0",
        false,
    );
    assert_ok(&out);
    assert!(String::from_utf8_lossy(&out.stdout).contains("A & B."));
    assert_ok(&aoc(dir.path(), &["puzzle", "22", "2", "--offline"]));
}
