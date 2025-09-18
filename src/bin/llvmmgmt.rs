use llvmmgmt::*;

use simplelog::*;
use std::{
    env,
    process::{exit},
};
use structopt::StructOpt;

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
    #[structopt(name = "uninstall", about = "Removes a built version")]
    Uninstall {
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
}

fn get_existing_build(name: &str) -> build::Build {
    let build = build::Build::from_name(name).unwrap();
    if build.exists() {
        build
    } else {
        eprintln!("Build '{}' does not exists", name);
        exit(1)
    }
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
        }
        LLVMMgmt::Uninstall { version } => {
            let build = get_existing_build(&version);
            build.uninstall()?;
        }
        LLVMMgmt::Use { version, global } => {
            let build = get_existing_build(&version);
            if global {
                build.set_global()?;
            } else {
                let path = env::current_dir().unwrap();
                build.set_local(&path)?;
            }
        }
        LLVMMgmt::Current => {
            let build = build::seek_build()?;
            println!("{}", build.name());
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
            } else {
                let builds = build::builds()?;
                for b in &builds {
                    println!("{}", b.name());
                }
            }
        }
        LLVMMgmt::Which => {
            let build = build::seek_build()?;
            println!("{}", build.prefix().join("bin/llvm-config").display());
        }
        LLVMMgmt::Init {} => config::init_config()?,
        LLVMMgmt::Shell { shell } => {
            let shell = shell.or_else(|| env::var("SHELL").ok()).unwrap_or_else(|| "bash".to_string());
            let script = match shell.as_str() {
                "zsh" => ZSH_SCRIPT,
                "bash" => BASH_SCRIPT,
                _ => {
                    eprintln!("Unsupported shell: {}. Supported shells are: bash, zsh", shell);
                    exit(1);
                }
            };
            println!("{}", script);
        }
    }
    Ok(())
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