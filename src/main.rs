use clap::Parser;

use deck_optim::game::Deck;
use deck_optim::scryfall::ScryfallClient;
use deck_optim::deck::DeckList;
use deck_optim::strategies::StrategyImpl;
use deck_optim::trial::run_trials;
use directories::ProjectDirs;
use itertools::Itertools;
use prettytable::{row, Table};
use rand::SeedableRng;

use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::filter::LevelFilter;

use deck_optim::game::card::CardCollection;
use deck_optim::metrics::MetricsData;
use deck_optim::watcher::WatcherImpl;

mod cli;
use cli::Cli;

mod file_utils;

type Result<T, E=Box<dyn std::error::Error>> = std::result::Result<T, E>;

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

pub fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "arbaregni", deck_optim::PROJECT_NAME)
        .ok_or_else(|| {
            format!("could not load cache directory for {}", deck_optim::PROJECT_NAME).into()
        })
}

const DEFAULT_NUM_TRIALS: u32 = 10000; 

fn make_table() -> Table {
    let mut table = Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);
    table
}

fn report_metrics_data(_cli: &Cli, metrics: &MetricsData) -> Result<()> {
    let mut table = make_table();

    table.set_titles(row!["Metrics Name", "Average"]);
    for key in metrics.keys().sorted() {

        let avg = metrics.average(key);
        table.add_row(row![key, avg]);

    }

    table.printstd();

    Ok(())
}



fn evaluate_deck(cli: &Cli, num_trials: u32, deck: Deck) -> MetricsData {

    let watcher = WatcherImpl;
    let strategies = StrategyImpl {
        rng: rand::rngs::StdRng::from_entropy()
    };

    let metrics = run_trials(num_trials, deck, strategies, watcher);
    
    report_metrics_data(&cli, &metrics)
        .handle_err(|e| log::error!("failed to report metrics data: {e}"));

    metrics
}

struct ExperimentResult {
    param: usize,
    measure: f32
}

/*
 *let results = (0..TOTAL_DECK_SIZE)
        .into_iter()
        .map(|param| {
            let metrics = evaluate_deck(&cli, num_trials, param);
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

*/

const CARD_CACHE_FILENAME: &'static str = "cards.json";

fn load_card_data(cli: Cli) -> Result<CardCollection> {
    let project_dirs = project_dirs()?;
    let card_cache = project_dirs.cache_dir().join(CARD_CACHE_FILENAME);

    let scenario = vec!["Lightning Bolt".to_string(), "Mountain".to_string()];

    let mut cards = CardCollection::empty();

    if !cli.refresh {
        log::info!("opening card cache at {}", card_cache.display());

        match file_utils::read_json_from_path(&card_cache) {
            Ok(c) => {
                log::info!("read {} cards from cache", cards.num_cards());
                cards = c;
            }
            Err(e) => {
                log::warn!("could not read from card cache {}: it will be refreshed entirely", card_cache.display());
                log::warn!("error reading from card cache: {e}");
            }
        }
    }

    let mut scryfall_client = ScryfallClient::new();

    log::info!("need a total of {} cards from scenario", scenario.len());
    cards.enhance_collection(scenario, &mut scryfall_client)?;

    log::info!("found {} total cards", cards.num_cards());

    log::info!("writing back to cache...");
    file_utils::write_json_to_path(&card_cache, &cards)
        .handle_err(|e| {
            log::error!("unable to save back to card cache at {} due to {e}. Continueing...", card_cache.display());
        });

    Ok(cards)
}

fn run(cli: Cli) -> Result<()> {
    let cards = load_card_data(cli)?;
    deck_optim::init(cards);

    // set up scryfall data
    let mut client = ScryfallClient::new();

    let results = client.get_card_collection(["Lightning Bolt", "Mountain"])?;
    log::info!("got results from scryfall: {results:#?}");

    return Ok(());

    let decklist: DeckList = file_utils::read_json_from_path(cli.deck_list.as_ref().expect("expected decklist"))?;
    log::info!("openned deck, has {} cards", decklist.count());

    let num_trials = cli.num_trials.unwrap_or(DEFAULT_NUM_TRIALS);

    let deck = decklist.into_deck()?;

    let metrics = evaluate_deck(&cli, num_trials, deck);

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
