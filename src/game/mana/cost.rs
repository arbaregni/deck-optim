use std::fmt;

use regex::Regex;

use super::ManaParseError;
use super::ManaPool;

/// Represents an amount of mana required by a cost
#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord)]
pub struct ManaCost {
    pub colors: ManaPool, 
    pub generic: u8,
}


impl std::default::Default for ManaCost {
    fn default() -> Self {
        ManaCost::empty()
    }
}

impl ManaCost {
    // Creates an empty mana pool.
    // ```
    // use deck_optim::game::ManaCost;
    //
    // let pool = ManaCost::empty();
    // assert_eq!(pool.value(), 0);
    // ```
    pub fn empty() -> ManaCost {
        Self {
            colors: ManaPool::empty(),
            generic: 0,
        }
    }

    // Creates a mana cost of entirely white mana
    pub fn white(white: u8) -> Self {
        Self {
            colors: ManaPool::white(white),
            generic: 0
        }
    }
    // Creates a mana cost of entirely blue mana
    pub fn blue(blue: u8) -> Self {
        Self {
            colors: ManaPool::blue(blue),
            generic: 0
        }
    }
    // Creates a mana cost of entirely black mana
    pub fn black(black: u8) -> Self {
        Self {
            colors: ManaPool::black(black),
            generic: 0
        }
    }
    // Creates a mana cost of entirely red mana
    pub fn red(red: u8) -> Self {
        Self {
            colors: ManaPool::red(red),
            generic: 0
        }
    }
    // Creates a mana cost of entirely green mana
    pub fn green(green: u8) -> Self {
        Self {
            colors: ManaPool::green(green),
            generic: 0
        }
    }
    // Creates a mana cost of entirely generic
    pub fn generic(generic: u8) -> Self {
        Self {
            colors: ManaPool::empty(),
            generic
        }
    }
    
    /// Parses a string representation of a mana pool and returns a `ManaCost` instance.
    ///
    /// # Examples
    /// ```
    /// use deck_optim::game::mana::ManaCost;
    /// use deck_optim::game::mana::ManaPool;
    ///
    /// let source = "{2}{G}";
    ///
    /// let actual_mana = ManaCost::try_parse(source).expect("should parse");
    /// let expected_mana = ManaCost {
    ///     colors: ManaPool::green(1),
    ///     generic: 2
    /// };
    ///
    /// assert_eq!(expected_mana, actual_mana);
    /// ```
    pub fn try_parse(source: &str) -> Result<ManaCost, ManaParseError> {

        let overall = Regex::new(r"^(\{[WUBRGC0-9]+\})*$").expect("regex to compile");
        if !overall.is_match(source) {
            return Err(ManaParseError::DidNotMatchRegex {
                re: overall,
                bad_string: source.to_string()
            })
        }
        let re = Regex::new(r"\{([WUBRGC0-9]+)\}").expect("regex to compile");

        let mut mana = ManaCost::empty();

        for cap in re.captures_iter(source) {
            let mat = &cap[1]; // capture group contents
            match mat {
                "W" => mana.colors.white     += 1,
                "U" => mana.colors.blue      += 1,
                "B" => mana.colors.black     += 1,
                "R" => mana.colors.red       += 1,
                "G" => mana.colors.green     += 1,
                "C" => mana.colors.colorless += 1,
                digits if digits.chars().all(|ch| ch.is_ascii_digit()) => {
                    mana.generic += match digits.parse::<u8>() {
                        Ok(num) => num,
                        Err(source) => return Err(ManaParseError::FailedToParseGenericCost {
                            source
                        })
                    };
                }
                bad_type => {
                    return Err(ManaParseError::InvalidManaType {
                        bad_type: bad_type.to_string()
                    });
                }

            }
        }

        Ok(mana)
    }

    pub fn mana_value(&self) -> u8 {
        self.colors.mana_value() + self.generic
    }

}

impl std::ops::Add for ManaCost {
    type Output = ManaCost;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            colors:  self.colors    + rhs.colors,
            generic: self.generic   + rhs.generic,
        }
    }
}

impl std::iter::Sum for ManaCost {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut total = ManaCost::empty();
        for mp in iter {
            total = total + mp;
        }
        total
    }
}


impl fmt::Display for ManaCost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        
        let generic = self.generic;
        if generic > 0 { write!(f, "{{{generic}}}")?; }
        for _ in 0..self.colors.white     { write!(f, "{{W}}")?; }
        for _ in 0..self.colors.blue      { write!(f, "{{U}}")?; }
        for _ in 0..self.colors.black     { write!(f, "{{B}}")?; }
        for _ in 0..self.colors.red       { write!(f, "{{R}}")?; }
        for _ in 0..self.colors.green     { write!(f, "{{G}}")?; }
        for _ in 0..self.colors.colorless { write!(f, "{{C}}")?; }

        if self.mana_value() ==  0 {
            write!(f, "{{0}}")?;
        }

        Ok(())
    }
}

crate::impl_serde_traits! {
    ManaCost {
        serialize => ManaCost::try_parse,
        deserialize => ManaCost::to_string
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use super::super::ManaPool;


    fn default<T: Default>() -> T {
        Default::default()
    }

    #[test]
    fn test_parse_generic() {
        let source = "{3}";

        let actual_mana = ManaCost::try_parse(source).expect("should parse");
        let expected_mana = ManaCost::generic(3);

        assert_eq!(expected_mana, actual_mana);
    }

    #[test]
    fn test_parse_empty() {
        let source = "";

        let actual_mana = ManaCost::try_parse(source).expect("should parse");
        let expected_mana = ManaCost::empty();

        assert_eq!(expected_mana, actual_mana);
    }

    #[test]
    fn test_parse_zero() {
        let source = "{0}";

        let actual_mana = ManaCost::try_parse(source).expect("should parse");
        let expected_mana = ManaCost::empty();
        assert_eq!(expected_mana, actual_mana);
    }

    #[test]
    fn test_parse_colors() {
        let source = "{W}{U}{B}{R}{G}";

        let actual_mana = ManaCost::try_parse(source).expect("should parse");
        let expected_mana = ManaCost {
            colors: ManaPool {
                white: 1,
                blue: 1,
                black: 1,
                red: 1,
                green: 1,
                ..default()
            },
            ..default()
        };

        assert_eq!(expected_mana, actual_mana);
    }

    #[test]
    fn test_parse_big_generic() {
        let source = "{10}{R}{G}";

        let actual_mana = ManaCost::try_parse(source).expect("should parse");
        let expected_mana = ManaCost {
            colors: ManaPool {
                red: 1,
                green: 1,
                ..default()
            },
            generic: 10,
        };

        assert_eq!(expected_mana, actual_mana);
    }

    #[test]
    fn test_parse_fail_regex() {
        let source = "{1";

        let result = ManaCost::try_parse(source);
        assert!(result.is_err())
    }

    #[test]
    fn test_parse_fail_to_parse_mana_symbol() {
        let source = "{*^%$}";

        let result = ManaCost::try_parse(source);
        assert!(result.is_err())
    }

    #[test]
    fn test_parse_fail_to_parse_big_number() {
        let source = "{999999}";

        let result = ManaCost::try_parse(source);
        assert!(result.is_err())
    }


    #[test]
    fn test_serialize_big_generic() {
        let mana = ManaCost {
            colors: ManaPool {
                white: 3,
                blue: 2,
                black: 1,
                ..default()
            },
            generic: 10,
        };

        let actual = format!("{}", mana);
        let expected = "{10}{W}{W}{W}{U}{U}{B}";

        assert_eq!(actual, expected);
    }
}
