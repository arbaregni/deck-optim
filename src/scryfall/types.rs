use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct CardData {
    pub name: String,
    pub type_line: String,
    pub mana_cost: String,
    pub oracle_text: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct CardCollectionResponse {
    pub data: Vec<CardData>,
    pub not_found: Vec<serde_json::Value>,
}
impl CardCollectionResponse {
    pub fn empty() -> Self {
        Self {
            data: Vec::new(),
            not_found: Vec::new(),
        }
    }
    pub fn extend(&mut self, other: Self) {
        self.data.extend(other.data);
        self.not_found.extend(other.not_found);
    }
}

#[derive(Debug, Serialize)]
#[allow(unused)]
pub struct CardCollectionRequest<'a> {
    pub identifiers: Vec<CardIdentifier<'a>>
}

#[derive(Debug, Serialize)]
#[allow(unused, non_camel_case_types)]
pub enum CardIdentifier<'a> {
    name(&'a str),
    id(&'a str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_identifier_deser() {

        let identifier = CardIdentifier::name("Ancient Tomb");
        let json = serde_json::to_string(&identifier).expect("success");

        assert_eq!(json, "{\"name\":\"Ancient Tomb\"}");
    }
}
