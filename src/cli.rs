use std::{io, path::PathBuf};

use clap::{Parser, Subcommand};

use crate::{ModelPars, run_model};

/// NIHR care ABM model
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run model
    Run {
        /// Parameter file
        pars: Option<PathBuf>,
    },
    /// Create configuration file
    Config {
        /// Parameter file to create
        pars: PathBuf,
    },
}

pub fn main() -> io::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run { pars } => {
            let pars = match pars {
                Some(path) => ModelPars::read_from(path)?,
                None => ModelPars::default(),
            };
            run_model::main(pars)
        }
        Commands::Config { pars } => ModelPars::default().save_to(pars),
    }
}
