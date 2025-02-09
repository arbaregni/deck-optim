use serde::Deserialize;
use thiserror::Error;

use crate::collection::CardCollection;
use crate::game::Deck;

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
    pub fn card_names(&self) -> Vec<&str> {
        self.decklist.iter()
            .map(|da| da.name.as_str())
            .collect()
    }
    pub fn into_deck(&self, collection: &CardCollection) -> Result<Deck, DeckConstructionError> {
        let mut num_missing = 0;
        let mut cards = Vec::with_capacity(self.count());
        for da in self.decklist.iter() {
            let name = da.name.as_str();
            let Some(card) = collection.card_named(name) else {
                log::error!("could not construct deck - no card with name '{name}'");
                num_missing += 1;
                continue
            };
            cards.push(card);
        }
        if num_missing > 0 {
            return Err(DeckConstructionError::MissingCards { num_missing })
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

#[derive(Debug,Error)]
pub enum DeckConstructionError {
    #[error("unable to construct deck - unable to find {num_missing} cards")]
    MissingCards { num_missing: usize }
}
