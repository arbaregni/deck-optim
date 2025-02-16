use crate::collection::Card;
use crate::game::Zone;
use crate::game::mana::ManaPool;

/// Contains all information necessary to make a card play
#[derive(Debug, Clone)]
pub struct CardPlay {
    /// What card is being played?
    pub card: Card,
    /// The origin of the card. Usually this will be from the hand, but could also be from the
    /// command zone, library, or graveyard.
    pub zone: Zone,
    // The mana we are using to pay for this card.
    pub payment: ManaPool
}
