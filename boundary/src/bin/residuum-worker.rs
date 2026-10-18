//! Minimal persistent worker: `residuum-worker --manifest <file>`. Reads one
//! request per line from stdin, writes one protocol line per request to
//! stdout, diagnostics to stderr, and exits with the documented code on EOF
//! or a terminal stop. All logic lives in `residuum_boundary::startup`.

use std::io::{BufReader, BufWriter, Write};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let stderr = std::io::stderr();
    let mut input = BufReader::new(stdin.lock());
    let mut output = BufWriter::new(stdout.lock());
    let mut diagnostics = stderr.lock();
    let exit = residuum_boundary::startup::run(&args, &mut input, &mut output, &mut diagnostics);
    let _ = output.flush();
    std::process::exit(exit.code());
}
