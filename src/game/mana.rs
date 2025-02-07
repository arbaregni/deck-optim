use std::fmt;

use regex::Regex;

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
    /// let source = "{2}{G}";
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

        let overall = Regex::new(r"^(\{[WUBRG0-9]+\})*$").expect("regex to compile");
        if !overall.is_match(source) {
            return Err(ManaPoolParseError::DidNotMatchRegex {
                re: overall,
                bad_string: source.to_string()
            })
        }
        let re = Regex::new(r"\{([WUBRG0-9]+)\}").expect("regex to compile");

        let mut mana = ManaPool::empty();

        for cap in re.captures_iter(source) {
            let mat = &cap[1]; // capture group contents
            match mat {
                "W" => mana.white += 1,
                "U" => mana.blue += 1,
                "B" => mana.black += 1,
                "R" => mana.red += 1,
                "G" => mana.green += 1,
                digits if digits.chars().all(|ch| ch.is_ascii_digit()) => {
                    mana.generic += match digits.parse::<u8>() {
                        Ok(num) => num,
                        Err(source) => return Err(ManaPoolParseError::FailedToParseGenericCost {
                            source
                        })
                    };
                }
                bad_type => {
                    return Err(ManaPoolParseError::InvalidManaType {
                        bad_type: bad_type.to_string()
                    });
                }

            }
        }

        Ok(mana)
    }

    pub fn mana_value(&self) -> u8 {
        let ManaPool { white, blue, black, red, green, generic } = self;
        [white, blue, black, red, green, generic].into_iter().sum()
    }

    pub fn less_than(&self, other: &ManaPool) -> bool {
        let ManaPool { white, blue, black, red, green, generic } = *self;
        white <= other.white
            && blue <= other.blue
            && black <= other.black
            && red <= other.red
            && green <= other.green
            && generic <= other.generic
    }
    pub fn try_subtract(&self, other: &ManaPool) -> Option<ManaPool> {
        let white = self.white.checked_sub(other.white)?;
        let blue = self.blue.checked_sub(other.blue)?;
        let black = self.black.checked_sub(other.black)?;
        let red = self.red.checked_sub(other.red)?;
        let green = self.green.checked_sub(other.green)?;
        let generic = self.generic.checked_sub(other.generic)?;

        Some(ManaPool {
            white,
            blue,
            black,
            red,
            green,
            generic,
        })

    }
}

impl std::ops::Add for ManaPool {
    type Output = ManaPool;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            white:   self.white     + rhs.white,
            blue:    self.blue      + rhs.blue,
            black:   self.black     + rhs.black,
            red:     self.red       + rhs.red,
            green:   self.green     + rhs.green,
            generic: self.generic   + rhs.generic,
        }
    }
}

impl std::iter::Sum for ManaPool {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut total = ManaPool::empty();
        for mp in iter {
            total = total + mp;
        }
        total
    } }

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
            .map_err(|e| E::custom(e))?;

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
impl serde::Serialize for ManaPool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        let s = format!("{}", self);
        serializer.serialize_str(&s)
    }
}

impl fmt::Display for ManaPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ManaPool { white, blue, black, red, green, generic } = *self;
        
        write!(f, "{{{generic}}}")?;
        for _ in 0..white   { write!(f, "{{W}}")?; }
        for _ in 0..blue    { write!(f, "{{U}}")?; }
        for _ in 0..black   { write!(f, "{{B}}")?; }
        for _ in 0..red     { write!(f, "{{R}}")?; }
        for _ in 0..green   { write!(f, "{{G}}")?; }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ManaPoolParseError {
    #[error("not a valid type of mana `{bad_type}`")]
    InvalidManaType { bad_type: String },
    #[error("failure while parsing generic portion of mana")]
    FailedToParseGenericCost { source: std::num::ParseIntError },
    #[error("failed regex validation: `{bad_string}`. Must pass `{re}`")]
    DidNotMatchRegex { re: Regex, bad_string: String },
}


#[cfg(test)]
mod tests {
    use super::*;

    fn default<T: Default>() -> T {
        Default::default()
    }

    #[test]
    fn test_generic() {
        let source = "{3}";

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
    fn test_zero() {
        let source = "{0}";

        let actual_mana = ManaPool::try_parse(source).expect("should parse");
        let expected_mana = ManaPool {
            ..default()
        };

        assert_eq!(expected_mana, actual_mana);
    }

    #[test]
    fn test_colors() {
        let source = "{W}{U}{B}{R}{G}";

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
        let source = "{10}{R}{G}";

        let actual_mana = ManaPool::try_parse(source).expect("should parse");
        let expected_mana = ManaPool {
            red: 1,
            green: 1,
            generic: 10,
            ..default()
        };

        assert_eq!(expected_mana, actual_mana);
    }

    #[test]
    fn test_fail_regex() {
        let source = "{1";

        let result = ManaPool::try_parse(source);
        assert!(result.is_err())
    }

    #[test]
    fn test_fail_to_parse_mana_symbol() {
        let source = "{*^%$}";

        let result = ManaPool::try_parse(source);
        assert!(result.is_err())
    }

    #[test]
    fn test_fail_to_parse_big_number() {
        let source = "{999999}";

        let result = ManaPool::try_parse(source);
        assert!(result.is_err())
    }




    #[test]
    fn test_serialize_big_generic() {
        let mana = ManaPool {
            white: 3,
            blue: 2,
            black: 1,
            generic: 10,
            ..default()
        };

        let actual = format!("{}", mana);
        let expected = "{10}{W}{W}{W}{U}{U}{B}";

        assert_eq!(actual, expected);
    }



    
    #[test]
    fn test_try_subtract_success() {
        let pool1 = ManaPool { white: 5, blue: 3, black: 2, red: 4, green: 1, generic: 6 };
        let pool2 = ManaPool { white: 2, blue: 1, black: 1, red: 2, green: 0, generic: 4 };
        
        let result = pool1.try_subtract(&pool2);
        assert_eq!(
            result,
            Some(ManaPool { white: 3, blue: 2, black: 1, red: 2, green: 1, generic: 2 })
        );
    }

    #[test]
    fn test_try_subtract_failure() {
        let pool1 = ManaPool { white: 1, blue: 1, black: 1, red: 1, green: 1, generic: 1 };
        let pool2 = ManaPool { white: 2, blue: 1, black: 1, red: 1, green: 1, generic: 1 };
        
        let result = pool1.try_subtract(&pool2);
        assert_eq!(result, None);
    }

    #[test]
    fn test_try_subtract_exact_match() {
        let pool1 = ManaPool { white: 2, blue: 2, black: 2, red: 2, green: 2, generic: 2 };
        let pool2 = ManaPool { white: 2, blue: 2, black: 2, red: 2, green: 2, generic: 2 };
        
        let result = pool1.try_subtract(&pool2);
        assert_eq!(
            result,
            Some(ManaPool { white: 0, blue: 0, black: 0, red: 0, green: 0, generic: 0 })
        );
    }

    #[test]
    fn test_try_subtract_zero_case() {
        let pool1 = ManaPool { white: 0, blue: 0, black: 0, red: 0, green: 0, generic: 0 };
        let pool2 = ManaPool { white: 0, blue: 0, black: 0, red: 0, green: 0, generic: 0 };

        let result = pool1.try_subtract(&pool2);
        assert_eq!(
            result,
            Some(ManaPool { white: 0, blue: 0, black: 0, red: 0, green: 0, generic: 0 })
        );
    }
}
