use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use clap::Parser;

use deck_optim::card_named;
use deck_optim::strategies::StrategyImpl;
use deck_optim::trial::Trial;
use deck_optim::watcher::{MetricsData, WatcherImpl};
use itertools::Itertools;
use rand::SeedableRng;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;

use deck_optim::game::card::CardCollection;

type Result<T, E=Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[arg(short='c', long)]
    /// Supply an optional parameter to read the input from a file
    pub card_collection: PathBuf,
    #[arg(short='t', long)]
    pub num_trials: Option<u32>
}

pub fn configure_logging() {
    let stdout_log = tracing_subscriber::fmt::layer()
            .compact()
            .with_level(true)
            .with_thread_names(true)
            .with_file(true)
            .with_filter(LevelFilter::WARN);

    tracing_subscriber::registry()
        .with(stdout_log)
        .init();
   
}

fn parse_card_collection(path: &Path) -> Result<CardCollection> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let card_data = serde_json::from_reader(reader)?;

    Ok(card_data)
}

const DEFAULT_NUM_TRIALS: u32 = 10000; 

fn report_metrics_data(_cli: &Cli, metrics: &MetricsData) -> Result<()> {
    use prettytable::{row, Table};

    let mut table = Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);

    table.set_titles(row!["Metrics Name", "Average"]);
    for key in metrics.keys().sorted() {

        let avg = metrics.average(key);
        table.add_row(row![key, avg]);

    }

    table.printstd();

    Ok(())
}

fn run(cli: Cli) -> Result<()> {
    let cards = parse_card_collection(&cli.card_collection)?;
    log::info!("parsed {} cards", cards.num_cards());
    deck_optim::init(cards);


    let num_trials = cli.num_trials.unwrap_or(DEFAULT_NUM_TRIALS);

    let mut deck = deck_optim::game::UnorderedPile::empty();
    deck.add(card_named("Lightning Bolt"), 23);
    deck.add(card_named("Mountain"), 17);

    let watcher = WatcherImpl;
    let strategies = StrategyImpl {
        rng: rand::rngs::StdRng::from_entropy()
    };

    let metrics = (0..num_trials)
        .into_iter()
        .into_par_iter()
        .map(|_| {
            let rng = rand::rngs::StdRng::from_entropy();
            let t = Trial::new(
                deck.clone(),
                rng,
            );
            t.run(&mut strategies.clone(), &watcher)
        })
        .reduce(|| MetricsData::empty(), MetricsData::join);


    report_metrics_data(&cli, &metrics)?;

    Ok(())
}

fn main() {
    let cli = Cli::parse();
    configure_logging();

    let res = run(cli);
    if let Err(e) = res {
        println!("{}", e);
    }
}
