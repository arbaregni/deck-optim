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

    /// Try to remove a single pip from this mana pool.
    /// # Example
    /// ```
    /// use deck_optim::game::mana::ManaPool;
    /// use deck_optim::game::mana::ManaType;
    ///
    /// let pool = ManaPool::try_parse("{W}{W}{R}").expect("should parse");
    /// 
    /// let without_red = ManaPool::try_parse("{W}{W}").expect("should parse");
    /// assert_eq!(pool.try_remove_pip(ManaType::Red), Some(without_red));
    ///
    /// let remove_a_white = ManaPool::try_parse("{W}{R}").expect("should parse");
    /// assert_eq!(pool.try_remove_pip(ManaType::White), Some(remove_a_white));
    ///
    /// assert_eq!(pool.try_remove_pip(ManaType::Black), None)
    /// ```
    pub fn try_remove_pip(&self, mana_type: ManaType) -> Option<ManaPool> {
        use ManaType::*;
        let new = match mana_type {
            White => {
                let white = self.white.checked_sub(1)?;
                ManaPool { white, ..(*self) }
            }
            Blue => {
                let blue = self.blue.checked_sub(1)?;
                ManaPool { blue, ..(*self) }

            }
            Black => {
                let black = self.black.checked_sub(1)?;
                ManaPool { black , ..(*self) }

            }
            Red => {
                let red = self.red.checked_sub(1)?;
                ManaPool { red , ..(*self) }

            }
            Green => {
                let green = self.green.checked_sub(1)?;
                ManaPool { green , ..(*self) }

            }
            Colorless => {
                let colorless = self.colorless.checked_sub(1)?;
                ManaPool { colorless , ..(*self) }
            }
        };
        Some(new)
    }

    /// Iterate over all possible ways we can pay this cost given the available mana
    /// ```
    /// use deck_optim::game::mana::ManaPool;
    /// use deck_optim::game::mana::ManaCost;
    ///
    /// let available = ManaPool::try_parse("{W}{U}{G}").expect("should parse");
    /// let cost = ManaCost::try_parse("{1}{W}").expect("should parse");
    ///
    /// let mut ways_to_pay = available.payment_methods_for(cost);
    ///
    /// let opt1 = ManaPool::try_parse("{W}{U}").expect("should parse");
    /// let opt2 = ManaPool::try_parse("{W}{G}").expect("should parse");
    /// assert_eq!(ways_to_pay.next(), Some(opt1));
    /// assert_eq!(ways_to_pay.next(), Some(opt2));
    /// assert_eq!(ways_to_pay.next(), None);
    ///
    /// ```
    pub fn payment_methods_for(&self, cost: ManaCost) -> impl Iterator<Item = ManaPool> {

        fn __collect_payment_methods(solutions: &mut Vec<ManaPool>, remaining: ManaPool, partial_soln: ManaPool, generic_to_be_paid: u8) {
            if generic_to_be_paid == 0 {
                solutions.push(partial_soln);
                return;
            }
            for mana_type in ManaType::all().into_iter().copied() {
                let Some(new_remaining) = remaining.try_remove_pip(mana_type) else {
                    // unable to remove pay for the generic with this pip, carry on
                    continue;
                };
                let new_partial_soln = partial_soln + ManaPool::of(mana_type, 1);
                __collect_payment_methods(solutions, new_remaining, new_partial_soln, generic_to_be_paid - 1);
            }
        }

        let mut solutions = Vec::with_capacity(cost.generic as usize);

        if let Some(remaining) = *self - cost.colors {
            // we just paid off the colored portion of the cost
            let generic_to_be_paid = cost.generic;
            // now there are many ways for the generic portion to be paid
            __collect_payment_methods(&mut solutions, remaining, cost.colors, generic_to_be_paid);
        }

        solutions.into_iter()
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

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

}
