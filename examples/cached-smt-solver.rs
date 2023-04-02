//! # `cached-smt-solver`
//!
//! This is a `sat-cache` interface for caching SMT-LIB2 queries. For
//! example, adding the binary for this example to your $PATH, along
//! with appropriately named shell scripts (ahead in $PATH of the
//! actual SMT binary) allows SMT queries to tools like Cryptol to be
//! cached.
//!
//! For example:
//!
//! ```z3
//! SAT_CACHE_SOLVER=/usr/local/bin/z3 cached-smt-solver $@
//! ```
//!
//! The first call takes noticable time:
//!
//! $ cryptol
//! ┏━╸┏━┓╻ ╻┏━┓╺┳╸┏━┓╻
//! ┃  ┣┳┛┗┳┛┣━┛ ┃ ┃ ┃┃
//! ┗━╸╹┗╸ ╹ ╹   ╹ ┗━┛┗━╸
//! version 2.13.0.99 (7e2613a, modified)
//!
//! Loading module Cryptol
//! Cryptol> :s prover=w4-z3
//! Cryptol> :sat \(x : [64]) -> x > 1000000 /\ x < 9999999 /\ (x * x) % 10000000 == x
//! Satisfiable
//! (\(x : [64]) -> x > 1000000 /\ x < 9999999 /\ (x * x) % 10000000 == x)
//!   0x00000000006c7b00
//!   = True
//! (Total Elapsed Time: 8.487s, using "Z3")
//!
//! Repeat calls return instantly, even between Cryptol sessions
//! (since the `SQLite` database persists).
//!
//! Cryptol> :sat \(x : [64]) -> x > 1000000 /\ x < 9999999 /\ (x * x) % 10000000 == x
//! Satisfiable
//! (\(x : [64]) -> x > 1000000 /\ x < 9999999 /\ (x * x) % 10000000 == x)
//!   0x00000000006c7b00
//!   = True
//! (Total Elapsed Time: 0.045s, using "Z3")

#![forbid(unsafe_code)]

use signal_hook::consts::TERM_SIGNALS;
use signal_hook::flag;
use std::env;
use std::sync::mpsc::channel;
use std::sync::{atomic::AtomicBool, Arc};

pub fn main() {
    let solver = match env::var("SAT_CACHE_SOLVER") {
        Ok(v) => v,
        Err(e) => panic!("SAT_CACHE_SOLVER is not set ({e})"),
    };

    let print_success = env::var("SAT_CACHE_PRINT_SUCCESS").is_ok();

    // Ask signal_hook to set the term variable to true
    // when the program receives a kill signal.
    let should_terminate = Arc::new(AtomicBool::new(false));
    // Register all kill signals.
    for sig in TERM_SIGNALS {
        flag::register_conditional_shutdown(*sig, 1, Arc::clone(&should_terminate)).unwrap();
        flag::register(*sig, Arc::clone(&should_terminate)).unwrap();
    }

    let (solver_sender, from_solver) = channel();
    let (to_solver, solver_receiver) = channel();
    let args: Vec<String> = env::args().collect();
    let inputs: Vec<&str> = args.iter().map(AsRef::as_ref).collect();
    let mut process = satcache::app::start_process(
        &solver,
        inputs[1..].to_vec(),
        solver_sender,
        solver_receiver,
        print_success,
    );

    satcache::simple_smt_transaction(&to_solver, &from_solver, &should_terminate, print_success);

    process.kill().unwrap();
}
