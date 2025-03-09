use itertools::Itertools;

use crate::game::mana::ManaPool;
use crate::game::mana::ManaCost;

/// Iterate over all the ways we can pay for a certain amount of generic mana with this mana pool.
/// # Example
/// ```
/// use deck_optim::game::mana::ManaPool;
/// use deck_optim::strategies::payment_solver;
///
/// let available = ManaPool::try_parse("{W}{U}").expect("should parse");
/// let mut payment_methods = payment_solver::payment_methods_for_generic(&available, 1);
///
/// assert_eq!(payment_methods.next(), Some(ManaPool::blue(1)));
/// assert_eq!(payment_methods.next(), Some(ManaPool::white(1)));
/// assert_eq!(payment_methods.next(), None);
/// ```
pub fn payment_methods_for_generic(available: &ManaPool, generic: u8) -> impl Iterator<Item = ManaPool> {
    if generic == 0 {
        // there is a single way to pay for {0}
        return vec![ManaPool::empty()].into_iter();
    }
    let mut solutions = vec![];
    // for each type of mana that we have, we could take that pip in order to pay for {1} generic
    for mt in available.mana_types() {
        // remove the pip
        let next = available.remove_pip(mt);
        let next_solutions = payment_methods_for_generic(&next, generic - 1)
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
/// use deck_optim::strategies::payment_solver;
///
/// let available = ManaPool::try_parse("{W}{U}{G}").expect("should parse");
/// let cost = ManaCost::try_parse("{1}{W}").expect("should parse");
///
/// let mut ways_to_pay = payment_solver::payment_methods_for(&available, &cost);
///
/// let opt1 = ManaPool::try_parse("{W}{G}").expect("should parse");
/// let opt2 = ManaPool::try_parse("{W}{U}").expect("should parse");
/// assert_eq!(ways_to_pay.next(), Some(opt1));
/// assert_eq!(ways_to_pay.next(), Some(opt2));
/// assert_eq!(ways_to_pay.next(), None);
///
/// ```
pub fn payment_methods_for(available: &ManaPool, cost: &ManaCost) -> impl Iterator<Item = ManaPool> {
    // first, pay off the colored portion
    let Some(remaining) = *available - cost.colors else {
        // unable to pay because of the colored mana requirements
        return vec![].into_iter();
    };

    // now, the question is: how many ways can the generic portion be payed off?
    let solutions = payment_methods_for_generic(&remaining, cost.generic)
        .map(|payment| payment + cost.colors)
        .collect_vec();

    // coerce the types so it's the same as above
    solutions.into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_methods_for_generic_zero() {
        let available = ManaPool::try_parse("{R}").expect("should parse");

        let mut payment_methods = payment_methods_for_generic(&available, 0);

        assert_eq!(payment_methods.next(), Some(ManaPool::empty()));
        assert_eq!(payment_methods.next(), None);
    }
    
    #[test]
    fn test_payment_methods_for_generic_zero_from_zero() {
        let available = ManaPool::try_parse("{0}").expect("should parse");

        let mut payment_methods = payment_methods_for_generic(&available, 0);

        assert_eq!(payment_methods.next(), Some(ManaPool::empty()));
        assert_eq!(payment_methods.next(), None);
    }

    #[test]
    fn test_payment_methods_for_generic_single() {
        let available = ManaPool::try_parse("{R}").expect("should parse");

        let mut payment_methods = payment_methods_for_generic(&available, 1);

        assert_eq!(payment_methods.next(), Some(ManaPool::try_parse("{R}").expect("should parse")));
        assert_eq!(payment_methods.next(), None);
    }

    #[test]
    fn test_payment_methods_for_generic_single_multiple_colors() {
        let available = ManaPool::try_parse("{R}{B}").expect("should parse");

        let mut payment_methods = payment_methods_for_generic(&available, 1);

        assert_eq!(payment_methods.next(), Some(ManaPool::try_parse("{R}").expect("should parse")));
        assert_eq!(payment_methods.next(), Some(ManaPool::try_parse("{B}").expect("should parse")));
        assert_eq!(payment_methods.next(), None);
    }

    #[test]
    fn test_payment_methods_for_generic_single_duplicate_colors() {
        let available = ManaPool::try_parse("{R}{R}").expect("should parse");

        let mut payment_methods = payment_methods_for_generic(&available, 1);

        assert_eq!(payment_methods.next(), Some(ManaPool::try_parse("{R}").expect("should parse")));
        assert_eq!(payment_methods.next(), None);
    }

    #[test]
    fn test_payment_methods_for_generic_single_impossible() {
        let available = ManaPool::try_parse("{0}").expect("should parse");

        let mut payment_methods = payment_methods_for_generic(&available, 1);

        assert_eq!(payment_methods.next(), None);
    }

    #[test]
    fn test_payment_methods_for_generic_pay_two_of_the_same() {
        let available = ManaPool::try_parse("{U}{U}").expect("should parse");

        let mut payment_methods = payment_methods_for_generic(&available, 2);

        assert_eq!(payment_methods.next(), Some(ManaPool::try_parse("{U}{U}").expect("should parse")));
        assert_eq!(payment_methods.next(), None);
    }
 
    #[test]
    fn test_payment_methods_for_generic_pay_two_extra_mana() {
        let available = ManaPool::try_parse("{U}{U}{B}{G}").expect("should parse");

        let mut payment_methods = payment_methods_for_generic(&available, 2);

        assert_eq!(payment_methods.next(), Some(ManaPool::try_parse("{B}{G}").expect("should parse")));
        assert_eq!(payment_methods.next(), Some(ManaPool::try_parse("{U}{G}").expect("should parse")));
        assert_eq!(payment_methods.next(), Some(ManaPool::try_parse("{U}{B}").expect("should parse")));
        assert_eq!(payment_methods.next(), Some(ManaPool::try_parse("{U}{U}").expect("should parse")));
        assert_eq!(payment_methods.next(), None);
    }

    #[test]
    fn test_payment_methods_for_generic_pay_two_impossible() {
        let available = ManaPool::try_parse("{U}").expect("should parse");

        let mut payment_methods = payment_methods_for_generic(&available, 2);

        assert_eq!(payment_methods.next(), None);
    }
           
                                    

   #[test]
    fn test_payment_method_single_possibility() {
        let available = ManaPool::try_parse("{R}{R}{G}{G}").expect("should parse");
        let cost = ManaCost::try_parse("{2}{G}{G}").expect("should parse");

        let mut payment_methods = payment_methods_for(&available, &cost);

        assert_eq!(payment_methods.next(), Some(ManaPool::try_parse("{R}{R}{G}{G}").expect("should parse")));
        assert_eq!(payment_methods.next(), None);
    }

    #[test]
    fn test_payment_method_multiple_answers() {
        let available = ManaPool::try_parse("{W}{U}{B}{G}{G}").expect("should parse");
        let cost = ManaCost::try_parse("{2}{G}{G}").expect("should parse");

        let payment_methods = payment_methods_for(&available, &cost);
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

        let payment_methods = payment_methods_for(&available, &cost);
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

