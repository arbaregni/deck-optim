use std::fmt;

use super::ManaParseError;
use super::ManaCost;
use super::ManaType;

/// Represents an amount of mana required by a cost
#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord)]
pub struct ManaPool {
    pub white: u8,
    pub blue: u8,
    pub black: u8,
    pub red: u8,
    pub green: u8,
    pub colorless: u8,
}


impl std::default::Default for ManaPool {
    fn default() -> Self {
        ManaPool::empty()
    }
}

impl ManaPool {
    // Creates an empty mana cost.
    // ```
    // use deck_optim::game::ManaPool;
    //
    // let cost = ManaPool::empty();
    // assert_eq!(cost.value(), 0);
    // ```
    pub fn empty() -> ManaPool {
        ManaPool {
            white: 0,
            blue: 0,
            black: 0,
            red: 0,
            green: 0,
            colorless: 0,
        }
    }

    pub fn try_from(cost: ManaCost) -> Result<ManaPool, ManaParseError> {
        if cost.generic > 0 {
            return Err(ManaParseError::GenericCostInManaPool);
        }
        Ok(cost.colors)
    }

    /// Creates a mana cost of white mana.
    pub fn white(white: u8) -> Self {
        Self {
            white,
            ..Default::default()
        }
    }
    /// Creates a mana cost of blue mana.
    pub fn blue(blue: u8) -> Self {
        Self {
            blue,
            ..Default::default()
        }
    }
    /// Creates a mana cost of black mana.
    pub fn black(black: u8) -> Self {
        Self {
            black,
            ..Default::default()
        }
    }
    /// Creates a mana cost of red mana.
    pub fn red(red: u8) -> Self {
        Self {
            red,
            ..Default::default()
        }
    }
    /// Creates a mana cost of green mana.
    pub fn green(green: u8) -> Self {
        Self {
            green,
            ..Default::default()
        }
    }
    /// Creates a mana cost of colorless mana.
    pub fn colorless(green: u8) -> Self {
        Self {
            green,
            ..Default::default()
        }
    }


    /// Creates a mana pool from a particular type of mana
    pub fn of(mana_type: ManaType, amount: u8) -> Self {
        use ManaType::*;
        match mana_type {
            White => Self::white(amount),
            Blue => Self::blue(amount),
            Black => Self::black(amount),
            Red => Self::red(amount),
            Green => Self::green(amount),
            Colorless => Self::colorless(amount),
        }
    }



    /// Parses a string representation of a mana cost and returns a `ManaPool` instance.
    ///
    /// # Examples
    /// ```
    /// use deck_optim::game::mana::ManaPool;
    ///
    /// let source = "{W}{G}{G}";
    ///
    /// let actual_mana = ManaPool::try_parse(source).expect("should parse");
    /// let expected_mana = ManaPool {
    ///     white: 1,
    ///     green: 2,
    ///     ..Default::default()
    /// };
    ///
    /// assert_eq!(expected_mana, actual_mana);
    /// ```
    pub fn try_parse(source: &str) -> Result<ManaPool, ManaParseError> {
        let mana_cost = ManaCost::try_parse(source)?;
        let mana_pool = ManaPool::try_from(mana_cost)?;
        Ok(mana_pool)
    }

    pub fn mana_value(&self) -> u8 {
        let ManaPool { white, blue, black, red, green, colorless } = self;
        [white, blue, black, red, green, colorless].into_iter().sum()
    }

    /// Remove a single pip from this mana pool.
    pub fn remove_pip(&self, mana_type: ManaType) -> ManaPool {
        let mut new = self.clone();
        new[mana_type] -= 1;
        new
    }
    /// Adds a single pip into this mana pool.
    pub fn add_pip(&self, mana_type: ManaType) -> ManaPool {
        let mut new = self.clone();
        new[mana_type] += 1;
        new
    }

    /// Iterate over all types of mana present in this mana pool.
    /// # Example
    /// ```
    /// use deck_optim::game::mana::ManaPool;
    /// use deck_optim::game::mana::ManaType;
    ///
    /// let available = ManaPool::try_parse("{W}{U}").expect("should parse");
    /// let mut mana_types = available.mana_types();
    ///
    /// assert_eq!(mana_types.next(), Some(ManaType::White));
    /// assert_eq!(mana_types.next(), Some(ManaType::Blue));
    /// assert_eq!(mana_types.next(), None);
    /// ```
    pub fn mana_types(&self) -> impl Iterator<Item = ManaType> + use<'_> {
        ManaType::all()
            .into_iter()
            .copied()
            .filter(|mt| self[*mt] > 0)
    }
}

impl std::ops::Index<ManaType> for ManaPool {
    type Output = u8;

    fn index(&self, t: ManaType) -> &Self::Output {
        use ManaType::*;
        match t {
            White => &self.white,
            Blue => &self.blue,
            Black => &self.black,
            Red => &self.red,
            Green => &self.green,
            Colorless => &self.colorless,
        }
    }
}
impl std::ops::IndexMut<ManaType> for ManaPool {
    fn index_mut(&mut self, index: ManaType) -> &mut Self::Output {
        use ManaType::*;
        match index {
            White => &mut self.white,
            Blue => &mut self.blue,
            Black => &mut self.black,
            Red => &mut self.red,
            Green => &mut self.green,
            Colorless => &mut self.colorless,
        }
    }
}

impl std::ops::Add for ManaPool {
    type Output = ManaPool;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            white:   self.white + rhs.white,
            blue:    self.blue  + rhs.blue,
            black:   self.black + rhs.black,
            red:     self.red   + rhs.red,
            green:   self.green + rhs.green,
            colorless: self.colorless + rhs.colorless,
        }
    }
}

impl std::ops::Sub for ManaPool {
    type Output = Option<ManaPool>;

    fn sub(self, rhs: Self) -> Self::Output {
        let white = self.white.checked_sub(rhs.white)?;
        let blue = self.blue.checked_sub(rhs.blue)?;
        let black = self.black.checked_sub(rhs.black)?;
        let red = self.red.checked_sub(rhs.red)?;
        let green = self.green.checked_sub(rhs.green)?;
        let colorless = self.colorless.checked_sub(rhs.colorless)?;
        Some(ManaPool {
            white, blue, black, red, green, colorless
        })
    }
}

crate::impl_serde_traits! {
    ManaPool {
        serialize => ManaPool::try_parse,
        deserialize => ManaPool::to_string
    }
}

impl std::iter::Sum for ManaPool {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut total = ManaPool::empty();
        for mp in iter {
            total = total + mp;
        }
        total
    }
}

impl fmt::Display for ManaPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ManaPool { white, blue, black, red, green, colorless } = *self;
        
        for _ in 0..white     { write!(f, "{{W}}")?; }
        for _ in 0..blue      { write!(f, "{{U}}")?; }
        for _ in 0..black     { write!(f, "{{B}}")?; }
        for _ in 0..red       { write!(f, "{{R}}")?; }
        for _ in 0..green     { write!(f, "{{G}}")?; }
        for _ in 0..colorless { write!(f, "{{C}}")?; }

        if self.mana_value() == 0 {
            write!(f, "{{0}}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    fn default<T: Default>() -> T {
        Default::default()
    }

    #[test]
    fn test_parse_generic() {
        let source = "{3}";

        let err = ManaPool::try_parse(source).expect_err("should not parse");

        assert!(matches!(err, ManaParseError::GenericCostInManaPool));
    }

    #[test]
    fn test_parse_empty() {
        let source = "";

        let actual_mana = ManaPool::try_parse(source).expect("should parse");
        let expected_mana = ManaPool::empty();

        assert_eq!(expected_mana, actual_mana);
    }

    #[test]
    fn test_parse_zero() {
        let source = "{0}";

        let actual_mana = ManaPool::try_parse(source).expect("should parse");
        let expected_mana = ManaPool::empty();
        assert_eq!(expected_mana, actual_mana);
    }

    #[test]
    fn test_parse_colors() {
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
    fn test_parse_fail_regex() {
        let source = "{1";

        let result = ManaPool::try_parse(source);
        assert!(result.is_err())
    }

    #[test]
    fn test_parse_fail_to_parse_mana_symbol() {
        let source = "{*^%$}";

        let result = ManaPool::try_parse(source);
        assert!(result.is_err())
    }

    #[test]
    fn test_parse_fail_to_parse_big_number() {
        let source = "{999999}";

        let result = ManaPool::try_parse(source);
        assert!(result.is_err())
    }


    #[test]
    fn test_add_simple() {
        let lhs = ManaPool::try_parse("{W}{W}{U}").expect("should parse");
        let rhs = ManaPool::try_parse("{W}{G}").expect("should parse");

        let actual = lhs + rhs;
        let expected = ManaPool::try_parse("{W}{W}{W}{U}{G}").expect("should parse");

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_sub_simple() {
        let lhs = ManaPool::try_parse("{W}{W}{U}").expect("should parse");
        let rhs = ManaPool::try_parse("{W}").expect("should parse");

        let actual = lhs - rhs;
        let expected = ManaPool::try_parse("{W}{U}").expect("should parse");

        assert_eq!(actual, Some(expected));
    }

    #[test]
    fn test_sub_impossible() {
        let lhs = ManaPool::try_parse("{W}{W}{U}").expect("should parse");
        let rhs = ManaPool::try_parse("{B}").expect("should parse");

        let actual = lhs - rhs;

        assert_eq!(actual, None);
    }

    #[test]
    fn test_sub_colorless() {
        let available = ManaPool::try_parse("{R}{R}{R}{G}{G}{C}{C}").expect("should parse");
        let payment = ManaPool::try_parse("{R}{R}{C}{C}{G}{G}").expect("should parse");

        let remaining = available - payment;
        let expected_remaining = Some(ManaPool::try_parse("{R}").expect("should parse"));

        assert_eq!(remaining, expected_remaining);
    }


}
