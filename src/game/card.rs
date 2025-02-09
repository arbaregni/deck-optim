use serde::{Deserialize, Serialize};

use crate::game::mana::ManaPool;

#[derive(Clone,Debug,Serialize,Deserialize)]
#[allow(dead_code)]
pub struct CardData {
    pub name: String,
    pub card_type: CardType,
    pub cost: Option<ManaPool>,
    pub produces: Option<ManaPool>, // tag for lands
}

#[derive(Clone,Debug,Serialize,Deserialize,Eq,PartialEq)]
pub enum SuperType {
    Legendary,
    Basic,
    Snow,
    World
}

#[derive(Clone,Debug,Serialize,Deserialize,Eq,PartialEq)]
pub enum CardType {
    Land,
    Instant,
    Creature,
    Sorcery,
}

