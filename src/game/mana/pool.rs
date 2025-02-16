use std::fmt;

use itertools::Itertools;

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
    fn remove_pip(&self, mana_type: ManaType) -> ManaPool {
        let mut new = self.clone();
        new[mana_type] -= 1;
        new
    }
    /// Adds a single pip into this mana pool.
    fn add_pip(&self, mana_type: ManaType) -> ManaPool {
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

    /// Iterate over all the ways we can pay for a certain amount of generic mana with this mana pool.
    /// # Example
    /// ```
    /// use deck_optim::game::mana::ManaPool;
    ///
    /// let available = ManaPool::try_parse("{W}{U}").expect("should parse");
    /// let mut payment_methods = available.payment_methods_for_generic(1);
    ///
    /// assert_eq!(payment_methods.next(), Some(ManaPool::blue(1)));
    /// assert_eq!(payment_methods.next(), Some(ManaPool::white(1)));
    /// assert_eq!(payment_methods.next(), None);
    /// ```
    pub fn payment_methods_for_generic(&self, generic: u8) -> impl Iterator<Item = ManaPool> {
        if generic == 0 {
            // there is a single way to pay for {0}
            return vec![ManaPool::empty()].into_iter();
        }
        let mut solutions = vec![];
        // for each type of mana that we have, we could take that pip in order to pay for {1} generic
        for mt in self.mana_types() {
            // remove the pip
            let next = self.remove_pip(mt);
            let next_solutions = next.payment_methods_for_generic(generic - 1)
                .map(|soln| soln.add_pip(mt));
            solutions.extend(next_solutions);
        }

        solutions.sort();
        solutions.dedup();
        solutions.into_iter()
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
    /// let opt1 = ManaPool::try_parse("{W}{G}").expect("should parse");
    /// let opt2 = ManaPool::try_parse("{W}{U}").expect("should parse");
    /// assert_eq!(ways_to_pay.next(), Some(opt1));
    /// assert_eq!(ways_to_pay.next(), Some(opt2));
    /// assert_eq!(ways_to_pay.next(), None);
    ///
    /// ```
    pub fn payment_methods_for(&self, cost: ManaCost) -> impl Iterator<Item = ManaPool> {
        let ManaCost { colors, generic } = cost;

        // first, pay off the colored portion
        let Some(remaining) = *self - colors else {
            // unable to pay because of the colored mana requirements
            return vec![].into_iter();
        };

        // now, the question is: how many ways can the generic portion be payed off?
        let solutions = remaining.payment_methods_for_generic(generic)
            .map(|payment| payment + colors)
            .collect_vec();

        // coerce the types so it's the same as above
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

    #[test]
    fn test_payment_methods_for_generic_zero() {
        let available = ManaPool::try_parse("{R}").expect("should parse");

        let mut payment_methods = available.payment_methods_for_generic(0);

        assert_eq!(payment_methods.next(), Some(ManaPool::empty()));
        assert_eq!(payment_methods.next(), None);
    }
    
    #[test]
    fn test_payment_methods_for_generic_zero_from_zero() {
        let available = ManaPool::try_parse("{0}").expect("should parse");

        let mut payment_methods = available.payment_methods_for_generic(0);

        assert_eq!(payment_methods.next(), Some(ManaPool::empty()));
        assert_eq!(payment_methods.next(), None);
    }

    #[test]
    fn test_payment_methods_for_generic_single() {
        let available = ManaPool::try_parse("{R}").expect("should parse");

        let mut payment_methods = available.payment_methods_for_generic(1);

        assert_eq!(payment_methods.next(), Some(ManaPool::try_parse("{R}").expect("should parse")));
        assert_eq!(payment_methods.next(), None);
    }

    #[test]
    fn test_payment_methods_for_generic_single_multiple_colors() {
        let available = ManaPool::try_parse("{R}{B}").expect("should parse");

        let mut payment_methods = available.payment_methods_for_generic(1);

        assert_eq!(payment_methods.next(), Some(ManaPool::try_parse("{R}").expect("should parse")));
        assert_eq!(payment_methods.next(), Some(ManaPool::try_parse("{B}").expect("should parse")));
        assert_eq!(payment_methods.next(), None);
    }

    #[test]
    fn test_payment_methods_for_generic_single_duplicate_colors() {
        let available = ManaPool::try_parse("{R}{R}").expect("should parse");

        let mut payment_methods = available.payment_methods_for_generic(1);

        assert_eq!(payment_methods.next(), Some(ManaPool::try_parse("{R}").expect("should parse")));
        assert_eq!(payment_methods.next(), None);
    }

    #[test]
    fn test_payment_methods_for_generic_single_impossible() {
        let available = ManaPool::try_parse("{0}").expect("should parse");

        let mut payment_methods = available.payment_methods_for_generic(1);

        assert_eq!(payment_methods.next(), None);
    }

    #[test]
    fn test_payment_methods_for_generic_pay_two_of_the_same() {
        let available = ManaPool::try_parse("{U}{U}").expect("should parse");

        let mut payment_methods = available.payment_methods_for_generic(2);

        assert_eq!(payment_methods.next(), Some(ManaPool::try_parse("{U}{U}").expect("should parse")));
        assert_eq!(payment_methods.next(), None);
    }
 
    #[test]
    fn test_payment_methods_for_generic_pay_two_extra_mana() {
        let available = ManaPool::try_parse("{U}{U}{B}{G}").expect("should parse");

        let mut payment_methods = available.payment_methods_for_generic(2);

        assert_eq!(payment_methods.next(), Some(ManaPool::try_parse("{B}{G}").expect("should parse")));
        assert_eq!(payment_methods.next(), Some(ManaPool::try_parse("{U}{G}").expect("should parse")));
        assert_eq!(payment_methods.next(), Some(ManaPool::try_parse("{U}{B}").expect("should parse")));
        assert_eq!(payment_methods.next(), Some(ManaPool::try_parse("{U}{U}").expect("should parse")));
        assert_eq!(payment_methods.next(), None);
    }

    #[test]
    fn test_payment_methods_for_generic_pay_two_impossible() {
        let available = ManaPool::try_parse("{U}").expect("should parse");

        let mut payment_methods = available.payment_methods_for_generic(2);

        assert_eq!(payment_methods.next(), None);
    }
           
                                    

   #[test]
    fn test_payment_method_single_possibility() {
        let available = ManaPool::try_parse("{R}{R}{G}{G}").expect("should parse");
        let cost = ManaCost::try_parse("{2}{G}{G}").expect("should parse");

        let mut payment_methods = available.payment_methods_for(cost);

        assert_eq!(payment_methods.next(), Some(ManaPool::try_parse("{R}{R}{G}{G}").expect("should parse")));
        assert_eq!(payment_methods.next(), None);
    }

    #[test]
    fn test_payment_method_multiple_answers() {
        let available = ManaPool::try_parse("{W}{U}{B}{G}{G}").expect("should parse");
        let cost = ManaCost::try_parse("{2}{G}{G}").expect("should parse");

        let payment_methods = available.payment_methods_for(cost);
        let mut actual_answers: Vec<_> = payment_methods.collect();

        let mut expected_answers = vec![
            ManaPool::try_parse("{W}{U}{G}{G}").expect("should parse"),
            ManaPool::try_parse("{W}{B}{G}{G}").expect("should parse"),
            ManaPool::try_parse("{U}{B}{G}{G}").expect("should parse"),
        ];

        actual_answers.sort();
        expected_answers.sort();
        assert_eq!(actual_answers, expected_answers);
    }


    #[test]
    fn test_payment_method_generic() {
        let available = ManaPool::try_parse("{R}{R}{R}{G}{C}{C}").expect("should parse");
        let cost = ManaCost::try_parse("{4}").expect("should parse");

        let payment_methods = available.payment_methods_for(cost);
        let mut actual_answers: Vec<_> = payment_methods.collect();

        let mut expected_answers = vec![
            ManaPool::try_parse("{R}{R}{R}{G}").expect("should parse"),
            ManaPool::try_parse("{R}{R}{R}{C}").expect("should parse"),
            ManaPool::try_parse("{R}{R}{C}{C}").expect("should parse"),
            ManaPool::try_parse("{R}{R}{C}{G}").expect("should parse"),
            ManaPool::try_parse("{R}{C}{C}{G}").expect("should parse"),
        ];

        actual_answers.sort();
        expected_answers.sort();
        assert_eq!(actual_answers, expected_answers);
    } 

}
