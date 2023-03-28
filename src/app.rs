// Gleaned from https://www.nikbrendler.com/rust-process-communication-part-2/

#![forbid(unsafe_code)]

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Receiver, Sender};

use std::thread;

fn start_process_thread(child: &mut Child, sender: Sender<String>, receiver: Receiver<String>) {
    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    thread::spawn(move || {
        let mut f = BufReader::new(stdout);
        loop {
            let line = receiver.recv().unwrap();
            writeln!(stdin, "{line}").unwrap();
            if line.eq("(exit)") {
                break;
            }
            let mut buf = String::new();
            f.read_line(&mut buf).unwrap();
            sender.send(buf).unwrap();
        }
    });
}

/// This function is a wrapper around a stateless binary application,
/// such as a SAT or SMT solver, that accepts multiple queries in a
/// session. The binary `app` must be in $PATH.

#[must_use]
pub fn start_process(
    app: &str,
    args: Vec<&str>,
    sender: Sender<String>,
    receiver: Receiver<String>,
) -> Child {
    let mut child = Command::new(app)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args(args)
        .spawn()
        .expect("Failed to spawn `{app} {args}` child application.");

    start_process_thread(&mut child, sender, receiver);

    child
}
