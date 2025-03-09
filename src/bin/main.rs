use std::path::PathBuf;

use clap::Parser;

use deck_optim::collection::CardSource;
use deck_optim::game::annotations::CardAnnotations;
use deck_optim::game::Deck;
use deck_optim::scryfall::ScryfallClient;
use deck_optim::deck::DeckList;
use deck_optim::strategies::StrategyImpl;
use deck_optim::trial;
use directories::ProjectDirs;
use itertools::Itertools;
use prettytable::{row, Table};
use rand::SeedableRng;

use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::filter::LevelFilter;

use deck_optim::collection::CardCollection;
use deck_optim::metrics::MetricsData;
use deck_optim::watcher::WatcherImpl;

use deck_optim::card_cache::LocalCardCache;
use deck_optim::file_utils;

type Result<T, E=Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[arg(long)]
    /// Supply this to force the cache to be refreshed
    pub refresh: bool,

    #[arg(short='t', long)]
    pub num_trials: Option<u32>,

    #[arg(long)]
    pub max_turns: Option<u32>,

    #[arg(long)]
    /// Supply this parameter to change the default level filters
    pub level_filter: Option<LevelFilter>,

    #[arg(short='d', long)]
    pub deck_list: PathBuf,
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

pub fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "arbaregni", deck_optim::PROJECT_NAME)
        .ok_or_else(|| {
            format!("could not load cache directory for {}", deck_optim::PROJECT_NAME).into()
        })
}

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

fn evaluate_deck(cli: &Cli, deck: Deck) -> MetricsData {
    let watcher = WatcherImpl;
    let strategies = StrategyImpl {
        rng: rand::rngs::StdRng::from_entropy()
    };

    let props = trial::Props {
        num_trials: cli.num_trials.unwrap_or(10_000),
        max_turn: cli.max_turns.unwrap_or(12),
    };
    let metrics = trial::run_trials(deck, strategies, watcher, props);
    
    report_metrics_data(&cli, &metrics)
        .handle_err(|e| log::error!("failed to report metrics data: {e}"));

    metrics
}

const CARD_CACHE_FILENAME: &'static str = "cards.json";
fn card_cache_path() -> Result<PathBuf> {
    let project_dirs = project_dirs()?;
    let card_cache = project_dirs.cache_dir().join(CARD_CACHE_FILENAME);
    Ok(card_cache)
}

const ANNOTATIONS_FILENAME: &'static str = "annotations.json";
fn annotations_path() -> Result<PathBuf> {
    let project_dirs = project_dirs()?;
    let path = project_dirs.data_dir().join(ANNOTATIONS_FILENAME);
    Ok(path)
}

fn load_annotations(_cli: &Cli) -> Result<CardAnnotations> {
    let path = annotations_path()?;

    log::info!("Loading annotations from project directory {}", path.display());
    let annotations: CardAnnotations = file_utils::read_json_from_path(&path)?;

    Ok(annotations)
}

fn load_card_data(scenario: Vec<&str>, cli: &Cli, card_cache: &mut LocalCardCache, scryfall_client: &mut ScryfallClient) -> Result<CardCollection> {
    let mut cards;
    if cli.refresh {
        log::info!("Refresh was requested, loading all card data from: {scryfall_client:?}");
        cards = CardCollection::from_source(&scenario, scryfall_client)?;
    } else {
        cards = CardCollection::from_source(&scenario, &mut card_cache.chain(scryfall_client))?;
    }

    log::info!("found {} total cards", cards.num_cards());

    log::info!("writing back to cache...");
    card_cache.save(cards.all_card_data());

    match load_annotations(cli) {
        Ok(annotations) => {
            log::info!("found {} annotations, applying them now", annotations.len());
            cards.apply_annotations(annotations);
        }
        Err(e) => {
            log::error!("could not load annotations due to {e}");
        }
    }

    log::debug!("loaded cards: {cards:#?}");

    Ok(cards)
}

fn run(cli: Cli) -> Result<()> {
    let card_cache = card_cache_path()?;
    let mut card_cache = LocalCardCache::from(card_cache);
    let mut scryfall_client = ScryfallClient::new();

    log::info!("loading deck from file");
    let decklist: DeckList = file_utils::read_json_from_path(&cli.deck_list)?;
    log::info!("openned deck, has {} cards", decklist.count());


    let scenario = decklist.card_names();

    let cards = load_card_data(scenario, &cli, &mut card_cache, &mut scryfall_client)?;
    let deck = decklist.into_deck(&cards)
        .inspect_err(|e| log::error!("error while loading deck list: {e}"))?;

    deck_optim::init(cards);

    // do the trial

    let _metrics = evaluate_deck(&cli, deck);

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
