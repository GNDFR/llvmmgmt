use llvmmgmt::*;
use llvmmgmt::error::{Result, Error, FileIoConvert};

use simplelog::*;
use std::{
    env,
    process::{exit},
    fs,
    collections::HashMap,
    path::PathBuf,
};
use structopt::StructOpt;

use crate::build::{Build, seek_build};
use crate::config::{config_dir, ENTRY_TOML};

fn get_existing_build(name: &str) -> Result<build::Build> {
    let build = build::Build::from_name(name)?;
    if build.exists() {
        Ok(build)
    } else {
        eprintln!("Build '{name}' does not exists");
        Err(Error::InvalidBuild { name: name.into(), message: "Build does not exist".into() })
    }
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "llvmmgmt",
    about = "Manage multiple LLVM/Clang builds",
    setting = structopt::clap::AppSettings::ColoredHelp
)]
enum LLVMMgmt {
    #[structopt(name = "install", about = "Downloads and builds a specific LLVM version")]
    Install {
        version: String,
    },

    #[structopt(name = "use", about = "Sets the current LLVM version")]
    Use {
        version: String,
        #[structopt(long)]
        global: bool,
    },
    #[structopt(name = "current", about = "Shows the currently active LLVM version")]
    Current,
    #[structopt(name = "list", about = "Lists all available or installed versions")]
    List {
        #[structopt(long)]
        available: bool,
    },
    #[structopt(name = "which", about = "Shows the path to the current llvm executable")]
    Which,
    #[structopt(name = "init", about = "Initializes the tool")]
    Init,
    #[structopt(name = "shell", about = "Show shell setup script")]
    Shell {
        #[structopt(long, short)]
        shell: Option<String>,
    },
    #[structopt(name = "entries", about = "List all available entries")]
    Entries,
    #[structopt(name = "build-entry", about = "Build a specific entry")]
    BuildEntry {
        name: String,
        #[structopt(long, default_value = "0")]
        nproc: usize,
    },
    #[structopt(name = "clean-cache", about = "Clean cache directory for an entry")]
    CleanCache {
        name: String,
    },
    #[structopt(name = "clean-build", about = "Clean build directory for an entry")]
    CleanBuild {
        name: String,
    },
    #[structopt(name = "checkout", about = "Checkout source code for an entry")]
    Checkout {
        name: String,
    },
    #[structopt(name = "update", about = "Update source code for an entry")]
    Update {
        name: String,
    },
    #[structopt(name = "set-global", about = "Set global default build")]
    SetGlobal {
        name: String,
    },
    #[structopt(name = "set-local", about = "Set local default build")]
    SetLocal {
        name: String,
    },
    #[structopt(name = "prefix", about = "Show current build prefix")]
    Prefix {
        #[structopt(long, short)]
        verbose: bool,
    },
    #[structopt(name = "archive", about = "Archive a build")]
    Archive {
        name: String,
        #[structopt(long, short)]
        verbose: bool,
    },
    #[structopt(name = "expand", about = "Expand an archived build")]
    Expand {
        archive: PathBuf,
        #[structopt(long, short)]
        verbose: bool,
    },
    #[structopt(name = "uninstall", about = "Removes a built version")]
    Uninstall {
        name: String,
    },
    #[structopt(name = "set-build-type", about = "Set build type for an entry")]
    SetBuildType {
        name: String,
        build_type: llvmmgmt::entry::BuildType,
    },
    #[structopt(name = "set-generator", about = "Set CMake generator for an entry")]
    SetGenerator {
        name: String,
        generator: String,
    },
}

fn main() -> error::Result<()> {
    TermLogger::init(
        LevelFilter::Info,
        ConfigBuilder::new().set_time_to_local(true).build(),
        TerminalMode::Mixed,
    )
    .or(SimpleLogger::init(
        LevelFilter::Info,
        ConfigBuilder::new().set_time_to_local(true).build(),
    ))
    .unwrap();

    let opt = LLVMMgmt::from_args();
    match opt {
        LLVMMgmt::Install { version } => {
            let entry = entry::load_entry(&version)?;
            let nproc = num_cpus::get();
            entry.checkout().unwrap();
            entry.build(nproc).unwrap();
            Ok(())
        }

        LLVMMgmt::Use { version, global } => {
            let build = get_existing_build(&version)?;
            if global {
                build.set_global()?;
            } else {
                let path = env::current_dir()?;
                build.set_local(&path)?;
            }
            Ok(())
        }
        LLVMMgmt::Current => {
            let build = build::seek_build()?;
            println!("{}", build.name());
            Ok(())
        }
        LLVMMgmt::List { available } => {
            if available {
                if let Ok(entries) = entry::load_entries() {
                    for entry in &entries {
                        println!("{}", entry.name());
                    }
                } else {
                    panic!("No entries. Please define entries in $XDG_CONFIG_HOME/llvmmgmt/entry.toml");
                }
                Ok(())
            } else {
                let builds = build::builds()?;
                for b in &builds {
                    println!("{}", b.name());
                }
                Ok(())
            }
        }
        LLVMMgmt::Which => {
            let build = build::seek_build()?;
            println!("{}", build.prefix().join("bin/llvm-config").display());
            Ok(())
        }
        LLVMMgmt::Init => config::init_config(),
        LLVMMgmt::Shell { shell } => {
            let shell = shell.or_else(|| env::var("SHELL").ok()).unwrap_or_else(|| "bash".to_string());
            let script = match shell.as_str() {
                "zsh" => ZSH_SCRIPT,
                "bash" => BASH_SCRIPT,
                _ => {
                    eprintln!("Unsupported shell: {shell}. Supported shells are: bash, zsh");
                    return Err(Error::UnsupportedShell { shell });
                }
            };
            println!("{script}");
            Ok(())
        }
        LLVMMgmt::Entries => {
            for entry in entry::load_entries()? {
                println!("{}", entry.name());
            }
            Ok(())
        }
        LLVMMgmt::BuildEntry { name, nproc } => {
            let entry = entry::load_entry(&name)?;
            entry.build(nproc)?;
            Ok(())
        }
        LLVMMgmt::CleanCache { name } => {
            let entry = entry::load_entry(&name)?;
            entry.clean_cache_dir()?;
            Ok(())
        }
        LLVMMgmt::CleanBuild { name } => {
            let entry = entry::load_entry(&name)?;
            entry.clean_build_dir()?;
            Ok(())
        }
        LLVMMgmt::Checkout { name } => {
            let entry = entry::load_entry(&name)?;
            entry.checkout()?;
            Ok(())
        }
        LLVMMgmt::Update { name } => {
            let entry = entry::load_entry(&name)?;
            entry.update()?;
            Ok(())
        }
        LLVMMgmt::SetGlobal { name } => {
            let build = Build::from_name(&name)?;
            build.set_global()?;
            Ok(())
        }
        LLVMMgmt::SetLocal { name } => {
            let build = Build::from_name(&name)?;
            build.set_local(&env::current_dir()?)?;
            Ok(())
        }
        LLVMMgmt::Prefix { verbose } => {
            let build = seek_build()?;
            if verbose {
                if let Some(path) = build.env_path() {
                    eprintln!("Current build is set by {}", path.display());
                }
            }
            println!("{}", build.prefix().display());
            Ok(())
        }
        LLVMMgmt::Archive { name, verbose } => {
            let build = Build::from_name(&name)?;
            build.archive(verbose)?;
            Ok(())
        }
        LLVMMgmt::Expand { archive, verbose } => {
            build::expand(&archive, verbose)?;
            Ok(())
        }
        LLVMMgmt::Uninstall { name } => {
            let build = Build::from_name(&name)?;
            build.uninstall()?;
            Ok(())
        }
        LLVMMgmt::SetBuildType { name, build_type } => {
            let mut entry = entry::load_entry(&name)?;
            entry.set_build_type(build_type)?;
            let global_toml = config_dir()?.join(ENTRY_TOML);
            let mut entries = entry::load_entry_toml(
                &fs::read_to_string(&global_toml).with(&global_toml)?,
            )?;
            let mut found = false;
            for e in entries.iter_mut() {
                if e.name() == entry.name() {
                    *e = entry;
                    found = true;
                    break;
                }
            }
            if !found {
                eprintln!("Entry '{}' not found in {}", name, global_toml.display());
                exit(1);
            }
            let mut map = HashMap::new();
            for e in entries {
                map.insert(e.name().to_string(), e.setting().clone());
            }
            let toml_str = toml::to_string(&map)?;
            fs::write(&global_toml, toml_str).with(&global_toml)?;
            Ok(())
        }
        LLVMMgmt::SetGenerator { name, generator } => {
            let mut entry = entry::load_entry(&name)?;
            entry.set_builder(&generator)?;
            let global_toml = config_dir()?.join(ENTRY_TOML);
            let mut entries = entry::load_entry_toml(
                &fs::read_to_string(&global_toml).with(&global_toml)?,
            )?;
            let mut found = false;
            for e in entries.iter_mut() {
                if e.name() == entry.name() {
                    *e = entry;
                    found = true;
                    break;
                }
            }
            if !found {
                eprintln!("Entry '{}' not found in {}", name, global_toml.display());
                exit(1);
            }
            let mut map = HashMap::new();
            for e in entries {
                map.insert(e.name().to_string(), e.setting().clone());
            }
            let toml_str = toml::to_string(&map)?;
            fs::write(&global_toml, toml_str).with(&global_toml)?;
            Ok(())
        }
    }
}

const BASH_SCRIPT: &str = r#"
function llvmmgmt_update() {
    # ... (bash-specific implementation)
}

export PROMPT_COMMAND=llvmmgmt_update
"#;

const ZSH_SCRIPT: &str = r#"
function llvmmgmt_remove_path() {
  path_base=${XDG_DATA_HOME:-$HOME/.local/share/llvmmgmt}
  path=("${(@)path:#$path_base/*}")
}

function llvmmgmt_append_path() {
  prefix=$(llvmmgmt prefix)
  if [[ -n "$prefix" && "$prefix" != "/usr" ]]; then
    # To avoid /usr/bin and /bin become the top of $PATH
    path=($prefix/bin(N-/) $path)
  fi
}

function llvmmgmt_env_llvm_sys () {
  export LLVM_SYS_$(llvmmgmt version --major --minor)_PREFIX=$(llvmmgmt prefix)
}

function llvmmgmt_update () {
  llvmmgmt_remove_path
  llvmmgmt_append_path
  if [[ -n "$LLVMMGMT_RUST_BINDING" ]]; then
    llvmmgmt_env_llvm_sys
  fi
}

autoload -Uz add-zsh-hook
add-zsh-hook precmd llvmmgmt_update
"#;
