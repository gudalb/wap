use clap::Parser;
use notify::{Event, EventKind, Result, Watcher};
use std::{
    path::Path,
    process::Command,
    sync::mpsc::{self, RecvTimeoutError},
    time::Duration,
};

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
    let timeoutms;

    match &args.timeoutms {
        Some(timeout) => {
            timeoutms = timeout;
            println!("using timeout: {}", timeoutms);
        }
        None => {
            println!("timeout not supplied, using default 5000ms");
            timeoutms = &5000;
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

    let mut has_changes = false;

    println!("Starting to monitor changes at: {}", path.display());

    loop {
        match rx.recv_timeout(Duration::from_millis(*timeoutms)) {
            Ok(Ok(event)) => match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                    has_changes = true;
                }
                _ => {}
            },
            Err(RecvTimeoutError::Timeout) => {
                if has_changes {
                    // TODO: improve commit message
                    let commit_message = String::from("message WIP");

                    if cfg!(target_os = "linux") {
                        let commit_command = Command::new("sh")
                            .arg(format!(
                                "cd {} && git add -A && git commit -m '{}'",
                                path.display(),
                                commit_message
                            ))
                            .output()
                            .expect("Failed to run commit command");

                        println!("commit status: {}", commit_command.status);

                        let push_command = Command::new("sh")
                            .arg(format!("cd {} && git push", path.display()))
                            .output()
                            .expect("Failed to run commit command");

                        println!("push status: {}", push_command.status);
                    } else {
                        println! {"could not commit and push. only linux supported"};
                    }
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
