//! Start-up logic of the minimal worker binary, with injectable streams so
//! that it is tested without a process:
//!
//! ```text
//! --manifest <file> → parse and validate the manifest DTO → Worker::new
//!                   → serve(stdin, stdout) → exit on EOF or terminal stop
//! ```
//!
//! "Persistent" means only that sessions live across the lines of one
//! process. No daemon, socket, restart recovery or controller. stdout is
//! reserved for protocol lines; every diagnostic goes to stderr as one line
//! prefixed with `worker:`. The manifest is read once; no request can change
//! it, because the worker owns the converted value and the protocol has no
//! request that touches it.

use std::io::{BufRead, Write};
use std::path::Path;

use residuum::experiments::worker::Worker;

use crate::manifest::{parse_manifest, to_manifest};
use crate::transport::{Served, StopReason, serve};

/// Exit codes, stable for the runner and the future controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exit {
    /// Clean EOF after serving zero or more lines.
    Eof = 0,
    /// Command line not understood.
    Usage = 2,
    /// Manifest unreadable, oversized, malformed or invalid.
    Manifest = 3,
    /// The theory imports do not elaborate or the worker cannot start.
    Worker = 4,
    /// Input ended inside a line; that line was not executed.
    IncompleteLine = 5,
    /// Reading or writing the protocol streams failed.
    Stream = 6,
}

impl Exit {
    pub fn code(self) -> i32 {
        self as i32
    }
}

pub const MAX_MANIFEST_BYTES: usize = 4 * 1024 * 1024;

fn diag<E: Write>(stderr: &mut E, message: &str) {
    // Diagnostics never reach stdout; a failing stderr is ignored on purpose.
    let _ = writeln!(stderr, "worker: {message}");
}

/// Parse `--manifest <file>`; nothing else is accepted.
pub fn manifest_path(args: &[String]) -> Result<&str, String> {
    match args {
        [flag, path] if flag == "--manifest" && !path.is_empty() => Ok(path),
        _ => Err("usage: residuum-worker --manifest <file>".into()),
    }
}

/// Run the worker over the given streams. Returns the exit code; the
/// summary of what was served goes to stderr.
pub fn run<R: BufRead, W: Write, E: Write>(
    args: &[String],
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Exit {
    let path = match manifest_path(args) {
        Ok(p) => p,
        Err(m) => {
            diag(stderr, &m);
            return Exit::Usage;
        }
    };
    let path = Path::new(path);
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            diag(
                stderr,
                &format!("cannot read manifest `{}`: {e}", path.display()),
            );
            return Exit::Manifest;
        }
    };
    let wire = match parse_manifest(&bytes, MAX_MANIFEST_BYTES) {
        Ok(w) => w,
        Err(e) => {
            diag(stderr, &format!("manifest rejected: {e}"));
            return Exit::Manifest;
        }
    };
    let base = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let (manifest, limits) = match to_manifest(&wire, base) {
        Ok(x) => x,
        Err(e) => {
            diag(stderr, &format!("manifest rejected: {e}"));
            return Exit::Manifest;
        }
    };
    diag(
        stderr,
        &format!(
            "episode `{}` manifest `{}` loaded; {} dossiers; serving",
            manifest.episode_id,
            manifest.manifest_hash,
            manifest.dossiers.len()
        ),
    );
    let mut worker = match Worker::new(manifest) {
        Ok(w) => w,
        Err(e) => {
            diag(stderr, &format!("worker cannot start: {e}"));
            return Exit::Worker;
        }
    };
    let served = serve(stdin, stdout, &mut worker, &limits);
    report(stderr, &served)
}

fn report<E: Write>(stderr: &mut E, served: &Served) -> Exit {
    let exit = match &served.stop {
        StopReason::Eof => Exit::Eof,
        StopReason::IncompleteLine => Exit::IncompleteLine,
        StopReason::WriteError(_) => Exit::Stream,
    };
    let reason = match &served.stop {
        StopReason::Eof => "eof".to_string(),
        StopReason::IncompleteLine => "incomplete_line".to_string(),
        StopReason::WriteError(m) => format!("stream_error: {m}"),
    };
    diag(
        stderr,
        &format!(
            "stopped: {reason}; lines {} responses {} faults {}; exit {}",
            served.lines,
            served.responses,
            served.faults,
            exit.code()
        ),
    );
    exit
}
