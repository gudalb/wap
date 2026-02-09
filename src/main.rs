use clap::Parser;
use notify::{Event, EventKind, Result, Watcher};
use std::{
    path::Path,
    process::Command,
    string,
    sync::mpsc::{self, RecvTimeoutError},
    time::Duration,
};

const DEFAULT_TIMEOUT_MS: u64 = 5000;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    path: String,
    #[arg(short, long)]
    timeoutms: Option<u64>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = Path::new(&args.path);
    let timeout_ms;

    match &args.timeoutms {
        Some(timeout) => {
            timeout_ms = timeout;
            println!("using timeout: {}", timeout_ms);
        }
        None => {
            println!(
                "timeout not supplied, using default {}ms",
                DEFAULT_TIMEOUT_MS
            );
            timeout_ms = &DEFAULT_TIMEOUT_MS;
        }
    };

    if !path.is_dir() {
        eprintln!(
            "Error: Path does not exist or is not a directory: {}",
            args.path
        );
        std::process::exit(1);
    }

    if !path.has_root() {
        eprintln!("Error: Path cannot be relative: {}", args.path);
        std::process::exit(1);
    }

    let (tx, rx) = mpsc::channel::<Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(path, notify::RecursiveMode::Recursive)?;

    // unwatch does not work on macos, event filtering based on path?
    // watcher.unwatch(&path.join(".git"))?;

    let mut has_changes = false;

    println!("Starting to monitor changes at: {}", path.display());

    loop {
        match rx.recv_timeout(Duration::from_millis(*timeout_ms)) {
            Ok(Ok(event)) => match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                    has_changes = true;
                    for ele in event.paths.iter() {
                        let res = ele.to_str();
                        match res {
                            Some(res) => {
                                println!("{}", res);
                                if (res.contains(".git")) {
                                    has_changes = false
                                }
                            }
                            _ => {}
                        }
                        println!()
                    }
                }
                _ => {}
            },
            Err(RecvTimeoutError::Timeout) => {
                if has_changes {
                    // TODO: improve commit message
                    println!("change detected, pushing");
                    let commit_message = "WIP message";

                    Command::new("git")
                        .current_dir(path)
                        .args(["add", "-A"])
                        .status()?;

                    Command::new("git")
                        .current_dir(path)
                        .args(["commit", "-m", commit_message])
                        .status()?;

                    Command::new("git")
                        .current_dir(path)
                        .args(["push"])
                        .status()?;
                } else {
                    println!("no change detected");
                }
                has_changes = false;
            }
            Err(RecvTimeoutError::Disconnected) => break,
            _ => {}
        }
    }

    Ok(())
}
