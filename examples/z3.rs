//! # `z3`
//!
//! This is a `sat-cache` interface for caching `z3` SMT-LIB2
//! queries. For example, adding the binary for this example to your
//! $PATH (ahead of the actual `z3` binary) allows SMT queries to
//! tools like Cryptol to be cached.
//!
//! The first call takes a couple of minutes:
//!
//! ```
//! Cryptol> :s satNum=all
//! Cryptol> :sat \(x : [64]) -> x > 100000 /\ x < 999999 /\ (x * x) % 1000000 == x
//! Satisfiable
//! (\(x : [64]) -> x > 100000 /\ x < 999999 /\ (x * x) % 1000000 == x)
//!   0x000000000001ab40
//!   = True
//! (\(x : [64]) -> x > 100000 /\ x < 999999 /\ (x * x) % 1000000 == x)
//!   0x00000000000d9701
//!   = True
//! Models found: 2
//! (Total Elapsed Time: 9.824s, using "Z3")
//! ```
//!
//! Repeat calls return instantly, even between Cryptol sessions
//! (since the `SQLite` database persists).
//!
//! ```
//! Cryptol> :sat \(x : [64]) -> x > 100000 /\ x < 999999 /\ (x * x) % 1000000 == x
//! Satisfiable
//! (\(x : [64]) -> x > 100000 /\ x < 999999 /\ (x * x) % 1000000 == x)
//!   0x000000000001ab40
//!   = True
//! (\(x : [64]) -> x > 100000 /\ x < 999999 /\ (x * x) % 1000000 == x)
//!   0x00000000000d9701
//!   = True
//! Models found: 2
//! (Total Elapsed Time: 0.080s, using "Z3")//! Cryptol> :s prover=w4-yices
//! ```

#![forbid(unsafe_code)]

use std::sync::mpsc::channel;

pub fn main() {
    let (z3_sender, from_z3) = channel();
    let (to_z3, z3_receiver) = channel();
    let mut z3 = satcache::app::start_process(
        "/usr/local/bin/z3",
        vec!["-smt2", "-in"],
        z3_sender,
        z3_receiver,
    );
    satcache::simple_smt_transaction(&to_z3, &from_z3);
    //z3.kill().unwrap(); // Just in case it's still running.
}
