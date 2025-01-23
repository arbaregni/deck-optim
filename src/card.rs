use std::collections::HashMap;

use serde::Deserialize;

use crate::{card_data, mana::ManaPool};

#[derive(Clone,Debug,Deserialize)]
pub struct CardCollection {
    cards: Vec<CardData>,
}

impl CardCollection {
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
    pub fn data(&self, card: Card) -> &CardData {
        &self.cards[card.idx]
    }
    pub fn iter(&self) -> impl Iterator<Item = Card> + '_ {
        (0..self.num_cards())
            .map(|idx| Card { idx })
    }
}

#[derive(Debug,Clone,Copy,Eq,PartialEq,Hash)]
pub struct Card {
    idx: usize
}
impl Card {
    pub fn data(self) -> &'static CardData {
        card_data(self)
    }
}

#[derive(Clone,Debug,Deserialize)]
#[allow(dead_code)]
pub struct CardData {
    pub name: String,
    pub card_type: CardType,
    pub cost: Option<ManaPool>,
    pub produces: Option<ManaPool>, // tag for lands
}

#[derive(Clone,Debug,Deserialize,Eq,PartialEq)]
pub enum CardType {
    Land,
    Instant
}

#[allow(unused)]
pub fn get_sample_cards(num: usize) -> Vec<Card> {
    (0..num)
        .into_iter()
        .map(|idx| Card { idx })
        .collect()
}
