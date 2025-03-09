use serde::{Deserialize, Serialize};

use crate::collection::Card;
use crate::game::annotations::Annotation;
use crate::game::mana::ManaCost;

use super::annotations::AnnotationValue;

#[derive(Clone,Debug,Serialize,Deserialize)]
#[allow(dead_code)]
pub struct CardData {
    pub name: String,
    pub card_type: CardType,
    pub cost: Option<ManaCost>,
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
    Artifact,
    Enchantment,
    Planeswalker
}

pub const PRODUCES_MANA_TAG: &'static str = "core:Produces";
pub const GAME_EFFECT_TAG: &'static str = "core:GameEffect";

impl Card {
    /// Get the name of the card
    pub fn name(self) -> &'static str {
        self.data().name.as_str()
    }

    pub fn effects(self) -> &'static [AnnotationValue] {
        const EMPTY: &'static [AnnotationValue] = &[];
        self.annotations().get(GAME_EFFECT_TAG)
            .map(Annotation::values)
            .unwrap_or(EMPTY) 
    }

}
