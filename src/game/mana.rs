mod pool;
pub use pool::*;

mod cost;
pub use cost::*;

mod r#type;
pub use r#type::*;

#[macro_use]
mod macros;

use regex::Regex;

#[derive(Debug, thiserror::Error)]
pub enum ManaParseError {
    #[error("not a valid type of mana `{bad_type}`")]
    InvalidManaType { bad_type: String },
    #[error("failure while parsing generic portion of mana")]
    FailedToParseGenericCost { source: std::num::ParseIntError },
    #[error("failed regex validation: `{bad_string}`. Must pass `{re}`")]
    DidNotMatchRegex { re: Regex, bad_string: String },
    #[error("can not use generic mana in a mana pool - only costs have generic portions. Did you mean colorless instead?")]
    GenericCostInManaPool,
}
