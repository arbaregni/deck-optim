use itertools::Itertools;

use crate::game;
use crate::scryfall::types;

/// Error in translating scryfall's card data into our model
#[derive(thiserror::Error, Debug)]
pub enum ConversionError {
    #[error("could not convert type line: `{card_type}` is unknown")]
    UnknownCardType { card_type: String },
    #[error("could not parse mana cost: `{mana_cost}` due to `{source}`")]
    CannotParseManaCost { mana_cost: String, source: game::ManaPoolParseError },
    #[error("too many separators found in type line")]
    TooManySeparators {},
    #[error("this combination of card types is not supported yet")]
    UnsupportedCardTypeCombination { card_types: Vec<game::CardType> }
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

const TYPE_LINE_SEPARATOR: &'static str = "—";

pub struct CardTypes {
    #[allow(dead_code)]
    super_types: Vec<game::SuperType>,
    card_types: Vec<game::CardType>
}

pub fn convert_card_types(types: &str) -> Result<CardTypes, ConversionError> {
    use game::CardType::*;
    use game::SuperType::*;

    let mut super_types = Vec::new();
    let mut card_types = Vec::new();

    for card_type in types.split_whitespace() {

        match card_type.to_lowercase().as_str() {
            "land"      => card_types.push(Land),
            "instant"   => card_types.push(Instant),
            "sorcery"   => card_types.push(Sorcery),
            "creature"  => card_types.push(Creature),

            "basic"     => super_types.push(Basic),
            "snow"      => super_types.push(Snow),
            "legendary" => super_types.push(Legendary),
            "world"     => super_types.push(World),

            _ => return Err(ConversionError::UnknownCardType { card_type: card_type.to_string() })

        }
    
    }
    Ok(CardTypes { super_types, card_types })
}

pub fn convert_type_line(type_line: String) -> Result<game::CardType, ConversionError> {
    let ct = match type_line.split(TYPE_LINE_SEPARATOR).collect_vec()[..] {
        [] => return Err(ConversionError::TooManySeparators { }),
        [card_types] => {
            convert_card_types(card_types)?
        }
        [card_types, _subtypes] => {
            convert_card_types(card_types)?
        }
        _ => return Err(ConversionError::TooManySeparators { }),   
    };

    match ct.card_types.as_slice() {
        [card_type] => Ok(card_type.clone()),
        _ => Err(ConversionError::UnsupportedCardTypeCombination { card_types: ct.card_types })
    }
}

/// Converts a scryfall card into a game::card::CardData
pub fn convert_card(card: types::CardData) -> Result<game::CardData, ConversionError> {
    let card_type = convert_type_line(card.type_line)?;
    let cost = convert_mana_cost(card.mana_cost)?;
    let out = game::card::CardData {
        name: card.name,
        card_type,
        cost,
    };
    Ok(out)

}

#[cfg(test)]
mod tests {
    use game::ManaPool;

    use super::*;

    #[test]
    fn test_convert_empty_mana_cost() {
        let source = "";
        let mana_cost = convert_mana_cost(source.to_string()).expect("no errors");
        assert_eq!(None, mana_cost);
    }

    #[test]
    fn test_convert_zero_mana_cost() {
        let source = "{0}";
        let mana_cost = convert_mana_cost(source.to_string()).expect("no errors");
        let expected = ManaPool::empty();
        assert_eq!(Some(expected), mana_cost);
    }

    #[test]
    fn test_convert_mana_cost() {
        let source = "{3}{R}{G}";
        let mana_cost = convert_mana_cost(source.to_string()).expect("no errors");
        let expected = ManaPool { 
            red: 1,
            green: 1,
            generic: 3,
            ..Default::default()
        };
        assert_eq!(Some(expected), mana_cost);
    }




    #[test]
    fn test_convert_instant() {
        let source = "Instant";
        let card_type = convert_card_types(source).expect("no errors");
        let expected = game::CardType::Instant;
        assert_eq!(vec![expected], card_type.card_types);
        assert_eq!(0, card_type.super_types.len());
    }


    #[test]
    fn test_convert_land() {
        let source = "Land";
        let card_type = convert_card_types(source).expect("no errors");
        let expected = game::CardType::Land;
        assert_eq!(vec![expected], card_type.card_types);
        assert_eq!(0, card_type.super_types.len());
    }

    #[test]
    fn test_convert_basic_land() {
        let source = "Basic Land";
        let card_type = convert_card_types(source).expect("no errors");
        assert_eq!(vec![game::CardType::Land], card_type.card_types);
        assert_eq!(vec![game::SuperType::Basic], card_type.super_types);
    }


    #[test]
    fn test_convert_mountain_typeline() {
        let source = "Basic Land — Mountain".to_string();
        let card_type = convert_type_line(source).expect("no errors");
        assert_eq!(game::CardType::Land, card_type);
    }
}
