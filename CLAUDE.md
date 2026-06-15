# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

`passmgr` is a secure command-line password manager in Rust (edition 2024). It stores credentials in a single encrypted JSON file and exposes a custom REPL (built on `rustyline`) with completion, hints, and highlighting. It is both a binary (`src/main.rs`) and a library (`src/lib.rs`), so internal modules are exercised directly from integration tests in `tests/`.

## Commands

```bash
cargo build                  # debug build
cargo build --release        # release binary -> target/release/passmgr
cargo run                    # run the interactive shell

cargo test                   # all unit + integration tests
cargo test --test integration_tests           # integration tests only
cargo test test_save_and_load_credentials      # a single test by name (substring match)
cargo test --lib             # unit tests inside src/ only

cargo fmt --all              # format (CI runs `cargo fmt --all -- --check`)
cargo clippy -- -D warnings  # lint; CI fails on any warning
```

CI (`.github/workflows/ci.yml`) runs build, test, fmt-check, and clippy on Linux/Windows/macOS, then builds release binaries. Match those gates locally before pushing: clippy warnings and fmt drift both break CI.

## Runtime data

All state lives under `~/.passmgr/` (see `src/config.rs`):
- `passwords.db` — encrypted credential store (`EncryptedStore` serialized as pretty JSON)
- `history` — REPL command history (capped at `DEFAULT_HISTORY_SIZE` = 1000)
- `passmgr.log` — application log (size-rotated, Info level by default)

## Architecture

The flow is `main.rs` → `Manager` → `Shell` → `Command`s, with crypto/storage as leaf concerns.

**`Manager` (`src/manager.rs`)** owns the lifecycle: detects new vs. existing user, derives keys, and bridges to the shell. `main.rs` handles master-password prompting (via `rpassword`) and calls `setup_new_user` / `validate_master_password`. Note: encryption uses a **fresh random salt and nonce on every save** — there is no separately stored password verifier, so "validating" the master password means attempting a full decrypt (`load_credentials_with_password`). A wrong password surfaces as a decrypt failure, not a distinct error.

**Crypto/storage split:**
- `src/crypto.rs` — Argon2id key derivation (`derive_key`, 32-byte key from password + 16-byte salt) and ChaCha20-Poly1305 AEAD (`encrypt`/`decrypt` with a 12-byte nonce). Salt/nonce come from `OsRng`.
- `src/storage.rs` — `EncryptedStore` struct (versioned) plus base64 encode/decode helpers and file read/write. This layer only moves bytes; it knows nothing about passwords.

**Shell (`src/shell/`)** is a from-scratch command system (it replaced clap for the REPL). Key pieces:
- `command.rs` — the `Command` trait, `CommandResult` enum (`Success`/`Error`/`Exit`/`Continue`), `ShellContext` (carries `&mut Credentials`, a `modified` flag, and the `key_trie`), and `CommandRegistry` (name + alias lookup, trie-backed completion).
- `commands/` — one file per command (`add`, `get`, `remove`, `list`, `help`, `quit`), all registered in `commands/mod.rs::register_all`. **To add a command:** implement `Command`, then register it there.
- `mod.rs` — `PassmgrHelper` wires rustyline's `Completer`/`Highlighter`/`Hinter`/`Validator` traits to the `completer`/`highlighter`/`hints` modules. `run_with_save` is the REPL loop: it parses a line, runs the command against a `ShellContext`, and **persists via a save callback only when `ctx.modified` is set**. Commands signal a save by calling `ctx.mark_modified()`.

**`Trie` (`src/trie.rs`)** powers prefix completion for both command names (owned by the registry) and credential keys (shared as `Arc<RwLock<Trie>>` between the shell and helper). Commands that mutate credentials must also keep the key trie in sync (e.g. `add` calls `ctx.key_trie.insert`).

**`Credentials` (`src/credentials.rs`)** is a thin `HashMap<String, String>` wrapper (name → secret). Serialization to/from the store goes through `to_map`/`from_map`.

## Conventions

- Errors use `anyhow::Result` throughout the binary/crypto/storage layers; `Credentials` methods return `Result<_, String>` for user-facing messages.
- Logging via the `log` macros (`log::info!`, etc.), backed by `simplelog`. The REPL also logs per-command timing at debug level.
- A secret may contain spaces: commands like `add` join `args[1..]` rather than taking a single token.
