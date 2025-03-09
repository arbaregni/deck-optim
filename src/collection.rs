mod card_source;
pub use card_source::CardSource;

pub mod card_cache;

use std::collections::HashMap;

use crate::game::CardData;
use crate::game::annotations::{
    AnnotationSet,
    AnnotationTarget,
    CardAnnotations,
};

#[derive(Clone,Debug)]
pub struct CardCollection {
    cards: Vec<CardData>,
    name_lookup: HashMap<String, Card>,
    annotations: HashMap<Card, AnnotationSet>,
}

impl CardCollection {
    /// Creates an empty collection
    pub fn empty() -> Self {
        Self {
            cards: Vec::new(),
            name_lookup: HashMap::new(),
            annotations: HashMap::new(),
        }
    }
    /// Initialize a collection from a vector of card data
    pub fn from_card_data(cards: Vec<CardData>) -> Self {
        let mut name_lookup = HashMap::with_capacity(cards.len());
        for (card_idx, card_data) in cards.iter().enumerate() {
            let name = card_data.name.to_string();
            let card = Card { idx: card_idx };
            name_lookup.insert(name, card);
        }
        Self {
            cards,
            name_lookup,
            annotations: HashMap::new(),
        }
    }
    /// Initialize from a list of sources
    pub fn from_source<'a, S: CardSource>(card_names: &[&'a str], source: &mut S) -> Result<Self, Box<dyn std::error::Error>> {
        let card_data = source.retrieve_cards(card_names)?;
        let col = CardCollection::from_card_data(card_data);
        Ok(col)
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
    pub fn card_data(&self, card: Card) -> &CardData {
        &self.cards[card.idx]
    }
    pub fn get(&self, card: Card) -> Option<&CardData> {
        self.cards.get(card.idx)
    }
    pub fn iter(&self) -> impl Iterator<Item = Card> + '_ {
        (0..self.num_cards())
            .map(|idx| Card { idx })
    }
    pub fn all_card_data(&self) -> &[CardData] {
        self.cards.as_slice()
    }
    pub fn apply_annotations(&mut self, annotations: CardAnnotations) {
        annotations
            .into_iter()
            .for_each(|an| {
                let AnnotationTarget { targets, annotation } = an;
                for card_name in targets {
                    let Some(card) = self.card_named(card_name.as_str()) else {
                        continue;
                    };
                    self.annotations.entry(card)
                        .or_default()
                        .insert(annotation.clone());
                }
            });
    }
    pub fn get_annotations(&self, card: Card) -> &AnnotationSet {
        const EMPTY: &'static AnnotationSet = &AnnotationSet::empty();
        self.annotations.get(&card)
            .unwrap_or(EMPTY)
    }
}


/// An opaque type that indexes into a CardCollection
#[derive(Clone,Copy,Eq,PartialEq,Hash,PartialOrd,Ord)]
pub struct Card {
    idx: usize
}
impl std::fmt::Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match global_collection::get_card_data(*self) {
            Some(data) => write!(f, "Card{{ idx{} - \"{}\"}}", self.idx, data.name),
            None => write!(f, "Card{{ idx{} - (no data)}}", self.idx)
        }
    }
}

impl Card {
    /// Retrieves the globally registered card data.
    /// Panics if the global collection has not been initialized.
    pub fn data(self) -> &'static CardData {
        global_collection::get_card_data(self).expect("card collection initialized")
    }
    pub fn annotations(self) -> &'static AnnotationSet {
        global_collection::get_card_annotations(self).expect("card collection initialized")
    }
    pub fn has_annotation(self, key: &str) -> bool {
        self.annotations().get(key).is_some()
    }
}


mod global_collection {
    use super::*;

    use once_cell::sync::OnceCell;
    use thiserror::Error;

    static CARD_COLLECTION: OnceCell<CardCollection> = OnceCell::new();

    pub fn init(card_collection: CardCollection) {
        CARD_COLLECTION.set(card_collection)
            .expect("initialization");
    }

    /// Retrieves the card data from a globally initialized card collection
    pub fn get_card_data(card: Card) -> Option<&'static CardData> {
        let col = CARD_COLLECTION.get()?;
        col.get(card)
    }

    /// Retrieves the card annotations from a globally initialzied card collection
    pub fn get_card_annotations(card: Card) -> Result<&'static AnnotationSet, CardNotFoundError> {
        let col = CARD_COLLECTION.get()
            .ok_or_else(|| CardNotFoundError::NotInitialized { card })?;
        let annot = col.get_annotations(card);
        Ok(annot)
    }

    #[derive(Debug,Error)]
    pub enum CardNotFoundError {
        #[error("unable to lookup card '{card:?}', card collection is not initialized")]
        NotInitialized { card: Card },
    }
}
pub use global_collection::init;

#[allow(unused)]
pub fn get_sample_cards(num: usize) -> Vec<Card> {
    (0..num)
        .into_iter()
        .map(|idx| Card { idx })
        .collect()
}
