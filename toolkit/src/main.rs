use clap::{Args, Parser, Subcommand, ValueEnum};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
#[command(
    name = "aoc",
    version,
    about = "A small, editor-friendly Advent of Code toolkit"
)]
struct Cli {
    /// Repository root (normally discovered from your working directory).
    #[arg(long, global = true)]
    root: Option<PathBuf>,
    #[command(subcommand)]
    command: Action,
}
#[derive(Args, Clone)]
struct Day {
    /// Four-digit year, or its two-digit abbreviation.
    #[arg(value_parser = parse_year)]
    year: u16,
    #[arg(value_parser = clap::value_parser!(u8).range(1..=25))]
    day: u8,
}
#[derive(Args, Clone)]
struct Target {
    #[command(flatten)]
    day: Day,
    #[arg(long, short = 'l')]
    lang: String,
}
#[derive(Clone, Copy, ValueEnum)]
enum Part {
    #[value(name = "1")]
    One,
    #[value(name = "2")]
    Two,
    Both,
}
impl Part {
    fn label(self) -> &'static str {
        match self {
            Self::One => "1",
            Self::Two => "2",
            Self::Both => "both",
        }
    }
    fn numbers(self) -> &'static [u8] {
        match self {
            Self::One => &[1],
            Self::Two => &[2],
            Self::Both => &[1, 2],
        }
    }
}
#[derive(Args, Clone)]
struct Run {
    #[command(flatten)]
    target: Target,
    #[arg(long, value_enum, default_value = "both")]
    part: Part,
    /// Override the central input file; relative paths are relative to the shell.
    #[arg(long)]
    input: Option<PathBuf>,
    /// Print process wall time, excluding the initial build, to stderr.
    #[arg(long)]
    time: bool,
}
#[derive(Subcommand)]
enum Action {
    /// Create empty solution stubs and register them with the native build tool.
    New(Target),
    /// Open a solution in Neovim with its project's language server and toolchain.
    Edit {
        #[command(flatten)]
        target: Target,
        /// Use your own Neovim configuration instead of the repository's setup.
        #[arg(long)]
        user_config: bool,
        /// Use tools already on PATH instead of entering a Nix language shell.
        #[arg(long)]
        no_nix: bool,
    },
    /// Build and run selected puzzle parts against a local input file.
    Run(Run),
    /// Run inline sample assertions for a day.
    Test {
        #[command(flatten)]
        target: Target,
        #[arg(long, value_enum, default_value = "both")]
        part: Part,
    },
    /// Repeated process wall-time measurements after one build (not a microbenchmark).
    Bench {
        #[command(flatten)]
        run: Run,
        #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u32).range(1..=10000))]
        iterations: u32,
        #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u32).range(0..=1000))]
        warmup: u32,
    },
    /// Run again after source/input changes; stop with Ctrl-C.
    Watch(Run),
    /// Download personal input, or reuse its local cache.
    Fetch {
        #[command(flatten)]
        day: Day,
        #[arg(long, conflicts_with = "force")]
        offline: bool,
        #[arg(long)]
        force: bool,
    },
    /// Read puzzle text in the terminal, with an offline HTML cache.
    Puzzle {
        #[command(flatten)]
        day: Day,
        #[arg(long, conflicts_with = "refresh")]
        offline: bool,
        #[arg(long)]
        refresh: bool,
    },
    /// Check required tools and native language adapters.
    Doctor {
        #[arg(long)]
        lang: Option<String>,
    },
    /// Show the registered languages and their adapters.
    List,
    /// Register a language, its commands, and an empty solution template.
    AddLanguage {
        name: String,
        #[arg(long)]
        extension: String,
        /// Shell command with {file}, {year}, {day}, and {part} placeholders; input arrives on stdin.
        #[arg(long)]
        run: String,
        /// Shell command that runs this day's sample assertions.
        #[arg(long)]
        test: String,
        /// Optional compilation command, executed before running/benchmarking.
        #[arg(long)]
        build: Option<String>,
        /// Empty source template (for example `raise NotImplementedError`).
        #[arg(long, default_value = "TODO: implement part1 and part2\n")]
        stub: String,
    },
}
#[derive(Serialize, Deserialize)]
struct Config {
    version: u8,
    languages: Vec<Language>,
}
#[derive(Clone, Serialize, Deserialize)]
struct Language {
    name: String,
    extension: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    adapter: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    run: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    test: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    build: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    template: Option<String>,
}
fn parse_year(value: &str) -> std::result::Result<u16, String> {
    let mut year = value.parse::<u16>().map_err(|_| "year must be a number")?;
    if year < 100 {
        year += 2000;
    }
    if !(2015..=2099).contains(&year) {
        return Err("year must be 2015..2099 (or 15..99)".into());
    }
    Ok(year)
}
fn main() {
    if let Err(error) = execute(Cli::parse()) {
        eprintln!("aoc: {error}");
        std::process::exit(1);
    }
}
fn root_path(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = explicit.or_else(|| env::var_os("AOC_ROOT").map(PathBuf::from)) {
        let root = path.canonicalize()?;
        if !root.join("aoc.toml").is_file() {
            return Err("repository root needs aoc.toml".into());
        }
        return Ok(root);
    }
    for path in env::current_dir()?.ancestors() {
        if path.join("aoc.toml").is_file() {
            return Ok(path.to_path_buf());
        }
    }
    Err("cannot find aoc.toml; run inside the repository or use --root PATH".into())
}
fn config(root: &Path) -> Result<Config> {
    let cfg: Config = toml::from_str(&fs::read_to_string(root.join("aoc.toml"))?)?;
    if cfg.version != 1 {
        return Err("unsupported aoc.toml version".into());
    }
    Ok(cfg)
}
fn language<'a>(cfg: &'a Config, name: &str) -> Result<&'a Language> {
    cfg.languages
        .iter()
        .find(|l| l.name == name)
        .ok_or_else(|| format!("unknown language {name:?}; see aoc list").into())
}
fn data_dir(root: &Path, day: &Day) -> PathBuf {
    root.join(format!("data/y{:02}/d{:02}", day.year % 100, day.day))
}
fn source_path(root: &Path, lang: &Language, day: &Day) -> PathBuf {
    root.join(format!(
        "solutions/{}/y{:02}/d{:02}.{}",
        lang.name,
        day.year % 100,
        day.day,
        lang.extension
    ))
}
fn editor_paths(root: &Path, lang: &Language, day: &Day) -> Result<(PathBuf, PathBuf)> {
    if !valid_identifier(&lang.name) || !valid_identifier(&lang.extension) {
        return Err("language and extension must use lowercase letters, digits, '-' or '_'".into());
    }
    let (project, source) = match lang.name.as_str() {
        "rust" => (
            root.join("rust"),
            root.join(format!("rust/years/y{}/src/d{:02}.rs", day.year, day.day)),
        ),
        "go" => (
            root.join("go"),
            root.join(format!(
                "go/y{:02}/d{:02}/solution.go",
                day.year % 100,
                day.day
            )),
        ),
        "elixir" => (
            root.join("elixir"),
            root.join(format!(
                "elixir/lib/y{:02}/d{:02}.ex",
                day.year % 100,
                day.day
            )),
        ),
        "scala" => (
            root.join("scala"),
            root.join(format!(
                "scala/src/main/scala/aoc/solutions/y{}/Day{:02}.scala",
                day.year, day.day
            )),
        ),
        "haskell" => (
            root.join("haskell"),
            root.join(format!("haskell/src/Year{}/Day{:02}.hs", day.year, day.day)),
        ),
        _ => (root.to_path_buf(), source_path(root, lang, day)),
    };
    if !source.is_file() {
        return Err(format!(
            "solution is missing: {}; create it with aoc new {} {} --lang {}",
            source.display(),
            day.year,
            day.day,
            lang.name
        )
        .into());
    }
    let source = source.canonicalize()?;
    let project = project.canonicalize()?;
    if !source.starts_with(root) || !project.starts_with(root) {
        return Err("solution and project must stay inside the repository".into());
    }
    Ok((project, source))
}
fn executable_on_path(name: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|path| path.join(name))
            .find(|path| {
                let Ok(metadata) = fs::metadata(path) else {
                    return false;
                };
                if !metadata.is_file() {
                    return false;
                }
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    metadata.permissions().mode() & 0o111 != 0
                }
                #[cfg(not(unix))]
                true
            })
            .and_then(|path| path.canonicalize().ok())
    })
}
fn edit(root: &Path, lang: &Language, day: &Day, user_config: bool) -> Result<()> {
    let (project, source) = editor_paths(root, lang, day)?;
    let init = root.join("editor/nvim/init.lua");
    if !user_config && !init.is_file() {
        return Err(format!(
            "repository Neovim configuration is missing: {}; use --user-config for your own setup",
            init.display()
        )
        .into());
    }
    let nvim = executable_on_path("nvim").ok_or(
        "Neovim is unavailable; install it or enter the repository's Nix development shell",
    )?;
    let mut command = Command::new(nvim);
    if !user_config {
        command.arg("-u").arg(init);
    }
    command
        .arg("--")
        .arg(source)
        .current_dir(&project)
        .env("AOC_ROOT", root)
        .env("AOC_PROJECT_ROOT", &project)
        .env("AOC_EDITOR_LANGUAGE", &lang.name);
    let status = command.status()?;
    if !status.success() {
        return Err(format!("editor failed ({status})").into());
    }
    Ok(())
}
fn enter_language_shell(root: &Path, cfg: &Config, action: &Action) -> Result<bool> {
    let (name, editor) = match action {
        Action::New(target) | Action::Test { target, .. } => (target.lang.as_str(), false),
        Action::Run(run) | Action::Watch(run) | Action::Bench { run, .. } => {
            (run.target.lang.as_str(), false)
        }
        Action::Edit { no_nix: true, .. } => return Ok(false),
        Action::Edit {
            target,
            user_config,
            ..
        } => {
            // Reject a missing solution before asking Nix to download a toolchain.
            let lang = language(cfg, &target.lang)?;
            editor_paths(root, lang, &target.day)?;
            if !user_config && !root.join("editor/nvim/init.lua").is_file() {
                return Ok(false); // edit provides the configuration error.
            }
            (target.lang.as_str(), true)
        }
        Action::Doctor { lang } => (lang.as_deref().unwrap_or("all"), false),
        _ => return Ok(false),
    };
    let all_languages = matches!(action, Action::Doctor { lang: None });
    if !all_languages {
        language(cfg, name)?;
    }
    let current = env::var("AOC_NIX_LANGUAGE").ok();
    if (!editor && current.is_none())
        || matches!(current.as_deref(), Some("all"))
        || current.as_deref() == Some(name)
        || !root.join("flake.nix").is_file()
        || (!all_languages && !root.join(format!("nix/languages/{name}.nix")).is_file())
    {
        return Ok(false);
    }
    let Some(nix) = executable_on_path("nix") else {
        return Ok(false);
    };
    let mut flake = root.as_os_str().to_os_string();
    flake.push(format!("#{name}"));
    // Re-enter once around the whole action, keeping Nix startup out of benchmarks
    // and preserving relative --root and --input paths by retaining the caller's cwd.
    let status = Command::new(nix)
        .args(["--extra-experimental-features", "nix-command flakes"])
        .arg("develop")
        .arg(flake)
        .arg("--command")
        .arg(env::current_exe()?)
        .args(env::args_os().skip(1))
        .env("AOC_ROOT", root)
        .status()?;
    if !status.success() {
        return Err(format!("Nix language shell failed ({status})").into());
    }
    Ok(true)
}
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
fn expand(template: &str, root: &Path, lang: &Language, day: &Day, part: &str) -> String {
    template
        .replace(
            "{file}",
            &shell_quote(&source_path(root, lang, day).to_string_lossy()),
        )
        .replace("{year}", &day.year.to_string())
        .replace("{day}", &day.day.to_string())
        .replace("{part}", part)
}
fn adapter_command(
    root: &Path,
    lang: &Language,
    mode: &str,
    day: &Day,
    part: &str,
) -> Result<Command> {
    let mut command;
    if let Some(adapter) = &lang.adapter {
        if !root.join(adapter).is_file() {
            return Err(format!("adapter is missing: {adapter}").into());
        }
        command = Command::new("bash");
        command.arg(root.join(adapter)).arg(mode);
        if mode != "doctor" {
            command.arg(day.year.to_string());
        }
        if mode != "doctor" && mode != "build" {
            command.arg(day.day.to_string());
        }
        if mode == "run" || mode == "test" {
            command.arg(part);
        }
    } else {
        let template = match mode {
            "run" => lang.run.as_ref(),
            "test" => lang.test.as_ref(),
            "build" => lang.build.as_ref(),
            _ => None,
        };
        let template = template.ok_or_else(|| format!("{} has no {mode} command", lang.name))?;
        command = Command::new("sh");
        command
            .arg("-c")
            .arg(expand(template, root, lang, day, part));
    }
    command
        .current_dir(root)
        .env("AOC_ROOT", root)
        .stdin(Stdio::null());
    Ok(command)
}
fn status(mut command: Command, label: &str, test: bool) -> Result<()> {
    let output = command.output()?;
    if test {
        io::stdout().write_all(&output.stdout)?;
    } else {
        io::stderr().write_all(&output.stdout)?;
    }
    io::stderr().write_all(&output.stderr)?;
    if !output.status.success() {
        return Err(format!("{label} failed ({})", output.status).into());
    }
    Ok(())
}
fn build(root: &Path, lang: &Language, day: &Day) -> Result<()> {
    if lang.adapter.is_some() || lang.build.is_some() {
        status(
            adapter_command(root, lang, "build", day, "both")?,
            "build",
            false,
        )?;
    }
    Ok(())
}
fn invoke(
    root: &Path,
    lang: &Language,
    day: &Day,
    part: u8,
    input: &[u8],
) -> Result<(String, Duration)> {
    let mut command = adapter_command(root, lang, "run", day, &part.to_string())?;
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    let started = Instant::now();
    let mut child = command.spawn()?;
    let mut pipe = child.stdin.take().ok_or("child stdin missing")?;
    let bytes = input.to_vec();
    let writer = thread::spawn(move || pipe.write_all(&bytes));
    let output = child.wait_with_output()?;
    let elapsed = started.elapsed();
    let written = writer.join().map_err(|_| "input writer panicked")?;
    if !output.status.success() {
        return Err(format!(
            "{} {} day {} part {part} failed ({})",
            lang.name, day.year, day.day, output.status
        )
        .into());
    }
    if let Err(error) = written {
        if error.kind() != io::ErrorKind::BrokenPipe {
            return Err(error.into());
        }
    }
    let answer = String::from_utf8(output.stdout)?.trim_end().to_owned();
    if answer.is_empty() {
        return Err("runner returned an empty answer".into());
    }
    Ok((answer, elapsed))
}
fn input(root: &Path, run: &Run) -> Result<Vec<u8>> {
    let path = run
        .input
        .clone()
        .unwrap_or_else(|| data_dir(root, &run.target.day).join("input.txt"));
    fs::read(&path).map_err(|error| {
        format!(
            "cannot read {}: {error}; use aoc fetch or --input PATH",
            path.display()
        )
        .into()
    })
}
fn run_once(root: &Path, lang: &Language, run: &Run) -> Result<()> {
    let input = input(root, run)?;
    build(root, lang, &run.target.day)?;
    for &part in run.part.numbers() {
        let (answer, elapsed) = invoke(root, lang, &run.target.day, part, &input)?;
        if matches!(run.part, Part::Both) {
            println!("Part {part}: {answer}");
        } else {
            println!("{answer}");
        }
        if run.time {
            eprintln!(
                "part {part}: {:.3} ms (process wall time; build excluded)",
                elapsed.as_secs_f64() * 1000.0
            );
        }
    }
    Ok(())
}
fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-' || c == b'_')
}
fn write_new(path: &Path, contents: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(contents)?;
    Ok(())
}
fn atomic_write(path: &Path, contents: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or("missing parent directory")?;
    fs::create_dir_all(parent)?;
    let temp = parent.join(format!(
        ".aoc-{}-{}.tmp",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_nanos()
    ));
    let result = (|| {
        write_new(&temp, contents)?;
        fs::rename(&temp, path)?;
        Ok(())
    })();
    if temp.exists() {
        let _ = fs::remove_file(&temp);
    }
    result
}
fn execute(cli: Cli) -> Result<()> {
    let root = root_path(cli.root)?;
    let mut cfg = config(&root)?;
    if enter_language_shell(&root, &cfg, &cli.command)? {
        return Ok(());
    }
    match cli.command {
        Action::New(target) => {
            let lang = language(&cfg, &target.lang)?;
            if lang.adapter.is_some() {
                status(
                    adapter_command(&root, lang, "new", &target.day, "both")?,
                    "new day",
                    true,
                )?;
            } else {
                let template = lang.template.as_ref().ok_or("language has no template")?;
                let contents = fs::read_to_string(root.join(template))?
                    .replace("{year}", &target.day.year.to_string())
                    .replace("{day}", &target.day.day.to_string());
                let path = source_path(&root, lang, &target.day);
                write_new(&path, contents.as_bytes())?;
                println!("Created {}", path.strip_prefix(&root)?.display());
            }
            fs::create_dir_all(data_dir(&root, &target.day))?;
        }
        Action::Run(run) => run_once(&root, language(&cfg, &run.target.lang)?, &run)?,
        Action::Edit {
            target,
            user_config,
            ..
        } => edit(
            &root,
            language(&cfg, &target.lang)?,
            &target.day,
            user_config,
        )?,
        Action::Test { target, part } => {
            let lang = language(&cfg, &target.lang)?;
            if lang.adapter.is_none() {
                build(&root, lang, &target.day)?;
            }
            status(
                adapter_command(&root, lang, "test", &target.day, part.label())?,
                "samples",
                true,
            )?;
        }
        Action::Bench {
            run,
            iterations,
            warmup,
        } => {
            let lang = language(&cfg, &run.target.lang)?;
            let input = input(&root, &run)?;
            build(&root, lang, &run.target.day)?;
            for &part in run.part.numbers() {
                let mut expected = None;
                let mut times = Vec::new();
                for i in 0..(warmup + iterations) {
                    let (answer, elapsed) = invoke(&root, lang, &run.target.day, part, &input)?;
                    if let Some(previous) = &expected {
                        if previous != &answer {
                            return Err("answer changed between benchmark runs".into());
                        }
                    } else {
                        expected = Some(answer);
                    }
                    if i >= warmup {
                        times.push(elapsed.as_secs_f64() * 1000.0);
                    }
                }
                times.sort_by(f64::total_cmp);
                let mean = times.iter().sum::<f64>() / times.len() as f64;
                let mid = times.len() / 2;
                let median = if times.len() % 2 == 0 {
                    (times[mid - 1] + times[mid]) / 2.0
                } else {
                    times[mid]
                };
                println!("Part {part}: {}\n{iterations} runs, {warmup} warmups; ms min={:.3} median={median:.3} mean={mean:.3} max={:.3}\nProcess wall time includes runtime startup; build excluded.", expected.unwrap(), times[0], times[times.len()-1]);
            }
        }
        Action::Watch(run) => {
            let lang = language(&cfg, &run.target.lang)?;
            if let Err(error) = run_once(&root, lang, &run) {
                eprintln!("aoc watch: {error}");
            }
            // The initial build may create files; take its settled state as the baseline.
            let mut previous = watch_snapshot(&root, run.input.as_deref())?;
            eprintln!("Watching source and input files; Ctrl-C to stop.");
            loop {
                thread::sleep(Duration::from_millis(400));
                let now = watch_snapshot(&root, run.input.as_deref())?;
                if now != previous {
                    thread::sleep(Duration::from_millis(150));
                    if let Err(error) = run_once(&root, lang, &run) {
                        eprintln!("aoc watch: {error}");
                    }
                    previous = watch_snapshot(&root, run.input.as_deref())?;
                }
            }
        }
        Action::Fetch {
            day,
            offline,
            force,
        } => {
            let path = data_dir(&root, &day).join("input.txt");
            if path.is_file() && !force {
                println!("{} (cached)", path.display());
            } else if offline {
                return Err("input is not cached; offline mode makes no network requests".into());
            } else {
                let body = download(
                    &format!(
                        "https://adventofcode.com/{}/day/{}/input",
                        day.year, day.day
                    ),
                    true,
                )?;
                if body.is_empty() || body.trim_start().starts_with('<') {
                    return Err(
                        "server returned an empty input or HTML; input cache unchanged".into(),
                    );
                }
                atomic_write(&path, body.as_bytes())?;
                println!("{}", path.display());
            }
        }
        Action::Puzzle {
            day,
            offline,
            refresh,
        } => {
            let path = data_dir(&root, &day).join("puzzle.html");
            let html = if path.is_file() && !refresh {
                fs::read_to_string(&path)?
            } else if offline {
                return Err("puzzle is not cached; offline mode makes no network requests".into());
            } else {
                let html = download(
                    &format!("https://adventofcode.com/{}/day/{}", day.year, day.day),
                    false,
                )?;
                puzzle_text(&html)?; // Validate before replacing a good cache.
                atomic_write(&path, html.as_bytes())?;
                html
            };
            println!("{}", puzzle_text(&html)?);
        }
        Action::List => {
            for lang in &cfg.languages {
                println!(
                    "{}\t.{}\t{}",
                    lang.name,
                    lang.extension,
                    lang.adapter.as_deref().unwrap_or("custom commands")
                );
            }
        }
        Action::Doctor { lang } => {
            let selected: Vec<_> = if let Some(name) = lang {
                vec![language(&cfg, &name)?]
            } else {
                cfg.languages.iter().collect()
            };
            let mut failed = false;
            // The packaged CLI does not require Rust in another language's shell.
            // Rust's adapter checks cargo and rustc when Rust is selected.
            for name in ["curl"] {
                match Command::new(name).arg("--version").output() {
                    Ok(out) if out.status.success() => println!(
                        "OK {name}: {}",
                        String::from_utf8_lossy(&out.stdout)
                            .lines()
                            .next()
                            .unwrap_or("available")
                    ),
                    _ => {
                        eprintln!("MISSING {name}");
                        failed = true;
                    }
                }
            }
            println!(
                "AoC session: {}",
                if env::var_os("AOC_SESSION").is_some_and(|s| !s.is_empty()) {
                    "set (not displayed or network-validated)"
                } else {
                    "not set; only authenticated downloads require it"
                }
            );
            for lang in selected {
                if lang.adapter.is_some() {
                    println!("Checking {}", lang.name);
                    let day = Day { year: 2021, day: 1 };
                    if let Err(error) = status(
                        adapter_command(&root, lang, "doctor", &day, "both")?,
                        "doctor",
                        true,
                    ) {
                        eprintln!("{}: {error}", lang.name);
                        failed = true;
                    }
                } else {
                    println!("{}: custom run/test commands registered; validate with aoc test (doctor does not execute arbitrary custom commands)", lang.name);
                }
            }
            if failed {
                return Err("one or more required tools are unavailable".into());
            }
        }
        Action::AddLanguage {
            name,
            extension,
            run,
            test,
            build,
            stub,
        } => {
            if !valid_identifier(&name) || !valid_identifier(&extension) {
                return Err(
                    "language and extension must use lowercase letters, digits, '-' or '_'".into(),
                );
            }
            if cfg.languages.iter().any(|l| l.name == name) {
                return Err(format!("language {name} is already registered").into());
            }
            if run.trim().is_empty() || test.trim().is_empty() {
                return Err("run and test commands cannot be empty".into());
            }
            let template = format!("templates/{name}/day.{extension}");
            let path = root.join(&template);
            if path.exists() {
                return Err(format!("refusing to overwrite {}", path.display()).into());
            }
            cfg.languages.push(Language {
                name: name.clone(),
                extension,
                adapter: None,
                run: Some(run),
                test: Some(test),
                build,
                template: Some(template),
            });
            let contents = toml::to_string_pretty(&cfg)?;
            write_new(&path, stub.as_bytes())?;
            if let Err(error) = atomic_write(&root.join("aoc.toml"), contents.as_bytes()) {
                let _ = fs::remove_file(&path);
                return Err(error);
            }
            println!(
                "Registered {name}; edit {} for an idiomatic empty stub and inline sample tests.",
                path.display()
            );
        }
    }
    Ok(())
}
fn download(url: &str, require_session: bool) -> Result<String> {
    let session = env::var("AOC_SESSION").ok().filter(|s| !s.is_empty());
    if require_session && session.is_none() {
        return Err("AOC_SESSION is missing; set it securely in your shell/environment settings, then retry fetch".into());
    }
    if let Some(value) = &session {
        if !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-._~".contains(&b))
        {
            return Err(
                "AOC_SESSION contains characters that cannot be used safely in a cookie".into(),
            );
        }
    }
    // Cookie goes through curl's stdin configuration, never argv, logs, or a file.
    let mut curl_config = format!("url = \"{url}\"\n");
    if let Some(value) = session {
        curl_config.push_str(&format!("header = \"Cookie: session={value}\"\n"));
    }
    let mut child = Command::new("curl")
        .args([
            "--disable",
            "--config",
            "-",
            "--silent",
            "--show-error",
            "--fail",
            "--proto",
            "=https",
            "--connect-timeout",
            "15",
            "--max-time",
            "60",
            "--user-agent",
            "advent-dx-toolkit/0.1 (local personal Advent of Code client)",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("curl stdin missing")?
        .write_all(curl_config.as_bytes())?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!("download failed ({}); check AoC access, session, and network settings; cache unchanged", output.status).into());
    }
    Ok(String::from_utf8(output.stdout)?)
}
fn puzzle_text(html: &str) -> Result<String> {
    let doc = Html::parse_document(html);
    let article = Selector::parse("article.day-desc").unwrap();
    let block = Selector::parse("h2, p, pre, li").unwrap();
    let mut chunks = Vec::new();
    for section in doc.select(&article) {
        for node in section.select(&block) {
            let text = node.text().collect::<String>();
            if !text.trim().is_empty() {
                chunks.push(text.trim_end().to_owned());
            }
        }
    }
    if chunks.is_empty() {
        return Err("response has no puzzle description; cache unchanged".into());
    }
    Ok(chunks.join("\n\n"))
}
fn watch_snapshot(
    root: &Path,
    input: Option<&Path>,
) -> Result<BTreeMap<PathBuf, (SystemTime, u64)>> {
    let mut state = snapshot(root)?;
    if let Some(input) = input {
        if input.is_file() {
            let meta = fs::metadata(input)?;
            state.insert(input.canonicalize()?, (meta.modified()?, meta.len()));
        }
    }
    Ok(state)
}
fn snapshot(root: &Path) -> Result<BTreeMap<PathBuf, (SystemTime, u64)>> {
    fn visit(path: &Path, out: &mut BTreeMap<PathBuf, (SystemTime, u64)>) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if [
                ".git",
                "target",
                "_build",
                "deps",
                "dist-newstyle",
                ".stack-work",
                ".scala-build",
                ".bsp",
                ".bloop",
                ".metals",
                ".elixir_ls",
                ".aoc-cache",
                ".aoc",
            ]
            .contains(&name.as_ref())
            {
                continue;
            }
            let kind = entry.file_type()?;
            if kind.is_dir() {
                visit(&entry.path(), out)?;
            } else if kind.is_file() {
                let meta = entry.metadata()?;
                out.insert(entry.path(), (meta.modified()?, meta.len()));
            }
        }
        Ok(())
    }
    let mut out = BTreeMap::new();
    visit(root, &mut out)?;
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_years() {
        assert_eq!(parse_year("22").unwrap(), 2022);
        assert!(parse_year("2014").is_err());
        assert!(parse_year("../../x").is_err());
    }
    #[test]
    fn renders_only_puzzle_articles() {
        let html = "<header>private navigation</header><article class='day-desc'><h2>Day 1</h2><p>A &lt; B &amp; C</p><pre><code>x\ny</code></pre></article>";
        assert_eq!(puzzle_text(html).unwrap(), "Day 1\n\nA < B & C\n\nx\ny");
        assert!(puzzle_text("<html>please log in</html>").is_err());
    }
    #[test]
    fn atomic_replacement_and_no_overwrite() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("input.txt");
        write_new(&path, b"old").unwrap();
        assert!(write_new(&path, b"bad").is_err());
        atomic_write(&path, b"new").unwrap();
        assert_eq!(fs::read(path).unwrap(), b"new");
    }
    #[test]
    fn shell_path_is_single_argument() {
        assert_eq!(shell_quote("one ' two"), "'one '\\'' two'");
    }
}
