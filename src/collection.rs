use std::collections::HashMap;
use once_cell::sync::OnceCell;
use thiserror::Error;
use crate::game::CardData;

#[derive(Clone,Debug)]
pub struct CardCollection {
    cards: Vec<CardData>,
    name_lookup: HashMap<String, Card>,
}

impl CardCollection {
    pub fn from_card_data(cards: Vec<CardData>) -> Self {
        let mut name_lookup = HashMap::with_capacity(cards.len());
        for (card_idx, card_data) in cards.iter().enumerate() {
            let name = card_data.name.to_string();
            let card = Card { idx: card_idx };
            name_lookup.insert(name, card);
        }
        Self {
            cards,
            name_lookup
        }
    }
    pub fn num_cards(&self) -> usize {
        self.cards.len()
    }
    pub fn card_named(&self, name: &str) -> Option<Card> {
        self.name_lookup.get(name).copied()
    }
    pub fn contains(&self, name: &str) -> bool {
        self.card_named(name).is_some()
    }
    pub fn enhance_collection<S: CardSource>(&mut self, mut card_names_to_have: Vec<String>, card_source: &mut S) -> Result<(), Box<dyn std::error::Error>> {
        // remove all of the names that we already have
        card_names_to_have.retain(|name| !self.contains(name));

        let data = card_source.get_cards_in_bulk(card_names_to_have)?;
        self.cards.extend(data);

        Ok(())
    }
    pub fn data(&self, card: Card) -> &CardData {
        &self.cards[card.idx]
    }
    pub fn get(&self, card: Card) -> Option<&CardData> {
        self.cards.get(card.idx)
    }
    pub fn iter(&self) -> impl Iterator<Item = Card> + '_ {
        (0..self.num_cards())
            .map(|idx| Card { idx })
    }
}

pub trait CardSource {
    fn get_cards_in_bulk(&mut self, card_names: Vec<String>) -> Result<Vec<CardData>, Box<dyn std::error::Error>>;
}

#[derive(Clone,Copy,Eq,PartialEq,Hash,PartialOrd,Ord)]
pub struct Card {
    idx: usize
}
impl Card {
    pub fn data(self) -> &'static CardData {
        card_data(self)
    }
}
impl std::fmt::Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match get_card_data(*self) {
            Some(data) => write!(f, "Card{{ idx{} - \"{}\"}}", self.idx, data.name),
            None => write!(f, "Card{{ idx{} - (no data)}}", self.idx)
        }
    }
}

static CARD_COLLECTION: OnceCell<CardCollection> = OnceCell::new();

pub fn init(card_collection: CardCollection) {
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
    let col = CARD_COLLECTION.get()
        .ok_or_else(|| CardNotFoundError::NameLookupUnintialized { card_name: name.to_string() })?;
    col.card_named(name)
        .ok_or_else(|| CardNotFoundError::CardNameNotPresent { card_name: name.to_string() })
}
pub fn card_named(name: &str) -> Card {
    CARD_COLLECTION.get()
        .expect("unitialized")
        .card_named(name)
        .expect("missing card")
}

#[derive(Debug,Error)]
pub enum CardNotFoundError {
    #[error("unable to lookup card with name '{card_name}', card collection is not initialized")]
    NameLookupUnintialized { card_name: String },
    #[error("could not find a card with name '{card_name}'")]
    CardNameNotPresent { card_name: String }
}

#[allow(unused)]
pub fn get_sample_cards(num: usize) -> Vec<Card> {
    (0..num)
        .into_iter()
        .map(|idx| Card { idx })
        .collect()
}
