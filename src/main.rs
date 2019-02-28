extern crate ansi_term;
extern crate libc;
extern crate openpty;

use ansi_term::Colour::Red;
use openpty::openpty;
use std::env;
use std::env::ArgsOs;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, ErrorKind::NotFound, Read, Write};
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};
use std::process::{self, Child, Command, Stdio};
use std::thread;

fn spawn_child<S: AsRef<OsStr>>(prog_name: S, argv: ArgsOs) -> Result<(Child, File, File), ()> {
    let mut cmd = Command::new(prog_name);
    cmd.args(argv);

    if unsafe { libc::isatty(libc::STDIN_FILENO) != 0 } {
        let (master, slave, _) = openpty(None, None, None).map_err(|_| ())?;
        let (stderr_master, stderr_slave, _) = openpty(None, None, None).map_err(|_| ())?;

        cmd.stdin(slave.try_clone().expect("could not clone slave fd"))
            .stdout(slave)
            .stderr(stderr_slave);

        let proc = _spawn_child_from_cmd(cmd)?;

        Ok((proc, master, stderr_master))
    } else {
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut proc = _spawn_child_from_cmd(cmd)?;

        // convert process::ChildStd{out,err} to File
        let stdout = unsafe { File::from_raw_fd(proc.stdout.take().unwrap().into_raw_fd()) };
        let stderr = unsafe { File::from_raw_fd(proc.stderr.take().unwrap().into_raw_fd()) };

        Ok((proc, stdout, stderr))
    }
}

fn _spawn_child_from_cmd(mut proc: Command) -> Result<Child, ()> {
    match proc.spawn() {
        Ok(child) => Ok(child),
        Err(ref e) if e.kind() == NotFound => {
            eprintln!(
                "launchred: {}: command not found",
                env::args().nth(1).unwrap()
            );
            return Err(());
        }
        Err(e) => {
            eprintln!("launchred: unknown error {:?}", e);
            return Err(());
        }
    }
}

fn set_blocking<F: AsRawFd>(fd: &mut F, blocking: bool) -> Result<(), ()> {
    match unsafe {
        let mut flags = libc::fcntl(fd.as_raw_fd(), libc::F_GETFL);
        if blocking {
          flags &= !libc::O_NONBLOCK;
        } else {
          flags |= libc::O_NONBLOCK;
        }
        libc::fcntl(fd.as_raw_fd(), libc::F_SETFL, flags)
    } {
        0 => Ok(()),
        _ => Err(()),
    }
}

fn run() -> Result<(), ()> {
    let mut argv = env::args_os();

    if let Some(prog_name) = argv.nth(1).filter(|s| s != "-h" && s != "--help") {
        let (mut proc, mut stdout, mut stderr) = spawn_child(prog_name, argv)?;
        let mut stdin = io::stdin();

        //set_blocking(&mut stdout, false)?;
        //set_blocking(&mut stderr, false)?;
        //set_blocking(&mut stdin, false)?;

        let stderr_thread = thread::spawn(move || {
            let mut buf: [u8; 1024] = [0; 1024];
            while let Ok(n) = stderr.read(&mut buf) {
                //io::stderr().write(&buf[0..n]);
                Red.paint(&buf[0..n]).write_to(&mut io::stderr());
            }
        });

        let mut buf: [u8; 1024] = [0; 1024];
        while let Ok(n) = stdout.read(&mut buf) {
            io::stdout().write(&buf[0..n]);
        }

        stderr_thread.join();

        /*if let Ok(true) = env::var("TERM").map(|x| x == "dumb") {
            for line in reader.lines() {
                eprintln!("[stderr] {}", line.unwrap());
            }
        } else {
            for line in reader.lines() {
                eprintln!("{}", Red.paint(line.unwrap()));
            }
        }*/

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
        return Err(());
    }
}

fn main() {
    std::process::exit(match run() {
        Ok(_) => 0,
        Err(_) => 1,
    });
}
