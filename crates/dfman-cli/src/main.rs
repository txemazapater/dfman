use dfman_core::{DirectorySnapshot, LaunchContext};
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  dfman scan <path>");
    eprintln!("  dfman open <path> [--left <path>] [--right <path>] [--select <path>]...");
}

fn run_scan(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    let Some(path) = args.next() else {
        print_usage();
        return Err("missing path".to_owned());
    };

    if args.next().is_some() {
        print_usage();
        return Err("too many arguments".to_owned());
    }

    let snapshot =
        DirectorySnapshot::scan(&path).map_err(|error| format!("cannot scan {path}: {error}"))?;

    println!("Snapshot: {}", snapshot.root.display());
    println!("Entries: {}", snapshot.entries.len());
    println!("Files: {}", snapshot.file_count());
    println!("Directories: {}", snapshot.directory_count());
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
