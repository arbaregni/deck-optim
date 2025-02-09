use rand::Rng;

use crate::collection::Card;

use super::UnorderedPile;

#[derive(Clone)]
pub struct OrderedPile {
    cards: Vec<Card>
}

impl OrderedPile {
    pub fn empty() -> Self {
        Self { cards: vec![] } 
    }
    pub fn from(cards: Vec<Card>) -> Self {
        Self { cards }
    }
    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }
    pub fn draw_n(&mut self, n: usize) -> Vec<Card> {
        let mut hand = Vec::with_capacity(n);
        for _ in 0..n {
            let Some(card) = self.draw() else { break };
            hand.push(card)
        }
        hand
    }
    pub fn add_to_top(&mut self, other_pile: &UnorderedPile) {
        for c in other_pile.iter() {
            self.cards.push(c);
        }
    }
    pub fn shuffle<R: Rng>(&mut self, rng: &mut R) {
        use rand::seq::SliceRandom;
        self.cards.shuffle(rng)
    }
    pub fn shuffled<R: Rng>(mut self, rng: &mut R) -> Self {
        self.shuffle(rng);
        self
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
