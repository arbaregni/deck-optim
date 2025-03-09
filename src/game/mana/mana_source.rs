use std::fmt;

use itertools::Itertools;

use crate::game::annotations::AnnotationValue;
use crate::game::mana::ManaPool;
use crate::collection::Card;
use crate::game::PRODUCES_MANA_TAG;

/// A mana source is a way to produce mana.
/// Typically, this is by tapping a land, a mana rock, or a mana dork.
#[derive(Clone, PartialEq, Eq)]
pub struct ManaSource {
    pub card: Card,
    pub produces: Vec<ManaPool>
}

impl fmt::Debug for ManaSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct("ManaSource");
        dbg.field("card", &self.card.name())
            .field("produces", &format_args!("{:?}", self.produces.iter().map(|m| format!("{}", m)).collect::<Vec<_>>()));
        dbg.finish()
    }
}

impl ManaSource {
    pub fn try_from(card: Card) -> Option<Self> {
        let produces = card
            .annotations()
            .get(PRODUCES_MANA_TAG)?
            .values()
            .iter()
            .filter_map(|value| match value {
                AnnotationValue::Mana(mana) => Some(mana),
                _ => {
                    log::error!("tag with {PRODUCES_MANA_TAG} should have type Mana, instead found: {value:?}. This will be ignored");
                    None
                }
            })
            .copied()
            .collect_vec();

        if produces.is_empty() {
            log::warn!("tag with {PRODUCES_MANA_TAG} has no values. Did you intend to supply a Mana tag value?");
            return None;
        }

        Some(Self {
            card,
            produces
        })

    }

    pub fn highest_mana_value(&self) -> u8 {
        self.produces
            .iter()
            .map(|mana| mana.mana_value())
            .max()
            .unwrap_or(0)
    }
}

