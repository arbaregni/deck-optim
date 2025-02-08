pub mod trial;
pub mod game;
pub mod watcher;
pub mod strategies;
pub mod metrics;
pub mod deck;
pub mod collection;

pub mod scryfall;

pub const PROJECT_NAME: &'static str = "deck-optim-0.1.0";

pub use collection::init;
