//! # `yices`
//!
//! This is a `sat-cache` interface for caching `yices` SMT-LIB2
//! queries. For example, adding the binary for this example to your
//! $PATH (ahead of the actual `yices` binary) allows SMT queries to
//! tools like Cryptol to be cached.
//!
//! The first call takes a couple of minutes:
//!
//! ```
//! Cryptol> :s prover=w4-yices
//! Cryptol> :s satNum=all
//! Cryptol> :sat \(x : [64]) -> x > 1000000 /\ x <= 9999999 /\ (x * x) % 10000000 == x
//! Satisfiable
//! (\(x : [64]) -> x > 1000000 /\ x <= 9999999 /\ (x * x) % 10000000 == x)
//!   0x00000000002c1b81
//!   = True
//! (\(x : [64]) -> x > 1000000 /\ x <= 9999999 /\ (x * x) % 10000000 == x)
//!   0x00000000006c7b00
//!   = True
//! Models found: 2
//! (Total Elapsed Time: 2m:10.427s, using "Yices")
//! ```
//!
//! Repeat calls return instantly, even between Cryptol sessions
//! (since the `SQLite` database persists).
//!
//! ```
//! Cryptol> :sat \(x : [64]) -> x > 1000000 /\ x <= 9999999 /\ (x * x) % 10000000 == x
//! Satisfiable
//! (\(x : [64]) -> x > 1000000 /\ x <= 9999999 /\ (x * x) % 10000000 == x)
//!   0x00000000002c1b81
//!   = True
//! (\(x : [64]) -> x > 1000000 /\ x <= 9999999 /\ (x * x) % 10000000 == x)
//!   0x00000000006c7b00
//!   = True
//! Models found: 2
//! (Total Elapsed Time: 0.030s, using "Yices")
//! ```

#![forbid(unsafe_code)]

use std::sync::mpsc::channel;

pub fn main() {
    let (yices_sender, from_yices) = channel();
    let (to_yices, yices_receiver) = channel();
    let _yices = satcache::app::start_process(
        "/usr/local/bin/yices",
        vec!["--mode=push-pop", "--print-success"],
        yices_sender,
        yices_receiver,
    );
    satcache::simple_smt_transaction(&to_yices, &from_yices);
}
