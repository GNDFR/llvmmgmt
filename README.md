llvmmgmt
=========

[![crate](https://img.shields.io/crates/v/llvmmgmt.svg)](https://crates.io/crates/llvmmgmt)

Manage multiple LLVM/Clang build

Install
-------

0. Install cmake, builder (make/ninja), and C++ compiler (g++/clang++)
1. Install Rust using [rustup](https://github.com/rust-lang-nursery/rustup.rs) or any other method.  The minimum supported Rust version is currently **1.48.0**.
2. `cargo install llvmmgmt`

### Basic Usage

To install a specific version of LLVM after following the installation steps above, run these shell commands ("10.0.0" can be replaced with any other version found with `llvmmgmt entries`):

```
llvmmgmt init
llvmmgmt entries
llvmmgmt build-entry 10.0.0
```

zsh integration
-----

You can swtich LLVM/Clang builds automatically using zsh precmd-hook. Please add a line into your `.zshrc`:

```
source <(llvmmgmt zsh)
```

If `$LLVMMGMT_RUST_BINDING` environmental value is non-zero, llvmmgmt exports `LLVM_SYS_60_PREFIX=$(llvmmgmt prefix)` in addition to `$PATH`.

```
export LLVMMGMT_RUST_BINDING=1
source <(llvmmgmt zsh)
```

This is useful for [llvm-sys.rs](https://github.com/tari/llvm-sys.rs) users. Be sure that this env value will not be unset by llvmmgmt, only overwrite.

Concepts
=========

entry
------

- **entry** describes how to compile LLVM/Clang
- Two types of entries
  - *Remote*: Download LLVM from Git/SVN repository or Tar archive, and then build
  - *Local*: Build locally cloned LLVM source
- See [the module document](https://docs.rs/llvmmgmt/*/llvmmgmt/entry/index.html) for detail

build
------

- **build** is a directory where compiled executables (e.g. clang) and libraries are installed.
- They are compiled by `llvmmgmt build-entry`, and placed at `$XDG_DATA_HOME/llvmmgmt` (usually `$HOME/.local/share/llvmmgmt`).
- There is a special build, "system", which uses system's executables.

global/local prefix
--------------------

- `llvmmgmt prefix` returns the path of the current build (e.g. `$XDG_DATA_HOME/llvmmgmt/llvm-dev`, or `/usr` for system build).
- `llvmmgmt global [name]` sets default build, and `llvmmgmt local [name]` sets directory-local build by creating `.llvmmgmt` text file.
- You can confirm which `.llvmmgmt` sets the current prefix by `llvmmgmt prefix -v`.
