# `Passmgr` Roadmap

## Progress Tracking

- [x] Stage 1 – Unlock & quit *(starter code provided)*
    - [x] Accept master password argument
    - [x] Exit with error if empty
    - [x] Print "Unlocked (stub)" if valid

- [x] Stage 2 – Interactive REPL with in-memory store
    - [x] Implement interactive prompt
    - [x] Add command: `add <n> <secret>`
    - [x] Add command: `get <n>`
    - [x] Add command: `list`
    - [x] Add command: `quit` / `exit`
    - [x] Set up in-memory HashMap store

- [ ] Stage 3 – Local persistence (unencrypted JSON)
    - [ ] Use dirs-next to locate home directory
    - [ ] Implement JSON serialization/deserialization
    - [ ] Save to disk after mutations
    - [ ] Load from disk on startup
    - [ ] Handle missing/invalid files

- [ ] Stage 4 – Encryption at rest
    - [ ] Implement Argon2id key derivation with salt
    - [ ] Implement AES-GCM 256 encryption
    - [ ] Format file as salt | nonce | ciphertext
    - [ ] Ensure no plaintext is stored on disk

## Stage 1 – Unlock & quit *(starter code provided)*

**Problem**

Write a binary `passmgr` that:

1. Accepts one required positional argument **`<master>`**.
2. Exits non-zero if the argument is empty; otherwise prints `Unlocked (stub).` and exits 0.

**Implementation notes**

* Use **clap 4**’s `derive` API for argument parsing.
* Return `std::process::ExitCode` for clarity.
* This stage’s code is in the canvas – nothing to implement.

**Tests**

```
cargo run                  → exit≠0, stderr ~ /USAGE/i
cargo run ""               → exit≠0, stderr ~ /empty/
cargo run hunter2          → exit 0,  stdout == “Unlocked (stub).”
```

Passing all three unlocks Stage 2.

---

## Stage 2 – Interactive REPL with in-memory store

**Problem**

After a successful unlock, drop into an endless prompt `passmgr> ` that understands:

| Command               | Effect                                      |
|-----------------------|---------------------------------------------|
| `add <name> <secret>` | Insert pair.  Failure if `<name>` exists.   |
| `get <name>`          | Print secret; exit ≠ 0 if missing.          |
| `list`                | Print one name per line (order not graded). |
| `quit` / `exit`       | Terminate process, returning 0.             |

**Implementation notes**

* `std::io::{stdin,stdout}` is fine; no extra crate required yet.
* Store credentials in a `HashMap<String,String>`.
* Keep the master password in scope only as long as needed, then overwrite (`String::clear()`).

**Tests**

* Feed `add foo bar\nget foo\nquit\n` via stdin – expect `bar` in stdout and exit 0.
* `list` after two `add` commands must contain both names.
* `get missing` must exit ≠ 0 and print “not found”.

---

## Stage 3 – Local persistence (unencrypted JSON)

**Problem**

Persist the map to disk after every mutating command and reload on start-up.

* File path: `~/.passmgr/db.json` (use `dirs-next` to find home).
* File format: UTF-8 JSON `{"name":"secret", ...}`.

**Implementation notes**

* Add **serde + serde\_json**.
* Tolerate a missing file by starting with an empty map.
* If the file exists but is invalid JSON, exit ≠ 0 with a clear error.

**Tests**

1. Remove the DB, run `add a 1`, quit. Second run with `get a` must print `1`.
2. Pre-write `db.json` with `not json`, expect non-zero exit code and error on start.

---

## Stage 4 – Encryption at rest

**Problem**

The JSON blob on disk must be encrypted with a key derived from the master password.

* Key derivation: **Argon2id** with a 16-byte random salt.
* Symmetric cipher: **AES-GCM 256**
* File layout (binary): `salt | nonce | ciphertext`.

**Implementation notes**

* Add crates: `argon2`, `aes-gcm`, `rand`, `base64` (for debugging prints).
* Store should be opaque – no plaintext credentials may occur verbatim in the file.
* Keep salt fixed after first run; regenerate only if the file is recreated.

**Tests**

* After `add secret`, raw file must not contain the bytes of `secret`.
* With a fixed test salt/nonce/key, encryption then decryption must round-trip.

---

## Stage 5 – Strict unlock / wrong-password handling

**Problem**

`passmgr` must attempt to decrypt immediately; if the master password is wrong, exit ≠ 0 *before* prompting.

**Implementation notes**

* Detect AES-GCM authentication failure and translate to “Invalid password”.
* Argon2 is intentionally slow – cache its parameters in the file header so you can recreate them.

**Tests**

* Create a store with `hunter2`. Running with `wrongpass` must exit ≠ 0.
* Running again with `hunter2` continues to work and shows existing entries.

---

## Stage 6 – Edit & peek commands

**Problem**

Add two new commands:

* `edit <name> <new_secret>` – overwrite secret (fail if missing).
* `peek <name>` – print secret **without** a trailing newline (handy for piping to clipboard tools).

**Implementation notes**

* Remember to `save()` after `edit`.
* For Windows users, suggest the `arboard` crate to copy to clipboard, but printing is enough for the grader.

**Tests**

* `add foo bar` → `peek foo` prints `bar` (no newline).
* `edit foo baz` → `peek foo` now prints `baz`.

---

## Stage 7 – Zeroisation & secure cleanup

**Problem**

Ensure no plaintext secrets or keys remain in memory once they’re no longer needed.

**Implementation notes**

* Add **zeroize**. Wrap key and decrypted JSON in `Zeroizing<T>`.
* Call `zeroize()` on `String`s you allocate temporarily (master password too).

**Tests**

* The grader links the `zeroize` test-helper feature and checks that dropped buffers have been wiped.
* Failure to zeroise causes a test panic.

---

## Stage 8 – Polish (optional, non-blocking)

* Command aliases (`ls`, `set`, etc.).
* `passmgr --help` shows colourful clap usage.
* Cross-platform clipboard integration.
* Integration tests using `assert_cmd` + `predicates`.

---

### Crate “shopping list” by stage

| Stage | Likely new crates                                |
|-------|--------------------------------------------------|
| 1     | `clap`, `anyhow`                                 |
| 2     | *(none)*                                         |
| 3     | `serde`, `serde_json`, `dirs-next`               |
| 4     | `argonautica` **or** `argon2`, `aes-gcm`, `rand` |
| 5     | already covered                                  |
| 6     | `arboard` (optional)                             |
| 7     | `zeroize`                                        |

---

### How the grader runs your code

* For argument-based stages it executes `cargo run --quiet -- <master> <args…>`.
* For REPL stages it streams commands on **stdin** terminated with `\\n`.
* It examines **stdout**, **stderr** and the program’s **exit code** exactly as stated in each stage.
* Your binary must never prompt for input during automated tests except in Stage 2+ where stdin is provided.
