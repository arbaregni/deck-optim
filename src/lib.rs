use std::collections::HashMap;

use once_cell::sync::OnceCell;

pub mod trial;
pub mod game;
pub mod watcher;
pub mod strategies;
pub mod metrics;
pub mod deck;

pub mod scryfall;

use game::card::{Card, CardCollection, CardData};

pub const PROJECT_NAME: &'static str = "deck-optim-0.1.0";

static CARD_COLLECTION: OnceCell<CardCollection> = OnceCell::new();
static NAME_TO_CARD: OnceCell<HashMap<String, Card>> = OnceCell::new();

pub fn init(card_collection: CardCollection) {
    let name_to_card = card_collection.create_name_lookup();

    NAME_TO_CARD.set(name_to_card)
        .expect("initialization");
    CARD_COLLECTION.set(card_collection)
        .expect("initialization");
}

pub fn card_data(card: Card) -> &'static CardData {
    CARD_COLLECTION.get()
        .expect("unitialized")
        .data(card)
}
pub fn get_card_data(card: Card) -> Option<&'static CardData> {
    let col = CARD_COLLECTION.get()?;
    col.get(card)
}

pub fn get_card_named(name: &str) -> Result<Card, CardNotFoundError> {
    let col = NAME_TO_CARD.get()
        .ok_or_else(|| CardNotFoundError::NameLookupUnintialized { card_name: name.to_string() })?;
    col.get(name)
        .copied()
        .ok_or_else(|| CardNotFoundError::CardNameNotPresent { card_name: name.to_string() })
}
pub fn card_named(name: &str) -> Card {
    NAME_TO_CARD.get()
        .expect("unitialized")
        .get(name)
        .copied()
        .expect("missing card")
}

#[derive(Debug)]
pub enum CardNotFoundError {
    NameLookupUnintialized {
        card_name: String,
    },
    CardNameNotPresent {
        card_name: String,
    }
}
impl std::fmt::Display for CardNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CardNotFoundError::NameLookupUnintialized { card_name } => write!(f, "unable to lookup card with name '{card_name}', card collection not initialized"),
            CardNotFoundError::CardNameNotPresent { card_name } => write!(f, "could not find a card with name '{card_name}'")
        }
    }
}
impl std::error::Error for CardNotFoundError { }
