use std::fmt;
use std::path::PathBuf;

use crate::collection::CardSource;
use crate::game::CardData;
use crate::file_utils;

/// Provides a source of cards that has been cached locally.
pub struct LocalCardCache {
    file_path: PathBuf
}
impl LocalCardCache {
    /// Initialize a card cache from a specific path
    pub fn from(file_path: PathBuf) -> Self {
        Self { file_path }
    }
    pub fn save(&mut self, card_data: &[CardData]) {
        match file_utils::write_json_to_path(&self.file_path, &card_data) {
            Ok(_) => {}
            Err(e) => {
                log::error!("unable to save back to card cache at {self} due to: {e}");
            }
        }
    }
}

impl CardSource for LocalCardCache {
    fn retrieve_cards(&mut self, card_names: &[&str]) -> Result<Vec<CardData>, Box<dyn std::error::Error>> {
        log::info!("opening card cache at {}", self.file_path.display());

        let mut cards: Vec<CardData> = file_utils::read_json_from_path(&self.file_path)
            .unwrap_or_else(|e| {
                log::warn!("Card cache will be refreshed. could not read from {self}, due to: {e}.");
                Vec::new()
            });

        log::info!("read {} cards from cache", cards.len());
        cards.retain(|card| card_names.contains(&card.name.as_str()));

        Ok(cards)

    }
}

impl fmt::Display for LocalCardCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.file_path.display())
    }
}
