pub mod trial;
pub mod game;
pub mod watcher;
pub mod strategies;
pub mod metrics;
pub mod deck;
pub mod experiment;

pub mod collection;
pub use collection::card_cache;

pub mod scryfall;

pub mod file_utils;
pub mod opt_utils;

pub const PROJECT_NAME: &'static str = "deck-optim-0.1.0";

pub use collection::init;
