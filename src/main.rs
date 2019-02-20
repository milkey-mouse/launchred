//extern crate ansi_term;

use ansi_term::Colour::Red;
use std::env;
use std::io::{BufRead, BufReader, ErrorKind::NotFound};
use std::process::{self, Command, Stdio};

fn main() {
    let mut argv = env::args_os();

    if let Some(prog_name) = argv.nth(1).filter(|s| s != "-h" && s != "--help") {
        // TODO: spawn child in pty like in backup-vm
        let mut proc = match Command::new(prog_name)
            .args(argv)
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(x) => x,
            Err(ref e) if e.kind() == NotFound => {
                eprintln!(
                    "launchred: {}: command not found",
                    env::args().nth(1).unwrap()
                );
                process::exit(1);
            }
            Err(e) => {
                eprintln!("launchred: unknown error {:?}", e);
                process::exit(1);
            }
        };

        let reader = BufReader::new(proc.stderr.take().unwrap());

        if let Ok(true) = env::var("TERM").map(|x| x == "dumb") {
            for line in reader.lines() {
                eprintln!("[stderr] {}", line.unwrap());
            }
        } else {
            for line in reader.lines() {
                eprintln!("{}", Red.paint(line.unwrap()));
            }
        }

        // TODO: when terminated by a signal 128 + signum should be exit code
        // see https://unix.stackexchange.com/q/99112#99134
        process::exit(match proc.try_wait() {
            Ok(Some(x)) => x.code().unwrap_or(1),
            //Ok(None) => proc.wait().map().map_or_else(|_| 1, |x| x.code().unwrap_or(1)),
            Ok(None) => proc.wait().map(|x| x.code().unwrap_or(1)).unwrap_or(1),
            Err(_e) => {
                eprintln!("launchred: error while waiting for child process");
                1
            }
        });
    } else {
        eprintln!("usage: launchred [-h|--help] COMMAND [ARGS...]");
        process::exit(1);
    }
}
