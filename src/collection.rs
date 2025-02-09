use std::collections::HashMap;

use crate::game::{annotations::{Annotation, AnnotationTarget, CardAnnotations}, CardData};

#[derive(Clone,Debug)]
pub struct CardCollection {
    cards: Vec<CardData>,
    name_lookup: HashMap<String, Card>,
    annotations: HashMap<Card, Vec<Annotation>>,
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
    pub fn from_source<'a, S: CardSource>(card_names: &[&'a str], source: &mut S) -> Result<Self, DynError> {
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
                        log::debug!("annotation is targetting {card_name}, but it does not exist");
                        continue;
                    };
                    self.annotations.entry(card)
                        .or_default()
                        .push(annotation.clone());
                }
            });
    }
    pub fn get_annotations(&self, card: Card) -> &[Annotation] {
        const EMPTY: &'static [Annotation] = &[];
        self.annotations.get(&card)
            .map(Vec::as_slice)
            .unwrap_or(EMPTY)
    }
}

type DynError = Box<dyn std::error::Error>;

/// A way to supply card data to the card collection
pub trait CardSource {
    fn retrieve_cards(&mut self, _card_names: &[&str]) -> Result<Vec<CardData>, DynError>;

    /// Creates a new card source that attempts to pull from this, then uses another card source as
    /// a backup
    fn chain<'a, S: CardSource + 'a>(&'a mut self, other: &'a mut S) -> ChainCardSource<'a> 
    where Self : Sized + 'a 
    { 
        ChainCardSource::from(self).extend(other)
    }
}

/// A way to combine card sources
pub struct ChainCardSource<'a> {
    sources: Vec<&'a mut dyn CardSource>
}
impl <'a> ChainCardSource<'a> {
    pub fn from<S: CardSource + 'a>(card_source: &'a mut S) -> Self {
        Self {
            sources: vec![card_source]
        }
    }
    pub fn extend<S: CardSource + 'a>(mut self, other: &'a mut S) -> Self {
        self.sources.push(other);
        self
    }
}
impl <'a> CardSource for ChainCardSource<'a> {
    fn retrieve_cards(&mut self, card_names: &[&str]) -> Result<Vec<CardData>, DynError> {
        let mut card_data = Vec::with_capacity(card_names.len());
        let mut still_required = card_names.to_vec();

        for s in self.sources.iter_mut() {
            let new_cards = s.retrieve_cards(still_required.as_mut())?;
            still_required.retain(|card_name| new_cards.iter().all(|card| &card.name != card_name));
            log::info!("adding {} cards to card data", new_cards.len());
            card_data.extend(new_cards);
        }

        still_required
            .into_iter()
            .for_each(|name| log::error!("unable to locate a card named '{name}'"));

        Ok(card_data)
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
    pub fn annotations(self) -> impl Iterator<Item = &'static Annotation> {
        global_collection::get_card_annotations(self).expect("card collection initialized").iter()
    }
    pub fn has_annotation(self, key: &str) -> bool {
        self.annotations().any(|a| a.key == key)
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
    pub fn get_card_annotations(card: Card) -> Result<&'static [Annotation], CardNotFoundError> {
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
