use std::path::PathBuf;

use clap::Parser;
use tracing_subscriber::filter::LevelFilter;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[arg(short='c', long)]
    /// Supply an optional parameter to read the input from a file
    pub card_collection: PathBuf,
    #[arg(short='t', long)]
    pub num_trials: Option<u32>,

    #[arg(long)]
    /// Supply this parameter to change the default level filters
    pub level_filter: Option<LevelFilter>,


    #[arg(short='d', long)]
    pub deck_list: Option<PathBuf>,
}

