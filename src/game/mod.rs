mod ordered_pile;
mod unordered_pile;
mod state;

pub use ordered_pile::*;
pub use unordered_pile::*;
pub use state::*;

pub type Deck = UnorderedPile;
pub type Library = OrderedPile;

pub type Hand = UnorderedPile;
