use rand::Rng;

use crate::collection::Card;

use super::UnorderedPile;

#[derive(Clone,Debug)]
pub struct OrderedPile {
    cards: Vec<Card>
}

impl OrderedPile {
    /// Creates an empty pile.
    ///
    /// # Example
    /// ```
    /// use deck_optim::game::OrderedPile;
    ///
    /// let deck = OrderedPile::empty();
    /// assert_eq!(deck.size(), 0);
    /// ```
    pub fn empty() -> Self {
        Self { cards: vec![] } 
    }
    /// Creates an [`OrderedPile`] from a vector of cards
    pub fn from(cards: Vec<Card>) -> Self {
        Self { cards }
    }
    /// Gets the number of cards in this pile.
    pub fn size(&self) -> usize {
        self.cards.len()
    }
    /// Draw the top card of this pile, if there is one.
    /// The card will be removed from the pile.
    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }
    /// Draw the top `n` cards of this pile.
    /// The cards will be removed from this pile.
    pub fn draw_n(&mut self, n: usize) -> Vec<Card> {
        let mut hand = Vec::with_capacity(n);
        for _ in 0..n {
            let Some(card) = self.draw() else { break };
            hand.push(card)
        }
        hand
    }
    /// Iterate over all cards in the pile
    pub fn iter(&self) -> impl Iterator<Item = Card> + '_ {
        self.cards
            .iter()
            .copied()
    }
    /// Add all cards in `other_pile` on top of this pile.
    pub fn add_to_top(&mut self, other_pile: &UnorderedPile) {
        for c in other_pile.iter() {
            self.cards.push(c);
        }
    }
    /// Shuffle this pile.
    pub fn shuffle<R: Rng>(&mut self, rng: &mut R) {
        use rand::seq::SliceRandom;
        self.cards.shuffle(rng)
    }
}

#[cfg(test)]
mod tests {
    use crate::collection::get_sample_cards;
    use super::*;

    #[test]
    fn test_draw_single_card() {
        let cards = get_sample_cards(3);
        let mut pile = OrderedPile { cards: cards.clone() };

        let drawn_card = pile.draw();

        assert_eq!(drawn_card, Some(cards[2]));
        assert_eq!(pile.cards, vec![cards[0], cards[1]]);
    }

    #[test]
    fn test_draw_from_empty_pile() {
        let mut pile = OrderedPile { cards: Vec::new() };

        let drawn_card = pile.draw();

        assert_eq!(drawn_card, None);
        assert_eq!(pile.cards, Vec::<Card>::new());
    }

    #[test]
    fn test_draw_n_fewer_than_available() {
        let cards = get_sample_cards(3);
        let mut pile = OrderedPile { cards: cards.clone() };

        let hand = pile.draw_n(2);

        assert_eq!(hand, vec![cards[2], cards[1]]);
        assert_eq!(pile.cards, vec![cards[0]]);
    }

    #[test]
    fn test_draw_n_exactly_available() {
        let cards = get_sample_cards(3);
        let mut pile = OrderedPile { cards: cards.clone() };

        let hand = pile.draw_n(3);

        assert_eq!(hand, vec![cards[2], cards[1], cards[0]]);
        assert_eq!(pile.cards, Vec::<Card>::new());
    }

    #[test]
    fn test_draw_n_more_than_available() {
        let cards = get_sample_cards(3);
        let mut pile = OrderedPile { cards: cards.clone() };

        let hand = pile.draw_n(5);

        assert_eq!(hand, vec![cards[2], cards[1], cards[0]]);
        assert_eq!(pile.cards, Vec::<Card>::new());
    }

    #[test]
    fn test_draw_n_from_empty_pile() {
        let mut pile = OrderedPile { cards: Vec::new() };

        let hand = pile.draw_n(3);

        assert_eq!(hand, Vec::<Card>::new());
        assert_eq!(pile.cards, Vec::<Card>::new());
    }
}
