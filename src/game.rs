pub mod ordered_pile;
pub mod unordered_pile;
pub mod state;
pub mod card;
pub mod mana;
pub mod annotations;
pub mod card_play;

pub use ordered_pile::*;
pub use unordered_pile::*;
pub use state::*;
pub use card::*;
pub use mana::*;

#[derive(Clone,Debug)]
pub struct Deck {
    pub command_zone: CommandZone,
    pub deck: UnorderedPile
}

pub type CommandZone = UnorderedPile;
pub type Library = OrderedPile;

pub type Hand = UnorderedPile;
pub type Graveyard = UnorderedPile;
pub type Battlefield = UnorderedPile;

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum Zone {
    Library,
    CommandZone,
    Hand,
    Graveyard,
    Battlefield
}
