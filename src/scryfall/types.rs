use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct CardData {
    pub name: String,
    pub type_line: String,
    pub mana_cost: String,
    pub oracle_text: String,
}
