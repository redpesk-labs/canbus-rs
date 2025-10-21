
# Contribute

## improve your Rust code

Use **Clippy** and **rustfmt** to keep the codebase clean, idiomatic, and consistent.

why clippy and rustfmt are mandatory in an industrial rust project?

A professional codebase needs predictability, safety, and speed of iteration. Two tools make that non-negotiable baseline real:

rustfmt enforces a single canonical style.

Clippy enforces a baseline of code quality by flagging common mistakes and non-idiomatic patterns.

Together they reduce defects, shrink review time, and make the code easier to maintain over years and teams.

### rust fmt

rustfmt (formatting you can rely on)

What it guarantees?

Zero bikeshedding: one style, produced automatically. Reviews focus on logic, not whitespace.

Stable diffs: predictable formatting reduces noisy diffs and makes git blame more meaningful.

Onboarding made easy: new contributors don’t need to learn a house style.

Tooling interoperability: editors, CI, and pre-commit hooks can all run the same formatter.

Organization policy (recommended)

Formatting is required; PRs must pass cargo fmt --all --check.

A project-local rustfmt.toml defines only needed deviations (often: none).

If you use `rustup` (recommended):

```bash
rustup update
rustup component add rustfmt
```

### run fmt

```bash
cargo fmt --all
```

### rust clippy

clippy (linting that prevents subtle bugs)

What it guarantees?

Bug prevention: catches suspicious code (unwrap() in tests ok, in prod not ok; needless clones; wrong iterator bounds; Mutex in async contexts; etc.).

Idiomatic rust: nudges toward patterns the ecosystem expects, improving readability and performance.

Security hygiene: warns about panic! in FFI, mem::uninitialized, or non-Send/Sync in shared contexts, among many others.

Organization policy (recommended)

Clippy runs on every commit and in CI with warnings elevated to errors

If you use `rustup` (recommended):

```bash
rustup update
rustup component add clippy
```

### run clippy

```bash
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic

    --all-targets lints libs, bins, tests, benches, examples

    --all-features checks all feature combos

    -D warnings fails the build on any warning

    -W clippy::pedantic enables extra strict lints
```

### quality gates (must pass before merge)

1. **Formatting:** `cargo fmt --all --check`
2. **Linting:** `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. **Tests:** `cargo test --workspace --all-features`
4. **Security:** `cargo audit` (no unpatched advisories) and `cargo deny check` (licenses/deps policy)
5. **Docs:** `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` (no doc warnings)
