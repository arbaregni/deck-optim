use serde::Deserialize;

use crate::{game::Deck, get_card_named, CardNotFoundError};

#[derive(Clone,Debug,Deserialize)]
pub struct DeckList {
    decklist: Vec<DeckAllocation>
}

#[derive(Clone,Debug,Deserialize)]
pub struct DeckAllocation {
    name: String,
    quantity: usize,
}

impl DeckList {
    pub fn count(&self) -> usize {
        self.decklist
            .iter()
            .map(DeckAllocation::quantity)
            .sum()
    }
    pub fn into_deck(&self) -> Result<Deck, CardNotFoundError> {
        let mut cards = Vec::with_capacity(self.count());
        for da in self.decklist.iter() {
            let card = get_card_named(da.name.as_str())?;
            cards.push(card);
        }
        let deck = Deck::from(cards);
        Ok(deck)
    }
}

impl DeckAllocation {
    pub fn quantity(&self) ->  usize {
        self.quantity
    }
}
