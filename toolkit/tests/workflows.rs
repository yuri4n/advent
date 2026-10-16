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
fn run_time_explains_that_processing_and_startup_are_included() {
    let dir = fixture();
    let out = aoc(
        dir.path(),
        &[
            "run", "22", "1", "--lang", "fixture", "--part", "1", "--time",
        ],
    );
    assert_ok(&out);
    let timing = String::from_utf8(out.stderr).unwrap();
    assert!(
        timing.contains("runtime startup + processing + solving"),
        "{timing}"
    );
    assert!(timing.contains("build excluded"));
    assert_eq!(out.stdout, b"fixture\npart=1\n");
}

#[test]
fn stdin_replays_for_both_parts_without_a_central_input() {
    use std::{io::Write, process::Stdio};
    let dir = fixture();
    fs::remove_file(dir.path().join("data/y22/d01/input.txt")).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_aoc"))
        .arg("--root")
        .arg(dir.path())
        .args(["run", "22", "1", "--lang", "fixture", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"  piped\r\n\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert_ok(&out);
    assert_eq!(
        out.stdout,
        b"Part 1:   piped\r\n\npart=1\nPart 2:   piped\r\n\npart=2\n"
    );
}

#[cfg(unix)]
#[test]
fn clipboard_is_read_once_replayed_and_never_written() {
    let dir = fixture();
    let helper = dir.path().join("clipboard reader");
    executable(&helper, "#!/bin/sh\nprintf 'read\\n' >> \"$AOC_ROOT/clipboard-calls\"\nprintf '  clipboard\\r\\n\\n'\nexit \"${CLIPBOARD_EXIT:-0}\"\n");
    let run = |extra: &[&str], code: &str| {
        Command::new(env!("CARGO_BIN_EXE_aoc"))
            .arg("--root")
            .arg(dir.path())
            .args(["run", "22", "1", "--lang", "fixture", "--clipboard"])
            .args(extra)
            .env("AOC_ROOT", dir.path())
            .env("AOC_CLIPBOARD_COMMAND", format!("'{}'", helper.display()))
            .env("CLIPBOARD_EXIT", code)
            .output()
            .unwrap()
    };
    let out = run(&[], "0");
    assert_ok(&out);
    assert_eq!(
        out.stdout,
        b"Part 1:   clipboard\r\n\npart=1\nPart 2:   clipboard\r\n\npart=2\n"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("clipboard-calls")).unwrap(),
        "read\n"
    );
    assert!(!run(&["--stdin"], "0").status.success());
    let failed = run(&[], "7");
    assert!(!failed.status.success());
    assert!(failed.stdout.is_empty());
}

#[test]
fn selected_example_reports_pass_fail_and_unchecked_without_changing_answer_stdout() {
    let dir = fixture();
    let config = dir.path().join("aoc.toml");
    fs::write(
        &config,
        fs::read_to_string(&config).unwrap() + "example_expectations = true\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("adapter.sh"),
        r#"#!/bin/sh
set -eu
case "$1" in
build) ;;
example) test "$4" = 2; printf '7\n' ;;
run) cat ;;
expect) test "$4" = 1; test "$5" = 2; cat "$AOC_ROOT/expectation" ;;
esac
"#,
    )
    .unwrap();
    let args = [
        "run",
        "22",
        "1",
        "--lang",
        "fixture",
        "--part",
        "1",
        "--example=2",
    ];
    fs::write(dir.path().join("expectation"), "AOC_EXPECT_V1\n7\n").unwrap();
    let pass = aoc(dir.path(), &args);
    assert_ok(&pass);
    assert_eq!(pass.stdout, b"7\n");
    assert!(String::from_utf8_lossy(&pass.stderr).contains("Example 2 part 1: PASS"));
    fs::write(dir.path().join("expectation"), "AOC_EXPECT_V1\n8\n").unwrap();
    let fail = aoc(dir.path(), &args);
    assert!(!fail.status.success());
    let error = String::from_utf8_lossy(&fail.stderr);
    assert!(
        error.contains("FAIL")
            && error.contains("expected")
            && error.contains('8')
            && error.contains('7'),
        "{error}"
    );
    fs::write(dir.path().join("expectation"), "AOC_NO_EXPECT_V1\n").unwrap();
    let unchecked = aoc(dir.path(), &args);
    assert_ok(&unchecked);
    assert!(String::from_utf8_lossy(&unchecked.stderr).contains("UNCHECKED"));
    fs::write(dir.path().join("expectation"), "broken protocol").unwrap();
    assert!(!aoc(dir.path(), &args).status.success());
}

#[test]
fn inline_example_uses_the_normal_runner_without_real_input_or_assertions() {
    let dir = fixture();
    fs::remove_file(dir.path().join("data/y22/d01/input.txt")).unwrap();
    fs::write(
        dir.path().join("adapter.sh"),
        r#"#!/bin/sh
set -eu
case "$1" in
build) echo build >> "$AOC_ROOT/calls" ;;
example) echo example >> "$AOC_ROOT/calls"; test "$4" = 2; printf '  example\n\n' ;;
run) echo run >> "$AOC_ROOT/calls"; cat; printf 'part=%s\n' "$4" ;;
esac
"#,
    )
    .unwrap();
    let out = aoc(
        dir.path(),
        &["run", "22", "1", "--lang", "fixture", "--example=2"],
    );
    assert_ok(&out);
    assert_eq!(
        out.stdout,
        b"Part 1:   example\n\npart=1\nPart 2:   example\n\npart=2\n"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("calls")).unwrap(),
        "build\nexample\nrun\nrun\n"
    );
    for extra in ["--example", "--example=0"] {
        assert!(
            !aoc(dir.path(), &["run", "22", "1", "--lang", "fixture", extra])
                .status
                .success()
        );
    }
    assert!(!aoc(
        dir.path(),
        &[
            "run",
            "22",
            "1",
            "--lang",
            "fixture",
            "--example=2",
            "--stdin"
        ]
    )
    .status
    .success());
}

#[test]
fn single_part_stdin_can_finish_before_the_producer_sends_eof() {
    use std::{
        io::Write,
        process::Stdio,
        thread,
        time::{Duration, Instant},
    };
    let dir = fixture();
    fs::write(
        dir.path().join("adapter.sh"),
        "#!/bin/sh\n[ \"$1\" = build ] && exit 0\nhead -n 1\n",
    )
    .unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_aoc"))
        .arg("--root")
        .arg(dir.path())
        .args([
            "run", "22", "1", "--lang", "fixture", "--part", "1", "--stdin",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut producer = child.stdin.take().unwrap();
    producer.write_all(b"first line\n").unwrap();
    let start = Instant::now();
    let completed = loop {
        if child.try_wait().unwrap().is_some() {
            break true;
        }
        if start.elapsed() > Duration::from_secs(5) {
            let _ = child.kill();
            break false;
        }
        thread::sleep(Duration::from_millis(20));
    };
    drop(producer);
    let out = child.wait_with_output().unwrap();
    assert!(
        completed,
        "CLI buffered stdin until EOF instead of streaming it"
    );
    assert_ok(&out);
    assert_eq!(out.stdout, b"first line\n");
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
    assert!(String::from_utf8_lossy(&out.stderr).contains("examples failed"));
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
fn native_bench_reports_with_and_without_processing_separately() {
    let dir = fixture();
    let config = dir.path().join("aoc.toml");
    let mut cfg = fs::read_to_string(&config).unwrap();
    cfg.push_str("native_bench = true\n");
    fs::write(config, cfg).unwrap();
    fs::write(
        dir.path().join("adapter.sh"),
        r#"#!/bin/sh
set -eu
case "$1" in
build) echo build >> "$AOC_ROOT/calls" ;;
bench)
  test "$4" = 2 && test "$5" = 2 && test "$6" = 1
  test "$(cat)" = fixture
  printf 'AOC_BENCH_V1\n5000000\n3000000 1000000\n5000000 2000000\nanswer\nwith newline\n'
  ;;
esac
"#,
    )
    .unwrap();
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
            "--iterations",
            "2",
        ],
    );
    assert_ok(&out);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("With processing") && text.contains("Without processing"),
        "{text}"
    );
    assert!(
        text.contains("median=4.000") && text.contains("median=1.500"),
        "{text}"
    );
    assert!(text.contains("answer\nwith newline"));
    assert!(text.contains("5.000 ms") && text.contains("buffer"));
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

#[cfg(unix)]
fn executable(path: &Path, contents: &str) {
    use std::os::unix::fs::PermissionsExt;
    fs::write(path, contents).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(unix)]
fn editor_fixture(language: &str, extension: &str, source: &str) -> TempDir {
    // Spaces and shell metacharacters must remain ordinary path characters.
    let dir = tempfile::Builder::new()
        .prefix("aoc editor ' $(never-run) ")
        .tempdir()
        .unwrap();
    fs::write(
        dir.path().join("aoc.toml"),
        format!("version = 1\n[[languages]]\nname = \"{language}\"\nextension = \"{extension}\"\n"),
    )
    .unwrap();
    let source = dir.path().join(source);
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(source, "unfinished solution\n").unwrap();
    fs::create_dir_all(dir.path().join("editor/nvim")).unwrap();
    fs::write(dir.path().join("editor/nvim/init.lua"), "-- fixture\n").unwrap();
    fs::write(dir.path().join("editor/nvim/bootstrap.lua"), "-- fixture\n").unwrap();
    fs::create_dir_all(dir.path().join("bin")).unwrap();
    executable(
        &dir.path().join("bin/nvim"),
        r#"#!/bin/sh
printf '%s\n' "$@" > "$AOC_ROOT/editor-argv"
printf '%s\n' "$PWD" "$AOC_ROOT" "$AOC_PROJECT_ROOT" "$AOC_EDITOR_LANGUAGE" "${AOC_NIX_LANGUAGE-unset}" > "$AOC_ROOT/editor-env"
printf '%s\n' "${AOC_NVIM_BOOTSTRAP-unset}" > "$AOC_ROOT/editor-bootstrap"
exit "${EDITOR_EXIT:-0}"
"#,
    );
    dir
}

#[cfg(unix)]
fn editor_command(root: &Path, language: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aoc"));
    command
        .arg("--root")
        .arg(root)
        .args(["edit", "22", "1", "--lang", language])
        .env("PATH", root.join("bin"))
        .env_remove("AOC_NIX_LANGUAGE")
        .env_remove("VISUAL")
        .env_remove("EDITOR")
        .env_remove("EDITOR_EXIT");
    command
}

#[cfg(unix)]
fn add_nix_fixture(root: &Path, language: &str) {
    fs::write(root.join("flake.nix"), "{}\n").unwrap();
    fs::create_dir_all(root.join("nix/languages")).unwrap();
    fs::write(root.join(format!("nix/languages/{language}.nix")), "{}\n").unwrap();
    executable(
        &root.join("bin/nix"),
        r#"#!/bin/sh
printf '%s\n' "$@" >> "$AOC_ROOT/nix-argv"
printf '%s\n' "$PWD" >> "$AOC_ROOT/nix-cwd"
[ "$1" = --extra-experimental-features ] && [ "$2" = 'nix-command flakes' ] || exit 90
shift 2
[ "$1" = develop ] && [ "$3" = --command ] || exit 91
export AOC_NIX_LANGUAGE="${2##*#}"
shift 3
exec "$@"
"#,
    );
}

#[cfg(unix)]
#[test]
fn edit_selects_native_projects_and_custom_files_without_a_shell() {
    for (language, extension, project, source) in [
        ("rust", "rs", "rust", "rust/years/y2022/src/d01.rs"),
        ("go", "go", "go", "go/y22/d01/solution.go"),
        ("elixir", "ex", "elixir", "elixir/lib/y22/d01.ex"),
        (
            "scala",
            "scala",
            "scala",
            "scala/src/main/scala/aoc/solutions/y2022/Day01.scala",
        ),
        ("haskell", "hs", "haskell", "haskell/src/Year2022/Day01.hs"),
        ("plain", "txt", "", "solutions/plain/y22/d01.txt"),
    ] {
        let dir = editor_fixture(language, extension, source);
        let root = dir.path().canonicalize().unwrap();
        let out = editor_command(&root, language).output().unwrap();
        assert_ok(&out);
        assert!(String::from_utf8_lossy(&out.stderr).contains(&format!(
            "Opening {} in nvim (personal + project, auto)",
            root.join(source).display()
        )));
        assert_eq!(
            fs::read_to_string(root.join("editor-argv")).unwrap(),
            format!(
                "-c\nlua dofile(vim.env.AOC_NVIM_BOOTSTRAP)\n--\n{}\n",
                root.join(source).display()
            )
        );
        assert_eq!(
            fs::read_to_string(root.join("editor-bootstrap")).unwrap(),
            format!("{}\n", root.join("editor/nvim/bootstrap.lua").display())
        );
        let project = root.join(project).canonicalize().unwrap();
        assert_eq!(
            fs::read_to_string(root.join("editor-env")).unwrap(),
            format!(
                "{}\n{}\n{}\n{language}\nunset\n",
                project.display(),
                root.display(),
                project.display()
            )
        );
    }
}

#[cfg(unix)]
#[test]
fn edit_enters_selected_nix_shell_once_preserving_relative_root_and_cwd() {
    let dir = editor_fixture("rust", "rs", "rust/years/y2022/src/d01.rs");
    let root = dir.path().canonicalize().unwrap();
    add_nix_fixture(&root, "rust");
    // A relative --root keeps its meaning when the CLI re-enters through Nix.
    let out = Command::new(env!("CARGO_BIN_EXE_aoc"))
        .args(["--root", ".", "edit", "22", "1", "--lang", "rust"])
        .current_dir(&root)
        .env("PATH", root.join("bin"))
        .env_remove("AOC_NIX_LANGUAGE")
        .env_remove("VISUAL")
        .env_remove("EDITOR")
        .output()
        .unwrap();
    assert_ok(&out);
    let progress = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        progress
            .matches("Preparing rust environment for nvim")
            .count(),
        1
    );
    assert_eq!(progress.matches("Opening ").count(), 1);
    let nix = fs::read_to_string(root.join("nix-argv")).unwrap();
    assert_eq!(
        nix,
        format!(
            "--extra-experimental-features\nnix-command flakes\ndevelop\n{}#rust\n--command\n{}\n--root\n.\nedit\n22\n1\n--lang\nrust\n",
            root.display(),
            env!("CARGO_BIN_EXE_aoc")
        )
    );
    assert_eq!(
        fs::read_to_string(root.join("nix-cwd")).unwrap(),
        format!("{}\n", root.display())
    );
    assert!(fs::read_to_string(root.join("editor-env"))
        .unwrap()
        .ends_with("\nrust\nrust\n"));
}

#[cfg(unix)]
#[test]
fn edit_respects_user_config_current_shell_and_explicit_no_nix() {
    for (marker, no_nix) in [("rust", false), ("all", false), ("base", true)] {
        let dir = editor_fixture("rust", "rs", "rust/years/y2022/src/d01.rs");
        let root = dir.path().canonicalize().unwrap();
        add_nix_fixture(&root, "rust");
        fs::remove_file(root.join("editor/nvim/init.lua")).unwrap();
        fs::remove_file(root.join("editor/nvim/bootstrap.lua")).unwrap();
        let mut command = editor_command(&root, "rust");
        command
            .arg("--user-config")
            .env("AOC_NIX_LANGUAGE", marker)
            .env("AOC_NVIM_BOOTSTRAP", "inherited bootstrap must not be used");
        if no_nix {
            command.arg("--no-nix");
        }
        let out = command.output().unwrap();
        assert_ok(&out);
        assert!(String::from_utf8_lossy(&out.stderr).contains("in nvim (personal only)"));
        assert!(!root.join("nix-argv").exists());
        assert_eq!(
            fs::read_to_string(root.join("editor-argv")).unwrap(),
            format!(
                "--\n{}\n",
                root.join("rust/years/y2022/src/d01.rs").display()
            )
        );
        assert_eq!(
            fs::read_to_string(root.join("editor-bootstrap")).unwrap(),
            "unset\n"
        );
    }
}

#[cfg(unix)]
#[test]
fn edit_standalone_uses_complete_repository_config_and_conflicts_with_user_only() {
    let dir = editor_fixture("rust", "rs", "rust/years/y2022/src/d01.rs");
    let root = dir.path().canonicalize().unwrap();
    fs::remove_file(root.join("editor/nvim/bootstrap.lua")).unwrap();
    let out = editor_command(&root, "rust")
        .arg("--standalone")
        .env("AOC_NVIM_BOOTSTRAP", "inherited bootstrap must not be used")
        .output()
        .unwrap();
    assert_ok(&out);
    assert!(String::from_utf8_lossy(&out.stderr).contains("in nvim (standalone)"));
    assert_eq!(
        fs::read_to_string(root.join("editor-argv")).unwrap(),
        format!(
            "-u\n{}\n--\n{}\n",
            root.join("editor/nvim/init.lua").display(),
            root.join("rust/years/y2022/src/d01.rs").display()
        )
    );
    assert_eq!(
        fs::read_to_string(root.join("editor-bootstrap")).unwrap(),
        "unset\n"
    );
    let out = editor_command(&root, "rust")
        .args(["--standalone", "--user-config"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("cannot be used with"));
}

#[cfg(unix)]
#[test]
fn edit_rejects_missing_startup_files_before_entering_nix() {
    for standalone in [false, true] {
        let dir = editor_fixture("rust", "rs", "rust/years/y2022/src/d01.rs");
        add_nix_fixture(dir.path(), "rust");
        let file = if standalone {
            "init.lua"
        } else {
            "bootstrap.lua"
        };
        fs::remove_file(dir.path().join("editor/nvim").join(file)).unwrap();
        let mut command = editor_command(dir.path(), "rust");
        if standalone {
            command.arg("--standalone");
        }
        let out = command.output().unwrap();
        assert!(!out.status.success());
        assert!(String::from_utf8_lossy(&out.stderr).contains(file));
        assert!(!dir.path().join("nix-argv").exists());
        assert!(!dir.path().join("editor-argv").exists());
    }
}

#[cfg(unix)]
#[test]
fn edit_resolves_relative_path_entries_before_changing_project_directory() {
    let dir = editor_fixture("rust", "rs", "rust/years/y2022/src/d01.rs");
    assert_ok(
        &editor_command(dir.path(), "rust")
            .current_dir(dir.path())
            .env("PATH", "bin")
            .output()
            .unwrap(),
    );
    assert!(dir.path().join("editor-argv").is_file());
}

#[cfg(unix)]
#[test]
fn custom_language_without_nix_module_uses_current_tools() {
    let dir = editor_fixture("plain", "txt", "solutions/plain/y22/d01.txt");
    add_nix_fixture(dir.path(), "rust");
    assert_ok(
        &editor_command(dir.path(), "plain")
            .env("AOC_NIX_LANGUAGE", "base")
            .output()
            .unwrap(),
    );
    assert!(!dir.path().join("nix-argv").exists());
}

#[cfg(unix)]
#[test]
fn edit_rejects_missing_and_escaping_sources_before_starting_tools() {
    use std::os::unix::fs::symlink;
    let dir = editor_fixture("rust", "rs", "rust/years/y2022/src/d01.rs");
    add_nix_fixture(dir.path(), "rust");
    let source = dir.path().join("rust/years/y2022/src/d01.rs");
    fs::remove_file(&source).unwrap();
    let out = editor_command(dir.path(), "rust").output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("aoc new 2022 1 --lang rust"));
    assert!(!source.exists());
    let external = tempfile::NamedTempFile::new().unwrap();
    symlink(external.path(), &source).unwrap();
    let out = editor_command(dir.path(), "rust").output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("inside the repository"));
    assert!(!dir.path().join("editor-argv").exists());
    assert!(!dir.path().join("nix-argv").exists());
}

#[cfg(unix)]
#[test]
fn edit_reports_unavailable_neovim_and_editor_failures() {
    let dir = editor_fixture("plain", "txt", "solutions/plain/y22/d01.txt");
    let out = editor_command(dir.path(), "plain")
        .env("EDITOR_EXIT", "7")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("editor failed"));
    fs::remove_file(dir.path().join("bin/nvim")).unwrap();
    let out = editor_command(dir.path(), "plain").output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("editor \"nvim\" is unavailable"));
}

#[cfg(unix)]
#[test]
fn doctor_does_not_require_rust_for_another_language() {
    let dir = editor_fixture("plain", "txt", "solutions/plain/y22/d01.txt");
    executable(
        &dir.path().join("bin/curl"),
        "#!/bin/sh\nprintf 'curl fixture\\n'\n",
    );
    let out = Command::new(env!("CARGO_BIN_EXE_aoc"))
        .arg("--root")
        .arg(dir.path())
        .args(["doctor", "--lang", "plain"])
        .env("PATH", dir.path().join("bin"))
        .env_remove("AOC_NIX_LANGUAGE")
        .env_remove("AOC_SESSION")
        .output()
        .unwrap();
    assert_ok(&out);
    assert!(String::from_utf8_lossy(&out.stdout).contains("OK curl"));
    assert!(!String::from_utf8_lossy(&out.stderr).contains("cargo"));
}

#[cfg(unix)]
#[test]
fn base_nix_shell_routes_native_actions_once_but_leaves_list_alone() {
    let dir = fixture();
    fs::create_dir(dir.path().join("bin")).unwrap();
    add_nix_fixture(dir.path(), "fixture");
    let command = |args: &[&str], marker: &str| {
        Command::new(env!("CARGO_BIN_EXE_aoc"))
            .arg("--root")
            .arg(dir.path())
            .args(args)
            .env(
                "PATH",
                format!(
                    "{}:{}",
                    dir.path().join("bin").display(),
                    std::env::var("PATH").unwrap()
                ),
            )
            .env("AOC_NIX_LANGUAGE", marker)
            .current_dir(dir.path())
            .output()
            .unwrap()
    };
    assert_ok(&command(&["list"], "base"));
    assert!(!dir.path().join("nix-argv").exists());
    fs::write(dir.path().join("selected.txt"), "relative input\n").unwrap();
    let out = command(
        &[
            "run",
            "22",
            "1",
            "--lang",
            "fixture",
            "--input",
            "selected.txt",
        ],
        "base",
    );
    assert_ok(&out);
    assert!(String::from_utf8_lossy(&out.stdout).contains("Part 1: relative input\n"));
    assert_eq!(
        fs::read_to_string(dir.path().join("calls")).unwrap(),
        "build\nrun\nrun\n"
    );
    let nix = fs::read_to_string(dir.path().join("nix-argv")).unwrap();
    assert_eq!(nix.matches("\ndevelop\n").count(), 1);
    for marker in ["fixture", "all"] {
        assert_ok(&command(&["run", "22", "1", "--lang", "fixture"], marker));
        assert_eq!(
            fs::read_to_string(dir.path().join("nix-argv")).unwrap(),
            nix
        );
    }
}

#[cfg(unix)]
fn fixture_editor(root: &Path, name: &str) {
    fs::copy(root.join("bin/nvim"), root.join("bin").join(name)).unwrap();
}

#[cfg(unix)]
#[test]
fn editor_selection_honors_override_visual_editor_and_blank_fallback() {
    for (explicit, visual, editor, selected, flag) in [
        (
            Some("cursor --explicit"),
            "zed --visual",
            "code --editor",
            "cursor",
            "--explicit",
        ),
        (None, "zed --visual", "code --editor", "zed", "--visual"),
        (
            None,
            "zeditor --visual",
            "code --editor",
            "zeditor",
            "--visual",
        ),
        (None, "  \t", "code --editor", "code", "--editor"),
        (None, "", " \t ", "nvim", "-c"),
    ] {
        let dir = editor_fixture("rust", "rs", "rust/years/y2022/src/d01.rs");
        for name in ["zed", "zeditor", "code", "cursor"] {
            fixture_editor(dir.path(), name);
        }
        let mut command = editor_command(dir.path(), "rust");
        command.env("VISUAL", visual).env("EDITOR", editor);
        if let Some(explicit) = explicit {
            command.args(["--editor", explicit]);
        }
        let out = command.output().unwrap();
        assert_ok(&out);
        assert!(String::from_utf8_lossy(&out.stderr).contains(&format!("in {selected} (")));
        let args = fs::read_to_string(dir.path().join("editor-argv")).unwrap();
        assert!(args.starts_with(&format!("{flag}\n")));
        if selected != "nvim" {
            assert!(args.ends_with(&format!(
                "--\n{}\n{}\n",
                dir.path().join("rust").display(),
                dir.path().join("rust/years/y2022/src/d01.rs").display()
            )));
            assert_eq!(
                fs::read_to_string(dir.path().join("editor-bootstrap")).unwrap(),
                "unset\n"
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn generic_editor_command_preserves_quoted_argv_without_shell_evaluation() {
    let dir = editor_fixture("rust", "rs", "rust/years/y2022/src/d01.rs");
    fixture_editor(dir.path(), "custom editor");
    let program = dir.path().join("bin/custom editor");
    let marker = dir.path().join("shell-evaluated");
    let substitution = format!("$(touch {})", marker.display());
    let configured = format!(
        "{} --wait 'two words' {} ';' '$HOME'",
        shlex::try_quote(program.to_str().unwrap()).unwrap(),
        shlex::try_quote(&substitution).unwrap()
    );
    assert_ok(
        &editor_command(dir.path(), "rust")
            .args(["--editor", &configured])
            .output()
            .unwrap(),
    );
    assert!(!marker.exists());
    assert_eq!(
        fs::read_to_string(dir.path().join("editor-argv")).unwrap(),
        format!(
            "--wait\ntwo words\n{substitution}\n;\n$HOME\n--\n{}\n",
            dir.path().join("rust/years/y2022/src/d01.rs").display()
        )
    );
    assert!(fs::read_to_string(dir.path().join("editor-env"))
        .unwrap()
        .starts_with(&format!("{}\n", dir.path().join("rust").display())));
}

#[cfg(unix)]
#[test]
fn invalid_editor_commands_and_neovim_only_flags_fail_before_nix() {
    for arguments in [
        vec!["--editor", ""],
        vec!["--editor", "zed 'unclosed"],
        vec!["--editor", "zed", "--standalone"],
        vec!["--editor", "zed", "--user-config"],
    ] {
        let dir = editor_fixture("rust", "rs", "rust/years/y2022/src/d01.rs");
        add_nix_fixture(dir.path(), "rust");
        let out = editor_command(dir.path(), "rust")
            .args(arguments)
            .output()
            .unwrap();
        assert!(!out.status.success());
        let message = String::from_utf8_lossy(&out.stderr);
        assert!(message.contains("editor command") || message.contains("apply only to Neovim"));
        assert!(!dir.path().join("nix-argv").exists());
        assert!(!dir.path().join("editor-argv").exists());
    }
}

#[cfg(unix)]
#[test]
fn zed_does_not_need_neovim_files_and_propagates_launch_failure() {
    let dir = editor_fixture("rust", "rs", "rust/years/y2022/src/d01.rs");
    fixture_editor(dir.path(), "zed");
    fs::remove_dir_all(dir.path().join("editor")).unwrap();
    let out = editor_command(dir.path(), "rust")
        .env("VISUAL", "zed --wait")
        .env("EDITOR_EXIT", "7")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("editor failed"));
    assert!(fs::read_to_string(dir.path().join("editor-argv"))
        .unwrap()
        .starts_with("--wait\n--\n"));
    assert_eq!(
        fs::read_to_string(dir.path().join("editor-bootstrap")).unwrap(),
        "unset\n"
    );
}

#[cfg(unix)]
fn fixture_editor_setup(root: &Path) {
    executable(
        &root.join("bin/prepare-project"),
        r#"#!/bin/sh
printf '%s\n' "$PWD" "$AOC_ROOT" "$AOC_PROJECT_ROOT" "$AOC_EDITOR_LANGUAGE" "$@" > "$AOC_ROOT/project-setup"
exit "${SETUP_EXIT:-0}"
"#,
    );
    let path = root.join("aoc.toml");
    let mut config = fs::read_to_string(&path).unwrap();
    config.push_str(&format!(
        "editor_setup = [\"{}\", \"argument with spaces\", \"literal;$HOME\"]\n",
        root.join("bin/prepare-project").display()
    ));
    fs::write(path, config).unwrap();
}

#[cfg(unix)]
#[test]
fn editor_project_setup_runs_for_generic_and_personal_only_modes() {
    for (editor, mode, expected) in [
        ("zed", None, true),
        ("nvim", Some("--user-config"), true),
        ("nvim", None, false),
        ("nvim", Some("--standalone"), false),
    ] {
        let dir = editor_fixture("rust", "rs", "rust/years/y2022/src/d01.rs");
        fixture_editor(dir.path(), "zed");
        fixture_editor_setup(dir.path());
        let mut command = editor_command(dir.path(), "rust");
        command.args(["--editor", editor]).env_remove("SETUP_EXIT");
        if let Some(mode) = mode {
            command.arg(mode);
        }
        assert_ok(&command.output().unwrap());
        assert_eq!(dir.path().join("project-setup").exists(), expected);
        if expected {
            assert_eq!(
                fs::read_to_string(dir.path().join("project-setup")).unwrap(),
                format!(
                    "{}\n{}\n{}\nrust\nargument with spaces\nliteral;$HOME\n",
                    dir.path().display(),
                    dir.path().display(),
                    dir.path().join("rust").display()
                )
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn editor_project_setup_errors_prevent_launching_editor() {
    let dir = editor_fixture("rust", "rs", "rust/years/y2022/src/d01.rs");
    fixture_editor(dir.path(), "zed");
    fixture_editor_setup(dir.path());
    let out = editor_command(dir.path(), "rust")
        .env("VISUAL", "zed")
        .env("SETUP_EXIT", "9")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("editor project setup failed"));
    assert!(!dir.path().join("editor-argv").exists());
    let config = dir.path().join("aoc.toml");
    let contents = fs::read_to_string(&config).unwrap();
    let prefix = contents.split("editor_setup =").next().unwrap();
    fs::write(config, format!("{prefix}editor_setup = []\n")).unwrap();
    let out = editor_command(dir.path(), "rust")
        .env("VISUAL", "zed")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("editor_setup must contain an executable")
    );
    assert!(!dir.path().join("editor-argv").exists());
}
