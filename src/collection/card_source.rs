use crate::game::CardData;

type DynError = Box<dyn std::error::Error>;

/// A way to supply card data to the card collection
pub trait CardSource : std::fmt::Debug {
    fn retrieve_cards(&mut self, _card_names: &[&str]) -> Result<Vec<CardData>, DynError>;

    /// Creates a new card source that attempts to pull from this, then uses another card source as
    /// a backup
    fn chain<'a, S: CardSource + 'a>(&'a mut self, other: &'a mut S) -> ChainCardSource<'a> 
    where Self : Sized + 'a 
    { 
        ChainCardSource::from(self).extend(other)
    }
}

/// A way to combine card sources
pub struct ChainCardSource<'a> {
    sources: Vec<&'a mut dyn CardSource>
}
impl <'a> ChainCardSource<'a> {
    pub fn from<S: CardSource + 'a>(card_source: &'a mut S) -> Self {
        Self {
            sources: vec![card_source]
        }
    }
    pub fn extend<S: CardSource + 'a>(mut self, other: &'a mut S) -> Self {
        self.sources.push(other);
        self
    }
}
impl <'a> CardSource for ChainCardSource<'a> {
    fn retrieve_cards(&mut self, card_names: &[&str]) -> Result<Vec<CardData>, DynError> {
        let mut card_data = Vec::with_capacity(card_names.len());
        let mut still_required = card_names.to_vec();

        for s in self.sources.iter_mut() {
            let new_cards = s.retrieve_cards(still_required.as_mut())?;
            still_required.retain(|card_name| new_cards.iter().all(|card| &card.name != card_name));
            log::info!("adding {} cards to card data from {s:?}", new_cards.len());
            card_data.extend(new_cards);
        }

        still_required
            .into_iter()
            .for_each(|name| log::error!("unable to locate a card named '{name}'"));

        Ok(card_data)
    }
}

impl <'a> std::fmt::Debug for ChainCardSource<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(Chain Source: {:?})", self.sources)
    }
}
