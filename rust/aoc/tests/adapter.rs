use std::{fs, path::PathBuf, process::Command};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "aoc-rust-adapter-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("rust/aoc/src")).unwrap();
        fs::write(
            root.join("rust/Cargo.toml"),
            "[workspace]\nresolver = \"2\"\nmembers = [\"aoc\", \"years/*\"]\n",
        )
        .unwrap();
        fs::write(
            root.join("rust/aoc/Cargo.toml"),
            include_str!("../Cargo.toml"),
        )
        .unwrap();
        fs::write(
            root.join("rust/aoc/src/lib.rs"),
            include_str!("../src/lib.rs"),
        )
        .unwrap();
        fs::write(
            root.join("rust/aoc/src/input.rs"),
            include_str!("../src/input.rs"),
        )
        .unwrap();
        Self(root)
    }

    fn adapter(&self, arguments: &[&str]) -> std::process::Output {
        Command::new("bash")
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../scripts/languages/rust.sh"
            ))
            .args(arguments)
            .env("AOC_ROOT", &self.0)
            .env("CARGO_TARGET_DIR", self.0.join("target"))
            .output()
            .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn generated_days_build_remain_unsolved_and_preserve_existing_files() {
    let fixture = Fixture::new();
    let generated = fixture.adapter(&["new", "2025", "03"]);
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let source = fixture.0.join("rust/years/y2025/src/d03.rs");
    let before = fs::read(&source).unwrap();
    assert!(!fixture.adapter(&["new", "2025", "3"]).status.success());
    assert_eq!(before, fs::read(&source).unwrap());
    assert!(fixture.adapter(&["new", "2025", "4"]).status.success());
    let build = fixture.adapter(&["build", "2025"]);
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let sample = fixture.adapter(&["test", "2025", "3"]);
    assert!(!sample.status.success());
    assert!(String::from_utf8_lossy(&sample.stderr).contains("incomplete"));
    let run = fixture.adapter(&["run", "2025", "3", "1"]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("unavailable"));
    assert!(!fixture.adapter(&["new", "2025", "26"]).status.success());

    let registry = fixture.0.join("rust/years/y2025/src/main.rs");
    fs::write(&registry, "fn main() {}\n").unwrap();
    assert!(!fixture.adapter(&["new", "2025", "5"]).status.success());
    assert!(!fixture.0.join("rust/years/y2025/src/d05.rs").exists());
    assert_eq!(fs::read_to_string(registry).unwrap(), "fn main() {}\n");
}
