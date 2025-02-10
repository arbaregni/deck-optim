use serde::{Deserialize, Serialize};

use crate::{collection::Card, game::mana::ManaPool};

use super::annotations::AnnotationValue;

#[derive(Clone,Debug,Serialize,Deserialize)]
#[allow(dead_code)]
pub struct CardData {
    pub name: String,
    pub card_type: CardType,
    pub cost: Option<ManaPool>,
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

pub const PRODUCES_MANA_TAG: &'static str = "core:Produces";

impl Card {

    // Get the mana produces by this card, as specified by the tag "core:Produces"
    pub fn produces_mana(self) -> Option<&'static ManaPool> {
        let tag = self.annotations()
            .get(PRODUCES_MANA_TAG)?;

        match tag.values() {
            [] => None,
            [AnnotationValue::Mana(mana)] => Some(mana),
            [val] => {
                log::warn!("tag with {PRODUCES_MANA_TAG} should have type Mana, instead found: {val:?}. This will be ignored");
                None
            }
            [..] => {
                log::warn!("tag with {PRODUCES_MANA_TAG} should have a single value, found {} instead. This will be ignored", tag.values().len());
                None
            }
        }


    }


}
