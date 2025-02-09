pub mod ordered_pile;
pub mod unordered_pile;
pub mod state;
pub mod card;
pub mod mana;
pub mod annotations;

pub use ordered_pile::*;
pub use unordered_pile::*;
pub use state::*;
pub use card::*;
pub use mana::*;

pub type Deck = UnorderedPile;
pub type Library = OrderedPile;

pub type Hand = UnorderedPile;
