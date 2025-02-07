use crate::game;
use crate::scryfall::types;

/// Error in translating scryfall's card data into our model
#[derive(thiserror::Error, Debug)]
pub enum ConversionError {
    #[error("could not convert type line: `{type_line}` is unknown")]
    UnknownCardType { type_line: String },
    #[error("could not parse mana cost: `{mana_cost}` due to `{source}`")]
    CannotParseManaCost { mana_cost: String, source: game::ManaPoolParseError },
}

pub fn convert_mana_cost(mana_cost: String) -> Result<Option<game::ManaPool>, ConversionError> {
    if mana_cost == "" {
        // empty mana cost means there is no mana cost
        return Ok(None);
    }

    let cost = game::ManaPool::try_parse(mana_cost.as_str())
        .map_err(|source| ConversionError::CannotParseManaCost {
            mana_cost, source
        })?;

    Ok(Some(cost))
}

pub fn convert_type_line(type_line: String) -> Result<game::CardType, ConversionError> {
    let card_type = match type_line.to_lowercase().as_str() {
        "instant" => game::card::CardType::Instant,
        "land" => game::card::CardType::Land,
        _ => return Err(ConversionError::UnknownCardType {
            type_line
        })
    };
    Ok(card_type)
}

/// Converts a scryfall card into a game::card::CardData
pub fn convert_card(card: types::CardData) -> Result<game::CardData, ConversionError> {
    let card_type = convert_type_line(card.type_line)?;
    let cost = convert_mana_cost(card.mana_cost)?;
    let out = game::card::CardData {
        name: card.name,
        card_type,
        cost,
        produces: None,
    };
    Ok(out)

}
