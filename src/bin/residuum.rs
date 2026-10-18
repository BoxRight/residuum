//! Internal worker. The repository's `residuum` launcher establishes limits.
use std::io;
use std::path::{Path, PathBuf};

#[path = "residuum/cli.rs"]
mod cli;
use cli::{Command, USAGE};

fn require_limits() -> Result<(), Box<dyn std::error::Error>> {
    let membership = std::fs::read_to_string("/proc/self/cgroup")?;
    let group = membership
        .lines()
        .find_map(|line| line.strip_prefix("0::"))
        .ok_or("cgroup v2 is required")?;
    let path = Path::new("/sys/fs/cgroup").join(group.trim_start_matches('/'));
    if std::fs::read_to_string(path.join("memory.max"))?.trim() != "1073741824"
        || std::fs::read_to_string(path.join("memory.swap.max"))?.trim() != "0"
    {
        return Err("refusing unbounded execution; use the repository's residuum launcher".into());
    }
    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
        print!("{USAGE}");
        return Ok(());
    }
    if args.len() < 2 {
        return Err(USAGE.into());
    }
    let command = Command::parse(args[0].to_str().ok_or("invalid command encoding")?)?;
    let steps = cli::StepOptions::parse(&args[2..])?;
    require_limits()?;
    let path = PathBuf::from(&args[1]);
    let output = if let Some(options) = steps {
        cli::execute_file_steps(command, &path, options)?
    } else {
        cli::execute_file(command, &path)?
    };
    use std::io::Write;
    io::stdout().lock().write_all(output.as_bytes())?;
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("residuum: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
