llvmmgmt
=========

[![crate](https://img.shields.io/crates/v/llvmmgmt.svg)](https://crates.io/crates/llvmmgmt)

Manage multiple LLVM/Clang builds.

## Install

0. Install cmake, a builder (make/ninja), and a C++ compiler (g++/clang++).
1. Install Rust using [rustup](https://rustup.rs). The project uses the 2021 edition of Rust, so the minimum supported Rust version is **1.56.0**.
2. `cargo install llvmmgmt`

## Basic Usage

To install a specific version of LLVM, run these shell commands. You can see a list of available versions with `llvmmgmt entries`.

```shell
# Initialize the configuration
llvmmgmt init

# See available LLVM versions
llvmmgmt entries

# Build a specific version (e.g., 10.0.0)
llvmmgmt entry build 10.0.0
```

### Switching between versions

`llvmmgmt` can manage different LLVM versions.

```shell
# See installed versions
llvmmgmt list

# Use a specific version for the current directory
llvmmgmt use 10.0.0

# Use a specific version globally
llvmmgmt use --global 10.0.0

# Show the current active version
llvmmgmt current
```

## Concepts

### entry

- An **entry** describes how to get and compile a specific LLVM/Clang version.
- There are two types of entries:
  - *Remote*: Downloads LLVM from a Git/SVN repository or a Tar archive.
  - *Local*: Uses a locally cloned LLVM source directory.
- You can manage entries with the `llvmmgmt entry` subcommand (e.g., `llvmmgmt entry build <name>`).

### build

- A **build** is a directory where the compiled LLVM/Clang executables and libraries are installed.
- Builds are created by `llvmmgmt entry build` and are located in `$XDG_DATA_HOME/llvmmgmt` (usually `$HOME/.local/share/llvmmgmt`).
- There is a special build named "system" which refers to the system-wide LLVM installation.

### prefix

- `llvmmgmt prefix` returns the path of the current build's installation directory (e.g., `$XDG_DATA_HOME/llvmmgmt/llvm-dev`, or `/usr` for the "system" build).
- You can see which configuration file is setting the current prefix with `llvmmgmt prefix -v`.

## Shell Integration

### zsh

You can automatically switch LLVM/Clang builds when you change directories by using a zsh precmd-hook. Add the following line to your `.zshrc`:

```shell
source <(llvmmgmt shell)
```

If the `$LLVMMGMT_RUST_BINDING` environment variable is set to a non-zero value, `llvmmgmt` will also export `LLVM_SYS_..._PREFIX` for `llvm-sys`.

```shell
export LLVMMGMT_RUST_BINDING=1
source <(llvmmgmt shell)
```
