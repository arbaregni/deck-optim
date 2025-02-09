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

pub const PRODUCES_MANA_TAG: &'static str = "produces";

impl Card {

    pub fn produces_mana(self) -> Option<&'static ManaPool> {
        let mut tag_values = self.annotations()
            .filter(|a| a.key == PRODUCES_MANA_TAG);

        let tag = tag_values.next()?;

        if let Some(next_tag) = tag_values.next() {
            log::warn!("multiple tags with key {PRODUCES_MANA_TAG} found on {self:?}. the first ({tag:?}) will be used; ({next_tag:?}) and any others will be ignored");
        }

        match tag.value.as_ref() {
            Some(AnnotationValue::Mana(mana_pool)) => Some(mana_pool),
            Some(value) => {
                log::warn!("tag with key {PRODUCES_MANA_TAG} should use a mana value, instead it is {value:?}. will be ignored");
                None
            }
            None => None
        }
    }


}
