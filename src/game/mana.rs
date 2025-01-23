use std::fmt;

/// Represents an amount of mana somewhere.
/// In a cost, producer, or in the player's mana pool.
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ManaPool {
    pub white: u8,
    pub blue: u8,
    pub black: u8,
    pub red: u8,
    pub green: u8,
    pub generic: u8,
}

impl std::default::Default for ManaPool {
    fn default() -> Self {
        ManaPool::empty()
    }
}

impl ManaPool {
    pub fn empty() -> ManaPool {
        ManaPool {
            white: 0,
            blue: 0,
            black: 0,
            red: 0,
            green: 0,
            generic: 0,
        }

    }

    /// Parses a string representation of a mana pool and returns a `ManaPool` instance.
    ///
    /// # Examples
    /// ```
    /// use deck_optim::game::mana::ManaPool;
    ///
    /// let source = "2G";
    ///
    /// let actual_mana = ManaPool::try_parse(source).expect("should parse");
    /// let expected_mana = ManaPool {
    ///     generic: 2,
    ///     green: 1,
    ///     ..Default::default()
    /// };
    ///
    /// assert_eq!(expected_mana, actual_mana);
    /// ```
    pub fn try_parse(source: &str) -> Result<ManaPool, ManaPoolParseError> {
        let mut mana = ManaPool::empty();

        let mut chars = source.chars().peekable();

        // split it into the numeric portion and the non numeric portion
        let mut generic_portion = String::new();
        while let Some(ch) = chars.peek() {
            if ch.is_numeric() {
                generic_portion.push(*ch);
                chars.next();
            } else {
                break;
            }
        }

        if generic_portion.len() > 0 {
            let generic = match generic_portion.parse::<u8>() {
                Ok(num) => num,
                Err(err) => return Err(ManaPoolParseError {
                    source: source.to_string(),
                    why: format!("failed to parse generic cost of mana: {err}")
                })
            };
            mana.generic = generic;
        }

        for ch in chars {
            match ch {
                'W' => mana.white += 1,
                'U' => mana.blue += 1,
                'B' => mana.black += 1,
                'R' => mana.red += 1,
                'G' => mana.green += 1,
                _ => {
                    return Err(ManaPoolParseError {
                        source: source.to_string(),
                        why: format!("invalid type of mana '{ch}'")
                    });
                }
            }
        }

        Ok(mana)
    }
}

struct ManaPoolVisitor;
impl <'de> serde::de::Visitor<'de> for ManaPoolVisitor {
    type Value = ManaPool;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a mana cost")
    }

    fn visit_str<E>(self, value: &str) -> Result<ManaPool, E>
        where E: serde::de::Error
    {
        let mp = ManaPool::try_parse(&value)
            .map_err(|e| E::custom(e.why))?;

        Ok(mp)
    }
}
impl <'de> serde::Deserialize<'de> for ManaPool {
    fn deserialize<D>(deser: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        deser.deserialize_str(ManaPoolVisitor)
    }
}

#[derive(Debug)]
pub struct ManaPoolParseError {
    why: String,
#[allow(dead_code)]
    source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default<T: Default>() -> T {
        Default::default()
    }

    #[test]
    fn test_generic() {
        let source = "3";

        let actual_mana = ManaPool::try_parse(source).expect("should parse");
        let expected_mana = ManaPool {
            generic: 3,
            ..default()
        };

        assert_eq!(expected_mana, actual_mana);
    }

    #[test]
    fn test_empty() {
        let source = "";

        let actual_mana = ManaPool::try_parse(source).expect("should parse");
        let expected_mana = ManaPool {
            ..default()
        };

        assert_eq!(expected_mana, actual_mana);
    }

    #[test]
    fn test_colors() {
        let source = "WUBRG";

        let actual_mana = ManaPool::try_parse(source).expect("should parse");
        let expected_mana = ManaPool {
            white: 1,
            blue: 1,
            black: 1,
            red: 1,
            green: 1,
            ..default()
        };

        assert_eq!(expected_mana, actual_mana);
    }

    #[test]
    fn test_big_generic() {
        let source = "10RG";

        let actual_mana = ManaPool::try_parse(source).expect("should parse");
        let expected_mana = ManaPool {
            red: 1,
            green: 1,
            generic: 10,
            ..default()
        };

        assert_eq!(expected_mana, actual_mana);
    }
}
