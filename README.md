# `SAT-cache`: a caching mechanism for SAT and SMT queries.

[![Build, Test, and Publish](https://github.com/weaversa/sat-cache/actions/workflows/main.yml/badge.svg)](https://github.com/weaversa/sat-cache/actions/workflows/main.yml)

# Purpose

This crate provides a mechanism for caching simple SMT queries, with the hope of expanding its capabilites.

# Assurance

This project uses a number of mechanisms for increasing its assurance.

  - `#![forbid(unsafe_code)]` is used to ensure the use of safe Rust,
  - the [`clippy`](https://github.com/rust-lang/rust-clippy) linter is
    used at the pedantic level,
  - the [rust formatter](https://github.com/rust-lang/rustfmt) is used
    to ensure the code adheres to idomatic Rust,
  - the above tools are used by the CI to enforce invariants on this project.

# Local Testing

Presuming Yices is installed in `/usr/local/bin/yices`. Compile this
project's example:

```
$ cargo build --examples
```

Then add the new `cached-smt-solver` binary to your path:

```
$ export PATH="$(pwd)/target/debug/examples/cached-smt-solver:$PATH"
```

Next, create a shell script to act as `yices` --

```yices
#!/bin/sh
export SAT_CACHE_PRINT_SUCCESS=''
export SAT_CACHE_SOLVER=/usr/local/bin/yices
cached-smt-solver $@
```

and add this shell script to your path, ahead of the real `yices`.

```
$ export PATH="<path to yices script>:$PATH"
```

Next, the following commands may be run to test this project:

```
$ time yices --mode=push-pop --print-success < examples/test.smt2
ok
ok
ok
ok
ok
ok
ok
ok
ok
ok
ok
ok
unsat
ok
ok
ok
ok
ok
sat
0b0000000000000000000000000000000000000000000011011001011100000001

real    0m1.083s
user    0m0.004s
sys     0m0.011s
```

Notice the `(check)` line will take longer the first time it's run. Running this example a second time will provide results instantly:

```
$ time yices --mode=push-pop --print-success < examples/test.smt2
ok
ok
ok
ok
ok
ok
ok
ok
ok
ok
ok
ok
unsat
ok
ok
ok
ok
ok
sat
0b0000000000000000000000000000000000000000000011011001011100000001

real    0m0.053s
user    0m0.002s
sys     0m0.008s
```

# Cryptol

The main aim of this project is to speed up repetitive SMT queries by
tools in the Cryptol ecosystem. For example, the first call to `:sat`
or `:prove` takes quite some time:

```
Cryptol> :s prover=w4-yices
Cryptol> :s satNum=all
Cryptol> :sat \(x : [64]) -> x > 1000000 /\ x <= 9999999 /\ (x * x) % 10000000 == x
Satisfiable
(\(x : [64]) -> x > 1000000 /\ x <= 9999999 /\ (x * x) % 10000000 == x)
  0x00000000002c1b81
  = True
(\(x : [64]) -> x > 1000000 /\ x <= 9999999 /\ (x * x) % 10000000 == x)
  0x00000000006c7b00
  = True
Models found: 2
(Total Elapsed Time: 2m:10.427s, using "Yices")
```

Repeat calls return instantly, even between Cryptol sessions
(since the `SQLite` database persists).

```
Cryptol> :sat \(x : [64]) -> x > 1000000 /\ x <= 9999999 /\ (x * x) % 10000000 == x
Satisfiable
(\(x : [64]) -> x > 1000000 /\ x <= 9999999 /\ (x * x) % 10000000 == x)
  0x00000000002c1b81
  = True
(\(x : [64]) -> x > 1000000 /\ x <= 9999999 /\ (x * x) % 10000000 == x)
  0x00000000006c7b00
  = True
Models found: 2
(Total Elapsed Time: 0.030s, using "Yices")
```
