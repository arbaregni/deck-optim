use rand::seq::SliceRandom;

use crate::trial::Rand;
use crate::game::Card;

use super::OrderedPile;

#[derive(Clone)]
pub struct UnorderedPile {
    // a deck is some number of copies of every card / card archetype
    cards: Vec<Card>,
}

impl UnorderedPile {
    pub fn empty() -> Self {
        Self {
            cards: vec![]
        }
    }
    pub fn iter(&self) -> impl Iterator<Item=Card> + '_ {
        self.cards.iter()
            .copied()
    }
    pub fn add(&mut self, card: Card, copies: u32) {
        for _ in 0..copies {
            self.cards.push(card);
        }
    }
    pub fn size(&self) -> usize {
        self.cards.len()
    }
    pub fn to_ordered(mut self, rng: &mut Rand) -> OrderedPile {
        self.cards.shuffle(rng);
        OrderedPile::from(self.cards)
    }
    /// Removes a specified card from the UnorderedPile, if it exists
    /// ```
    /// use deck_optim::card;
    /// use deck_optim::game::UnorderedPile;
    ///
    /// let cards = card::get_sample_cards(10);
    /// let mut pile = UnorderedPile::from(cards.clone());
    ///
    /// // The card exists, so `remove` returns `true`
    /// assert_eq!(true, pile.remove(cards[2]));
    ///
    /// // The card no longer exists, so `remove` returns `false`
    /// assert_eq!(false, pile.remove(cards[2]));
    /// ```
    pub fn remove(&mut self, card: Card) -> bool {
        let Some((idx, _)) = self.cards
            .iter()
            .enumerate()
            .find(|(_, c)| **c == card)
            else {
                return false; // no card to remove
            };
        self.cards.swap_remove(idx);
        true
    }
}

impl From<Vec<Card>> for UnorderedPile {
    fn from(cards: Vec<Card>) -> Self {
        Self { cards } 
    }
}
