//! cmd

use std::path::PathBuf;

use clap::{ArgAction, Parser, ValueEnum};
use strum::{AsRefStr, FromRepr};

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct GenesisCli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Parser)]
pub enum Commands {
    #[command(name = "run", about = "run cli")]
    Run {
        // #[clap(long, short, value_enum)]
        // mode: ModeEnum,
        #[arg(long, short, value_parser = verify_input_file, default_value = "config.toml", action=ArgAction::Set)]
        config: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum, FromRepr, AsRefStr)]
pub enum ModeEnum {
    Debug,
    Info,
    Warn,
    Error,
}

// 配置文件校验
fn verify_input_file(input: &str) -> anyhow::Result<PathBuf> {
    let pb = PathBuf::from(input);
    if pb.exists() {
        anyhow::Ok(pb)
    } else {
        anyhow::bail!("config file isn not exist")
    }
}
