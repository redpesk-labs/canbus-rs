<img align="right" width="180"  src="docs/asset/tux-iotbzh-canbus.png">
## Introduction

Following libraries/crates interfaces can-socket Linux Kernel capabilities with Rust world.

Current version supports:

* dbc-file parsing and code generator with optional canid white/black list
* raw-can for std+FD frames with optional 'by canid' filters
* bmc-socket with full options (timeout, watchdog, mask, ...)
* can message pool:

    * api to get decoded messages/signals
    * automatic subscription to dbc defined canids
    * signal value cache with status and time stamp
    * native integration with socket-bmc for timeout,watchdog,...

Under development feature (may run until summer-2026)

 * ISOTP/J1939 integration with linux kernel modules
 * Rest/worksocket API through redpesk/AFB bindings
 * integration with Kuksa-databroker

WARNING:

 * canrus-rs should remain under heavy work until end of spring-2026
 * dbc definition quickly generate huge rust parser rust code (use parser white list to reduce size)

Community Support:

* https://matrix.to/#/#redpesk-core:matrix.org
* please keep github only for push request

![community-spport](docs/asset/matrix-redpesk-community.png)

## Architecture

### General architecture
![canbus-rs-archi](docs/asset/canbus-rs-archi.jpg)

### Can Message Pool Apis
![canbus-rs-pool](docs/asset/canbus-rs-pool.jpg)

## Dependencies

* canutils: for can player
* clang: for build.rs

## Compiling

Note:

* Compilation regenerate parser from build.rs selected DBC file.
Depending on selected file this may generate huge rust file. TelsaM3 DBC
generate more than 30000line of rust code. For debug and test it is
recommended to force a canid whitelist within build.rs to limit the
size of generate code.

* Warning: opening a 30K lines cratch vscode, nevertheless vi/gedit still work.

```
git clone https://github.com/redpesk-labs/canbus-rs
cd canbus-rs
touch examples/dbc-log/*.dbc // force dbc parser regeneration
cargo build
```

## Install VCAN

To simulate CAN message injection, you need a vcan device

```bash
echo sudo dnf/zypper install can-utils
sudo modprobe vcan
sudo ip link add dev vcan0 type vcan
sudo ip link set vcan0 up
ip addr | grep "can"  ;# check interface is up
```

## Start a demo

* start virtual can injection
    * apt-get install can-utils;  dnf install can-utils; zypper install can-utils;
    * canplayer vcan0=elmcan -v -I ./examples/model3/etc/candump/model3.log -l i -g 10

* start can-model3

```bash
[canbus-rs]$ ${CARGO_TARGET_DIR:-./target}/debug/can-model3 vcan0 500
(1) => CanID:280 opcode:RxChanged stamp:1681732233413819
  -- DiAccelPedalPos           value:30.400   (f64) status:Updated age:0
  -- DiBrakePedalState         value:0         (u8) status:Unchanged age:0
  -- DiDriveBlocked            value:0         (u8) status:Unchanged age:0
  ...
```

![can-model3](docs/asset/can-model3-demo.png)


Fulup:

- implementer isotp


## improve your Rust code

Use **Clippy** and **rustfmt** to keep the codebase clean, idiomatic, and consistent.

### install

If you use `rustup` (recommended):

```bash
rustup update
rustup component add clippy rustfmt
```



On Fedora without rustup:

sudo dnf install rust-clippy rustfmt

Check versions:

cargo clippy -V
rustfmt -V

### run clippy

cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic

    --all-targets lints libs, bins, tests, benches, examples

    --all-features checks all feature combos

    -D warnings fails the build on any warning

    -W clippy::pedantic enables extra strict lints


cargo clippy -p lib_dbcparser --all-targets --all-features -- -D warnings -W clippy::pedantic
cargo clippy -p lib_sockcan    --all-targets --all-features -- -D warnings -W clippy::pedantic
cargo clippy -p can-basic      --all-targets --all-features -- -D warnings -W clippy::pedantic
cargo clippy -p can-bms        --all-targets --all-features -- -D warnings -W clippy::pedantic
cargo clippy -p can-model3     --all-targets --all-features -- -D warnings -W clippy::pedantic



3) configure clippy

Create a clippy.toml at the repo root:

warn = [
  "clippy::pedantic",
  "clippy::unwrap_used",
  "clippy::expect_used",
  "clippy::panic",
  "clippy::todo",
]

allow = [
  "clippy::module_name_repetitions",
  "clippy::too_many_lines",
  "clippy::missing_errors_doc",
  "clippy::missing_panics_doc",
]

4) format code

Add a rustfmt.toml:

edition = "2021"
use_small_heuristics = "Max"
imports_granularity = "Crate"
group_imports = "One"
format_code_in_doc_comments = true
wrap_comments = true


cargo fmt --all

5) ci integration (GitHub Actions)

name: ci
on: [push, pull_request]
jobs:
  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic
      - run: cargo test --all-features

6) common fixes

Replace unwrap/expect in libraries with ? and typed errors (e.g., thiserror)

Use anyhow in binaries/examples for ergonomic errors

Replace println!/eprintln! with a logger (tracing, log)

Document every unsafe block with a // SAFETY: comment describing invariants

7) troubleshooting

no such command: clippy → rustup component add clippy (or sudo dnf install rust-clippy on Fedora)

noisy generated code → apply #[allow(...)] narrowly around the generated section, not globally

no such command: clippy → rustup component add clippy (or sudo dnf install rust-clippy on Fedora)

