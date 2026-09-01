use dfman_core::DirectorySnapshot;
use std::env;
use std::process::ExitCode;

fn print_usage() {
    eprintln!("Usage: dfman scan <path>");
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);

    let Some(command) = args.next() else {
        print_usage();
        return Err("missing command".to_owned());
    };

    match command.as_str() {
        "scan" => {
            let Some(path) = args.next() else {
                print_usage();
                return Err("missing path".to_owned());
            };

            if args.next().is_some() {
                print_usage();
                return Err("too many arguments".to_owned());
            }

            let snapshot = DirectorySnapshot::scan(&path)
                .map_err(|error| format!("cannot scan {path}: {error}"))?;

            println!("Snapshot: {}", snapshot.root.display());
            println!("Entries: {}", snapshot.entries.len());
            println!("Files: {}", snapshot.file_count());
            println!("Directories: {}", snapshot.directory_count());
            Ok(())
        }
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
