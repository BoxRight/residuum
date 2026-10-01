//! Experimental runner; execution goes through scripts/test_safe.py.

fn read_fixture(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let metadata = std::fs::metadata(path)?;
    if !metadata.is_file() || metadata.len() > 64 * 1024 {
        return Err("expected a regular fixture of at most 64 KiB".into());
    }
    let source = std::fs::read_to_string(path)?;
    if source.len() > 64 * 1024 {
        return Err("fixture exceeds the 64 KiB experimental limit".into());
    }
    Ok(source)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let first = arguments.next().ok_or("expected one Formal fixture path")?;
    let incremental = first == "--incremental";
    let forward = first == "--forward";
    let path = if incremental || forward {
        arguments
            .next()
            .ok_or("expected one Formal fixture path after mode flag")?
    } else {
        first
    };
    let library_path = match arguments.next() {
        None => None,
        Some(flag) if incremental && flag == "--library" => {
            Some(arguments.next().ok_or("expected library path")?)
        }
        Some(_) => {
            return Err(
                "expected one Formal fixture path and optional --library in incremental mode"
                    .into(),
            );
        }
    };
    if arguments.next().is_some() {
        return Err("unexpected fixture argument".into());
    }
    let source = read_fixture(&path)?;
    let module = if let Some(library_path) = library_path {
        let library = read_fixture(&library_path)?;
        if library.len() + source.len() > 64 * 1024 {
            return Err("combined fixtures exceed the 64 KiB experimental limit".into());
        }
        residuum::experiments::elaborate_composed_fixture(
            residuum::parse_formal(&library)?,
            residuum::parse_formal(&source)?,
        )?
    } else {
        residuum::parse_formal_to_typed(&source)?
    };
    if incremental {
        let result = residuum::experiments::run_incremental_experiment(&module)?;
        print!("{}", residuum::experiments::render_incremental(&result));
    } else if forward {
        let results = residuum::experiments::run_forward_experiments(&module)?;
        print!("{}", residuum::experiments::render_results(&results));
    } else {
        let results = residuum::experiments::run_experiments(&module)?;
        print!("{}", residuum::experiments::render_results(&results));
    }
    Ok(())
}
