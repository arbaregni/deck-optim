use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use clap::Parser;

use deck_optim::card_named;
use deck_optim::strategies::StrategyImpl;
use deck_optim::trial::run_trials;
use itertools::Itertools;
use prettytable::{row, Table};
use rand::SeedableRng;

use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;

use deck_optim::game::card::CardCollection;
use deck_optim::metrics::MetricsData;
use deck_optim::watcher::WatcherImpl;


type Result<T, E=Box<dyn std::error::Error>> = std::result::Result<T, E>;

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
    pub level_filter: Option<LevelFilter>
}

pub fn configure_logging(cli: &Cli) {
    let level_filter = cli.level_filter.unwrap_or(LevelFilter::INFO);

    let stdout_log = tracing_subscriber::fmt::layer()
            .compact()
            .with_level(true)
            .with_thread_names(true)
            .with_file(true)
            .with_filter(level_filter);

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

fn make_table() -> Table {
    let mut table = Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);
    table
}

fn report_metrics_data(_cli: &Cli, metrics: &MetricsData) -> Result<()> {
    use prettytable::{row, Table};

    let mut table = make_table();

    table.set_titles(row!["Metrics Name", "Average"]);
    for key in metrics.keys().sorted() {

        let avg = metrics.average(key);
        table.add_row(row![key, avg]);

    }

    table.printstd();

    Ok(())
}


const TOTAL_DECK_SIZE: usize = 40;

fn evaluate_hyper_param(cli: &Cli, num_trials: u32, param: usize) -> MetricsData {

    let mut deck = deck_optim::game::UnorderedPile::empty();
    deck.add_copies(card_named("Lightning Bolt"), param);
    deck.add_copies(card_named("Mountain"),  TOTAL_DECK_SIZE - param);

    let watcher = WatcherImpl;
    let strategies = StrategyImpl {
        rng: rand::rngs::StdRng::from_entropy()
    };

    println!("============= # of lightning bolts = {param} ==================");

    let metrics = run_trials(num_trials, deck, strategies, watcher);
    
    report_metrics_data(&cli, &metrics)
        .handle_err(|e| log::error!("failed to report metrics data: {e}"));

    metrics
}

struct ExperimentResult {
    param: usize,
    measure: f32
}

fn run(cli: Cli) -> Result<()> {
    let cards = parse_card_collection(&cli.card_collection)?;
    log::info!("parsed {} cards", cards.num_cards());
    deck_optim::init(cards);


    let num_trials = cli.num_trials.unwrap_or(DEFAULT_NUM_TRIALS);

    let results = (0..TOTAL_DECK_SIZE)
        .into_iter()
        .map(|param| {
            let metrics = evaluate_hyper_param(&cli, num_trials, param);
            let mut measure = metrics.average("turn-to-reach-7-plays");
            if measure == 0.0 {
                measure = 9999.0;
            }
            ExperimentResult {
                param,
                measure
            }
        })
        .collect_vec();

    // select the best parameter
    let best = results
        .iter()
        .min_by(|exp1, exp2| exp1.measure.partial_cmp(&exp2.measure).expect("no nan"));

    let mut table = make_table();
    table.set_titles(row!["# of lightning bolts", "turns until 21 damage", ""]);
    for exp in results.iter() {
        let is_best = match best {
            Some(best_exp) if best_exp.param == exp.param => "BEST",
            _ => ""
        };
        table.add_row(row![exp.param, exp.measure, is_best]);
    }


    println!();
    println!();
    println!("===================================================");
    table.printstd();

    Ok(())
}

fn main() {
    let cli = Cli::parse();
    configure_logging(&cli);

    let res = run(cli);
    if let Err(e) = res {
        println!("{}", e);
    }
}

trait ResultExt<E> {
    fn handle_err<F>(self, handler: F) where F: FnOnce(E);
}
impl <E> ResultExt<E> for Result<(), E> {
    fn handle_err<F>(self, handler: F) where F: FnOnce(E) {
        match self {
            Ok(_) => (),
            Err(e) => handler(e)
        }
    }
}
