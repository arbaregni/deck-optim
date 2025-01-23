use std::collections::HashMap;

use card::{Card, CardCollection, CardData};
use once_cell::sync::OnceCell;

pub mod card;
pub mod mana;
pub mod trial;
pub mod game;
pub mod watcher;
pub mod measure;
pub mod strategies;

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

pub fn card_named(name: &str) -> Card {
    NAME_TO_CARD.get()
        .expect("unitialized")
        .get(name)
        .copied()
        .expect("missing card")
}
