use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::card_data;
use crate::game::mana::ManaPool;

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct CardCollection {
    cards: Vec<CardData>,
}

impl CardCollection {
    pub fn empty() -> Self {
        Self { cards: Vec::new() }
    }
    pub fn num_cards(&self) -> usize {
        self.cards.len()
    }
    pub fn create_name_lookup(&self) -> HashMap<String, Card> {
        let mut map = HashMap::with_capacity(self.cards.len());
        for card in self.iter() {
            let name = self.data(card).name.to_string();
            map.insert(name, card);

        }
        map
    }
    pub fn enhance_collection<S: CardSource>(&mut self, mut card_names_to_have: Vec<String>, card_source: &mut S) -> Result<(), Box<dyn std::error::Error>> {
        // todo: make this not have to recreate the name lookup
        let name_to_card = self.create_name_lookup();

        // remove all of the names that we already have
        card_names_to_have.retain(|name| !name_to_card.contains_key(name));

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
        match crate::get_card_data(*self) {
            Some(data) => write!(f, "Card{{ idx{} - \"{}\"}}", self.idx, data.name),
            None => write!(f, "Card{{ idx{} - (no data)}}", self.idx)
        }
    }
}

#[derive(Clone,Debug,Serialize,Deserialize)]
#[allow(dead_code)]
pub struct CardData {
    pub name: String,
    pub card_type: CardType,
    pub cost: Option<ManaPool>,
    pub produces: Option<ManaPool>, // tag for lands
}

#[derive(Clone,Debug,Serialize,Deserialize,Eq,PartialEq)]
pub enum CardType {
    Land,
    Instant,
}

#[allow(unused)]
pub fn get_sample_cards(num: usize) -> Vec<Card> {
    (0..num)
        .into_iter()
        .map(|idx| Card { idx })
        .collect()
}
