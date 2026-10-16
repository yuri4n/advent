use std::{
    fs::{self, File},
    io::{self, Read, Seek},
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::Result;

/// Replayable sources never retain the complete input in RAM.
/// A single stdin execution can bypass spooling entirely.
pub enum Source {
    Path(PathBuf),
    Spool(File),
    Stdin,
}

impl Source {
    pub fn path(path: PathBuf) -> Result<Self> {
        fs::File::open(&path).map_err(|error| {
            format!(
                "cannot read {}: {error}; use aoc fetch, --input PATH, or --stdin",
                path.display()
            )
        })?;
        Ok(Self::Path(path))
    }

    pub fn spool(mut reader: impl Read) -> Result<Self> {
        // An anonymous private temporary file is removed even on process exit.
        let mut file = tempfile::tempfile()?;
        io::copy(&mut reader, &mut file)?;
        Ok(Self::Spool(file))
    }

    pub fn stdin(replay: bool) -> Result<Self> {
        if replay {
            Self::spool(io::stdin().lock())
        } else {
            Ok(Self::Stdin)
        }
    }

    pub fn capture(mut command: Command, label: &str) -> Result<Self> {
        let file = tempfile::tempfile()?;
        let status = command
            .stdin(Stdio::null())
            .stdout(file.try_clone()?)
            .stderr(Stdio::inherit())
            .status()?;
        if !status.success() {
            return Err(format!("{label} failed ({status})").into());
        }
        Ok(Self::Spool(file))
    }

    pub fn clipboard() -> Result<Self> {
        let words = if let Ok(configured) = std::env::var("AOC_CLIPBOARD_COMMAND") {
            shlex::split(&configured).ok_or("AOC_CLIPBOARD_COMMAND has unmatched quotes")?
        } else {
            let candidates: &[&[&str]] = if cfg!(target_os = "macos") {
                &[&["pbpaste"]]
            } else {
                &[
                    &["wl-paste", "--no-newline"],
                    &["xclip", "-selection", "clipboard", "-out"],
                    &["xsel", "--clipboard", "--output"],
                ]
            };
            candidates.iter().find(|args| crate::executable_on_path(args[0]).is_some())
                .ok_or("no clipboard reader found; install pbpaste (macOS), wl-paste (Wayland), or xclip/xsel (X11), or pipe input with --stdin")?
                .iter().map(|word| word.to_string()).collect()
        };
        let (program, arguments) = words
            .split_first()
            .ok_or("clipboard command cannot be empty")?;
        let mut command = Command::new(program);
        command.args(arguments);
        Self::capture(command, "clipboard read")
    }

    pub fn open(&self) -> Result<Stdio> {
        Ok(match self {
            Self::Path(path) => Stdio::from(File::open(path)?),
            Self::Spool(file) => {
                let mut handle = file.try_clone()?;
                handle.rewind()?;
                Stdio::from(handle)
            }
            Self::Stdin => Stdio::inherit(),
        })
    }
}
