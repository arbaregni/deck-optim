use itertools::Itertools;

use crate::collection::Card;
use crate::game::mana::ManaPool;
use crate::game::mana::ManaCost;
use crate::game::mana::ManaSource;

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


/// A payment solution is a list of 
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentSolution {
    pub cards_to_tap: Vec<(Card, ManaPool)>,
    pub mana_used: ManaPool,
}
impl PaymentSolution {
    fn new() -> Self {
        Self {
            cards_to_tap: Vec::with_capacity(3),
            mana_used: ManaPool::empty()
        }
    }
    fn add(&mut self, card: Card, mana: ManaPool) {
        self.mana_used = self.mana_used + mana;
        self.cards_to_tap.push((card, mana));
    }
    fn with_payment(&self, card: Card, mana: ManaPool) -> Self {
        let mut next = self.clone();
        next.add(card, mana);
        next
    }
}

/// Construct a possible way to pay for the given mana cost, using a list of mana sources.
///
/// The sources actually used will be removed from the input parameter and passed back as realized
/// payment methods in the output.
/// 
/// # Example
/// ```
/// use deck_optim::strategies::payment_solver;
/// use deck_optim::strategies::payment_solver::PaymentSolution;
/// use deck_optim::game::mana::ManaPool;
/// use deck_optim::game::mana::ManaCost;
/// use deck_optim::game::mana::ManaSource;
///
/// let [mock_forest, mock_taiga] = deck_optim::collection::get_sample_cards_static::<2>();
///
/// let mut mana_sources = vec![
///     ManaSource {
///         card: mock_forest,
///         produces: vec![ManaPool::green(1)]
///     },
///     ManaSource {
///         card: mock_taiga,
///         produces: vec![ManaPool::red(1), ManaPool::green(1)]
///     }
/// ];
///
/// let cost_to_pay = ManaCost::try_parse("{R}{G}").expect("should parse");
///
/// let (solution, unused_sources) = payment_solver::autotap_pay_for(mana_sources, &cost_to_pay)
///     .expect("found a solution");
///
/// let expected_solution = PaymentSolution {
///     cards_to_tap: vec![
///         (mock_forest, ManaPool::green(1)),
///         (mock_taiga, ManaPool::red(1))
///     ],
///     mana_used: ManaPool::try_parse("{R}{G}").expect("should parse")
/// };
///
/// assert_eq!(solution, expected_solution);
/// assert_eq!(0, unused_sources.len());
/// ```
pub fn autotap_pay_for(mut available_mana: Vec<ManaSource>, cost: &ManaCost) -> Option<(PaymentSolution, Vec<ManaSource>)> {
    let mut partial_soln = PaymentSolution::new();
    // first, we tap everything that only can tap for one thing.
    // this reduces the size of the list and prevents unnecessary recursion
    
    // TODO: only tap what is needed
    available_mana.retain(|mana_source| {
        match &mana_source.produces[..] {
            &[] => false,
            &[mana] => {
                partial_soln.add(mana_source.card, mana);
                false
            }
            _ => true,
        }
    });

    fn _autotap_recursive(partial_soln: PaymentSolution, mut available_mana: Vec<ManaSource>, cost: &ManaCost) -> Option<(PaymentSolution, Vec<ManaSource>)> {
        // if we can pay for the cost already, do so and we are done.
        // this should be done before going into each source, because we might have enough floating
        // mana already to pay for the cost
        if payment_methods_for(&partial_soln.mana_used, cost).next().is_some() {
            return Some((partial_soln, available_mana));
        }

        let Some(new_source) = available_mana.pop() else {
            return None; // no more mana to use
        };
        // try to tap the first one, then tap the second
        for payment in new_source.produces.iter() {

            let next = partial_soln.with_payment(new_source.card, *payment);

            // TODO: is this too inefficient?
            if let Some(solution) = _autotap_recursive(next, available_mana.clone(), cost) {
                return Some(solution);
            }
        }

        None
    }

    _autotap_recursive(partial_soln, available_mana, cost)
}

#[cfg(test)]
mod tests {
    use crate::collection;

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

    #[test]
    fn test_autotap_simple_match() {
        let [mock_mountain] = collection::get_sample_cards_static::<1>();

        let mana_sources = vec![
            ManaSource {
                card: mock_mountain,
                produces: vec![ManaPool::red(1)]
            }
        ];

        let cost_to_pay = ManaCost::try_parse("{R}").expect("should parse");

        let (solution, unused_sources) = autotap_pay_for(mana_sources, &cost_to_pay)
            .expect("found a solution");

        let expected_solution = PaymentSolution {
            cards_to_tap: vec![
                (mock_mountain, ManaPool::red(1))
            ],
            mana_used: ManaPool::red(1)
        };

        assert_eq!(solution, expected_solution);
        assert_eq!(0, unused_sources.len());
    }

    #[test]
    fn test_overpayment() {
        let [mock_ancient_tomb] = collection::get_sample_cards_static::<1>();

        let mana_sources = vec![
            ManaSource {
                card: mock_ancient_tomb,
                produces: vec![ManaPool::colorless(2)]
            }
        ];

        let cost_to_pay = ManaCost::try_parse("{1}").expect("should parse");

        let (solution, unused_mana) = autotap_pay_for(mana_sources, &cost_to_pay)
            .expect("found a solution");

        let expected_solution = PaymentSolution {
            cards_to_tap: vec![
                (mock_ancient_tomb, ManaPool::colorless(2))  // Overpaying, but necessary
            ],
            mana_used: ManaPool::colorless(2)
        };

        assert_eq!(solution, expected_solution);
        assert_eq!(0, unused_mana.len()); 
    }

    #[test]
    fn test_unable_to_pay() {

        let [mock_forest] = collection::get_sample_cards_static::<1>();

        let mana_sources = vec![
            ManaSource {
                card: mock_forest,
                produces: vec![ManaPool::green(1)]
            }
        ];

        let cost_to_pay = ManaCost::try_parse("{R}{G}").expect("should parse");

        let solution = autotap_pay_for(mana_sources, &cost_to_pay);

        assert_eq!(solution, None);
    }

    #[test]
    fn test_single_dual_land() {
        let [mock_forest, mock_taiga] = collection::get_sample_cards_static::<2>();

        let mana_sources = vec![
            ManaSource {
                card: mock_forest,
                produces: vec![ManaPool::green(1)]
            },
            ManaSource {
                card: mock_taiga,
                produces: vec![ManaPool::red(1), ManaPool::green(1)]
            }
        ];

        let cost_to_pay = ManaCost::try_parse("{R}{G}").expect("should parse");

        let (solution, unused_mana) = autotap_pay_for(mana_sources, &cost_to_pay)
            .expect("found a solution");

        let expected_solution = PaymentSolution {
            cards_to_tap: vec![
                (mock_forest, ManaPool::green(1)),
                (mock_taiga, ManaPool::red(1))
            ],
            mana_used: ManaPool::try_parse("{R}{G}").expect("should parse")
        };

        assert_eq!(solution, expected_solution);
        assert_eq!(0, unused_mana.len());
    }

    #[test]
    fn test_dual_land_unable_to_pay() {
      let [mock_forest, mock_taiga] = collection::get_sample_cards_static::<2>();

        let mana_sources = vec![
            ManaSource {
                card: mock_forest,
                produces: vec![ManaPool::green(1)]
            },
            ManaSource {
                card: mock_taiga,
                produces: vec![ManaPool::red(1), ManaPool::green(1)]
            }
        ];

        let cost_to_pay = ManaCost::try_parse("{U}{G}").expect("should parse");

        let solution = autotap_pay_for(mana_sources, &cost_to_pay);

        assert_eq!(solution, None);
    }

    #[test]
    fn test_mana_source_with_no_produces() {
        let [mock_forest, mock_taiga, mock_other] = collection::get_sample_cards_static::<3>();

        let mana_sources = vec![
            ManaSource {
                card: mock_forest,
                produces: vec![ManaPool::green(1)]
            },
            ManaSource {
                card: mock_taiga,
                produces: vec![ManaPool::red(1), ManaPool::green(1)]
            },
            ManaSource {
                card: mock_other,
                produces: vec![]
            }
        ];

        let cost_to_pay = ManaCost::try_parse("{R}{G}").expect("should parse");

        let (solution, unused_sources) = autotap_pay_for(mana_sources, &cost_to_pay)
            .expect("found a solution");

        let expected_solution = PaymentSolution {
            cards_to_tap: vec![
                (mock_forest, ManaPool::green(1)),
                (mock_taiga, ManaPool::red(1))
            ],
            mana_used: ManaPool::try_parse("{R}{G}").expect("should parse")
        };

        assert_eq!(solution, expected_solution);
        assert_eq!(0, unused_sources.len());
    }

}

