# passmgr

A secure command-line password manager written in `Rust`.

## Features

- Securely store and manage your credentials
- Strong encryption using `ChaCha20-Poly1305`
- Password derivation using `Argon2id`
- Simple command-line interface
- Local storage with no external dependencies

## Installation

### Prerequisites

- Rust and Cargo (install from [rustup.rs](https://rustup.rs))

### Building from source

```bash
# Clone the repository
git clone https://github.com/AlexShem/passmgr.git
cd passmgr

# Build the project
cargo build --release

# The executable will be in target/release/passmgr
```

## Usage

### First Time Setup

The first time you run `passmgr`, it will create a new password database in your home directory (
`~/.passmgr/passwords.db`
on Unix-like systems, or similar to `C:\Users\username\.passmgr\passwords.db` location on Windows).

```bash
# Run the program
./target/release/passmgr
```

You'll be prompted to create a master password. This password is used to encrypt all your credentials, so make sure it's
strong, and you don't forget it!

### Managing Credentials

Once you've set up your master password, you can use the following commands:

- `add`: Add a new credential
  ```
  passmgr> add --name "example-account" --secret "your-password-here"
  ```

- `get`: Retrieve a credential
  ```
  passmgr> get "example-account"
  ```

- `remove` (or `rm`): Delete a credential
  ```
  passmgr> remove "example-account"
  ```

- `list`: Show all stored credential names
  ```
  passmgr> list
  ```

- `quit` (or `exit`): Exit the program
  ```
  passmgr> quit
  ```

- For help with commands
  ```
  passmgr> --help
  ```

## Security

- Your credentials are encrypted using `ChaCha20-Poly1305`, a high-performance authenticated encryption algorithm.
- Password derivation is handled by `Argon2id`, designed to be resistant to both brute force and side-channel attacks.
- The master password is never stored; it's only used to derive encryption keys.
- If you forget your master password, there is no recovery mechanism by design.
