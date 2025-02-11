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
            for _ in 0..da.quantity() {
                cards.push(card);
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::CardCollection;
    use crate::game::CardData;
    use crate::game::CardType;
    use crate::game::ManaCost;


    fn mock_collection() -> CardCollection {
        let cards = vec![
            CardData {
                name: "Hill Giant".to_string(),
                card_type: CardType::Creature,
                cost: Some(ManaCost::try_parse("{3}{R}").expect("mana cost"))
            },
            CardData {
                name: "Lightning Bolt".to_string(),
                card_type: CardType::Instant,
                cost: Some(ManaCost::try_parse("{R}").expect("mana cost"))
            },
            CardData {
                name: "Island".to_string(),
                card_type: CardType::Land,
                cost: None,
            },
        ];
        CardCollection::from_card_data(cards)
    }

    #[test]
    fn test_count_cards() {
        let decklist = DeckList {
            decklist: vec![
                DeckAllocation { name: "Fireball".to_string(), quantity: 3 },
                DeckAllocation { name: "Lightning Bolt".to_string(), quantity: 2 },
            ],
        };
        assert_eq!(decklist.count(), 5);
    }

    #[test]
    fn test_card_names() {
        let decklist = DeckList {
            decklist: vec![
                DeckAllocation { name: "Fireball".to_string(), quantity: 3 },
                DeckAllocation { name: "Lightning Bolt".to_string(), quantity: 2 },
            ],
        };
        let mut names = decklist.card_names();
        names.sort();
        assert_eq!(names, vec!["Fireball", "Lightning Bolt"]);
    }

    #[test]
    fn test_into_deck_success() {
        let collection = mock_collection();
        let decklist = DeckList {
            decklist: vec![
                DeckAllocation { name: "Hill Giant".to_string(), quantity: 2 },
                DeckAllocation { name: "Lightning Bolt".to_string(), quantity: 1 },
            ],
        };
        let deck = decklist.into_deck(&collection).unwrap();
        assert_eq!(deck.size(), 3);
    }

    #[test]
    fn test_into_deck_missing_cards() {
        let collection = mock_collection();
        let decklist = DeckList {
            decklist: vec![
                DeckAllocation { name: "Lightning Bolt".to_string(), quantity: 2 },
                DeckAllocation { name: "Nonexistent Card".to_string(), quantity: 1 },
            ],
        };
        let result = decklist.into_deck(&collection);
        let err = result.expect_err("should fail");
        match err {
            DeckConstructionError::MissingCards { num_missing } => {
                assert_eq!(num_missing, 1);
            }
        }
    }
}
    
