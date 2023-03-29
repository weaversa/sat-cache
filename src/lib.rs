//! # `sat-cache`
//!
//! `sat-cache` is an interface for caching SAT/SMT queries.

#![forbid(unsafe_code)]

pub mod app;

use base64::{engine::general_purpose, Engine as _};
use rusqlite::Connection;
use sha3::{Digest, Sha3_384};
use std::io::{BufRead, Write};
use std::sync::mpsc::{Receiver, Sender};

/// This function creates a connection to an `SQLite` database.
///
/// # Panics
///
/// This function may panic if something goes wrong with opening the
/// given database.
#[must_use]
pub fn db_connect(db_file: &str) -> Connection {
    // Create database connection. Panic on failure.
    let db = Connection::open(db_file).unwrap();

    // Create the main table if it doesn't already exist. Panic on failure.
    db.execute(
        "CREATE TABLE IF NOT EXISTS data (hash TEXT NOT NULL PRIMARY KEY, result TEXT NOT NULL)",
        [],
    )
    .unwrap();

    db
}

/// This function inserts a `(hash, result)` into the `SQLite` database.
fn insert_result(db: &Connection, hash: &str, result: &str) {
    // Insert hash:result into the database.
    db.execute(
        "INSERT OR IGNORE INTO data (hash, result) VALUES (?1, ?2)",
        [hash, result],
    )
    .unwrap();
}

/// This function queries the `SQLite` database for `result` at the
/// given `hash`.
pub fn get_result(db: &Connection, hash: &str) -> String {
    // Retrieve from the database, the result based on a given
    // hash. Returns the empty string if no such hash exists.
    match db.query_row("SELECT result FROM data where hash = ?1", [hash], |row| {
        row.get(0)
    }) {
        Ok(r) => r,
        Err(_) => String::new(),
    }
}

/// This function will return an encoding the current SHA3-384 state.
fn get_hash(hasher: &Sha3_384) -> String {
    // Read hash digest
    let hash = hasher.clone().finalize();

    // Encode the digest
    general_purpose::STANDARD_NO_PAD.encode(hash)
}

/// Helper function to get the next whole line from `stdin`.
fn get_next_line_from_stdin() -> String {
    let stdin = std::io::stdin();
    match stdin.lock().lines().next() {
        Some(Ok(r)) => r,
        Some(Err(_)) | None => String::new(),
    }
}

/// Helper function to send a whole line to `stdout`.
fn send_line_to_stdout(line: &str) {
    let stdout = std::io::stdout();
    stdout.lock().write_all(line.as_bytes()).unwrap();
}

/// This is function acts as middleware between an application and an
/// SMT solver. It will pass transactions between the two, caching any
/// long-running commands such as `(check)` and `(check-sat)`, and
/// their dependencies (such as retrieving satisfying assignments).
///
/// # Panics
///
/// This function may panic if the request to the SMT solver has some
/// issue such as more (pop) than (push) commands.
pub fn simple_smt_transaction(to_app: &Sender<String>, from_app: &Receiver<String>) {
    // Connect to the database.
    let db = db_connect("satcache.db");

    // Create a SHA3-384 object.
    let mut hasher = Sha3_384::new();

    // Create stack of hashes to maintain push/pop commands.
    let mut stack: Vec<Sha3_384> = Vec::new();
    stack.push(hasher.clone());

    let mut exit = false;

    // The main loop.
    loop {
        if exit {
            break;
        }
        let line = get_next_line_from_stdin();
        if line.is_empty() {
            break;
        }

        if line.eq("(exit)") {
            exit = true;
        }

        // Skip SMT-LIB2 comments.
        if line.starts_with(';') {
            continue;
        }

        if line.starts_with("(push") {
            // This is a little presumptive as `push` can take an argument.
            stack.push(hasher.clone());
        } else if line.starts_with("(pop") {
            // This is also a little presumptive as `pop` can take an argument.
            hasher = stack.pop().unwrap();
            assert!(
                !stack.is_empty(),
                "(pop) command without a corresponding (push)"
            );
        } else {
            // Add the line into the hash state.
            hasher.update(&line);
        }

        // Proccess line
        if line.starts_with("(check")
            || line.starts_with("(eval ")
            || line.starts_with("(get-value ")
        {
            // Check to see if this session is cached.
            let mut response = get_result(&db, &get_hash(&hasher));
            if response.is_empty() {
                // Session is not cached.
                // Pass the request along and cache the result.
                to_app.send(line).unwrap();
                // Read the response. Break the loop on error.
                response = match from_app.recv() {
                    Ok(r) => r,
                    Err(_) => break,
                };
                // Cache session result.
                insert_result(&db, &get_hash(&hasher), &response);
            }

            // Send the response back to the calling application.
            send_line_to_stdout(&response);
        } else {
            // This kind of line will not be cached. Send it along to
            // the SMT solver.
            to_app.send(line).unwrap();
            // Read the response. Break the loop on error.
            let Ok(response) = from_app.recv() else { break };
            // Send the response back to the calling application.
            send_line_to_stdout(&response);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // create a SHA3-384 object
        let mut hasher = Sha3_384::new();
        let db = db_connect("satcache.db");
        hasher.update("hellofish");
        insert_result(&db, &get_hash(&hasher), "file is sat");
        let s = get_result(&db, &get_hash(&hasher));
        assert_eq!(s, "file is sat");
        hasher.reset();
        hasher.update("hellofish1");
        let s2 = get_result(&db, &get_hash(&hasher));
        assert!(s2.is_empty());
    }
}
