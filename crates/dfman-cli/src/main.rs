use dfman_core::{DirectorySnapshot, LaunchContext};
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, Instant};

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  dfman scan <path>");
    eprintln!("  dfman benchmark <path> [--runs <n>]");
    eprintln!("  dfman open <path> [--left <path>] [--right <path>] [--select <path>]...");
}

fn take_single_path(mut args: impl Iterator<Item = String>) -> Result<String, String> {
    let Some(path) = args.next() else {
        print_usage();
        return Err("missing path".to_owned());
    };

    if args.next().is_some() {
        print_usage();
        return Err("too many arguments".to_owned());
    }

    Ok(path)
}

fn run_scan(args: impl Iterator<Item = String>) -> Result<(), String> {
    let path = take_single_path(args)?;
    let snapshot =
        DirectorySnapshot::scan(&path).map_err(|error| format!("cannot scan {path}: {error}"))?;

    println!("Snapshot: {}", snapshot.root.display());
    println!("Entries: {}", snapshot.entries.len());
    println!("Files: {}", snapshot.file_count());
    println!("Directories: {}", snapshot.directory_count());
    Ok(())
}

fn run_benchmark(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    let Some(path) = args.next() else {
        print_usage();
        return Err("missing path".to_owned());
    };

    let mut runs = 5_u32;

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--runs" => {
                let Some(value) = args.next() else {
                    return Err("--runs requires an integer".to_owned());
                };
                runs = value
                    .parse::<u32>()
                    .map_err(|_| "--runs must be a positive integer".to_owned())?;
                if runs == 0 {
                    return Err("--runs must be greater than zero".to_owned());
                }
            }
            _ => return Err(format!("unknown benchmark argument: {argument}")),
        }
    }

    let warmup_start = Instant::now();
    let warmup = DirectorySnapshot::scan(&path)
        .map_err(|error| format!("cannot scan {path}: {error}"))?;
    let warmup_elapsed = warmup_start.elapsed();

    let entries = warmup.entries.len();
    let files = warmup.file_count();
    let directories = warmup.directory_count();

    let mut durations = Vec::with_capacity(runs as usize);

    for _ in 0..runs {
        let start = Instant::now();
        let snapshot = DirectorySnapshot::scan(&path)
            .map_err(|error| format!("cannot scan {path}: {error}"))?;
        let elapsed = start.elapsed();

        if snapshot.entries.len() != entries {
            return Err("directory contents changed during benchmark".to_owned());
        }

        durations.push(elapsed);
    }

    let min = *durations.iter().min().expect("benchmark has at least one run");
    let max = *durations.iter().max().expect("benchmark has at least one run");
    let total = durations.iter().copied().sum::<Duration>();
    let average = total / runs;
    let entries_per_second = if average.is_zero() {
        f64::INFINITY
    } else {
        entries as f64 / average.as_secs_f64()
    };

    println!("dfman benchmark");
    println!("Backend: std::fs");
    println!("Path: {}", warmup.root.display());
    println!("Entries: {entries}");
    println!("Files: {files}");
    println!("Directories: {directories}");
    println!("Warmup: {:.3} ms", warmup_elapsed.as_secs_f64() * 1_000.0);
    println!("Runs: {runs}");
    println!("Min: {:.3} ms", min.as_secs_f64() * 1_000.0);
    println!("Average: {:.3} ms", average.as_secs_f64() * 1_000.0);
    println!("Max: {:.3} ms", max.as_secs_f64() * 1_000.0);
    println!("Rate: {:.0} entries/s", entries_per_second);

    Ok(())
}

fn run_open(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    let Some(path) = args.next() else {
        print_usage();
        return Err("missing path".to_owned());
    };

    let mut context = LaunchContext::at(path);

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--left" => {
                let Some(value) = args.next() else {
                    return Err("--left requires a path".to_owned());
                };
                context.left_path = Some(PathBuf::from(value));
            }
            "--right" => {
                let Some(value) = args.next() else {
                    return Err("--right requires a path".to_owned());
                };
                context.right_path = Some(PathBuf::from(value));
            }
            "--select" => {
                let Some(value) = args.next() else {
                    return Err("--select requires a path".to_owned());
                };
                context.selected_entries.push(PathBuf::from(value));
            }
            _ => return Err(format!("unknown open argument: {argument}")),
        }
    }

    println!("Launch context");
    println!("Current: {}", context.current_path.display());
    println!(
        "Left: {}",
        context
            .left_path
            .as_ref()
            .map_or_else(|| "<default>".to_owned(), |path| path.display().to_string())
    );
    println!(
        "Right: {}",
        context
            .right_path
            .as_ref()
            .map_or_else(|| "<default>".to_owned(), |path| path.display().to_string())
    );
    println!("Initial basket entries: {}", context.selected_entries.len());

    for entry in &context.selected_entries {
        println!("  + {}", entry.display());
    }

    Ok(())
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);

    let Some(command) = args.next() else {
        print_usage();
        return Err("missing command".to_owned());
    };

    match command.as_str() {
        "scan" => run_scan(args),
        "benchmark" => run_benchmark(args),
        "open" => run_open(args),
        _ => {
            print_usage();
            Err(format!("unknown command: {command}"))
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("dfman: {error}");
            ExitCode::FAILURE
        }
    }
}
