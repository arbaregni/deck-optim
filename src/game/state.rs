use itertools::Itertools;
use rand::Rng;

use crate::collection::Card;
use crate::game::card::CardType;
use crate::trial::Rand;
use crate::game::{
    Battlefield, CommandZone, Graveyard, Hand, Library 
};
use crate::game::unordered_pile::UnorderedPile;
use crate::game::Deck;
use crate::game::Zone;
use crate::game::card_play::CardPlay;
use crate::game::mana::ManaPool;
use crate::game::mana::ManaSource;

const PROB_OF_GOING_FIRST: f64 = 0.5;
const HAND_SIZE: u32 = 7;

/// Represents the state of the game simulation at a given instant.
#[derive(Debug, Clone)]
pub struct State {
    pub turn: u32,
    pub draw_on_first_turn: bool,
    pub num_mulligans_taken: u32,
    pub game_loss: bool,
    pub turn_state: TurnState,

    pub max_land_drops_per_turn: u32,

    // 
    // ZONES
    //
    pub library: Library,
    pub hand: Hand,
    pub permanents: Battlefield,
    pub graveyard: Graveyard, 
    pub command_zone: CommandZone,
}

impl State {
    /// Create a new initial state from the deck. 
    pub fn new(deck: Deck, rng: &mut Rand) -> State {
        State {
            library: deck.deck.to_ordered(rng),
            command_zone: deck.command_zone,

            hand: Hand::empty(),
            permanents: Battlefield::empty(),
            graveyard: Graveyard::empty(),

            turn: 0,
            draw_on_first_turn: rng.gen_bool(PROB_OF_GOING_FIRST),
            num_mulligans_taken: 0,
            game_loss: false,
            max_land_drops_per_turn: 1,
            turn_state: TurnState::new(),
        }
    }

    // ===================================================================
    //  Game actions and methods to mutate the state
    // ===================================================================

    /// Draw a hand, decreases as the number of mulligans taken.
    pub fn draw_hand(&mut self) {
        if self.num_mulligans_taken >= HAND_SIZE {
            log::warn!("taking more mulligans than hand size allowed, ignoring extra mulligans");
            return;
        }
        let hand_size = HAND_SIZE - self.num_mulligans_taken;
        self.hand = self.library.draw_n(hand_size as usize).into();
    }

    /// Put the hand into library and shuffle. Hand is now empty.
    pub fn shuffle_hand_into_library(&mut self, rng: &mut Rand) {
        self.library.add_to_top(&self.hand);
        self.library.shuffle(rng);
        self.hand = Hand::empty();
    }

    
    /// Remove a card from the top of the library and put it into hand.
    /// Will mark the game loss flag if this draw is impossible.
    pub fn draw_to_hand(&mut self) {
        match self.library.draw() {
            Some(card) => {
                self.hand.add(card);
            }
            None => {
                self.game_loss = true;
            }
        }
    }

    /// Removes a card from wherever.
    fn remove_from_zone(&mut self, card: Card, zone: Zone) {
        match zone {
            Zone::CommandZone => self.command_zone.remove(card),
            Zone::Hand => self.hand.remove(card),
            Zone::Graveyard => self.graveyard.remove(card),
            Zone::Battlefield => self.permanents.remove(card),
            Zone::Library => todo!("not sure how to remove from library. i.e. which copy are we removing? is it from the top? etc."),
        };
    }

    /// Create a copy of this state, where the card has been played.
    pub fn with_having_played(&self, card_play: CardPlay) -> Self {
        let mut next = self.clone();
        next.play_card(card_play);
        next
    }

    /// Move the card from wherever it came from to wherever it is going.
    pub fn play_card(&mut self, card_play: CardPlay) {
         let CardPlay { card, zone, payment: _ } = card_play;

         self.remove_from_zone(card, zone);

         match card.data().card_type {
            CardType::Instant  | CardType::Sorcery => {
                self.graveyard.add(card);
            }
            CardType::Land => {
                self.turn_state.land_drops_made += 1;
                if self.turn_state.land_drops_made > self.max_land_drops_per_turn {
                    log::warn!("ILLEGAL PLAY: played {card:?} as {}th land drop, only {}", self.turn_state.land_drops_made, self.max_land_drops_per_turn);
                }
                self.permanents.add(card);
            }
            _ => {
                self.permanents.add(card);
            }
        }
    }

    pub fn end_turn(&mut self) {
        self.turn_state.reset();
        self.turn += 1;
    }


    // =====================================================================
    //  Accessors, helpful for the trial which is running the simulation,
    //    or for the strategies which need complicated views into the state
    // =====================================================================

    pub fn turn(&self) -> u32 {
        self.turn
    }


    pub fn legal_card_plays(&self) -> impl Iterator<Item = CardPlay> + '_ {
        let hand = self.hand.iter()
            .filter(|c| c.data().cost.is_some())
            .map(|card| CardPlay {
                card, zone: Zone::Hand, payment: ManaPool::empty()
            });
        let commanders = self.command_zone.iter()
            .map(|card| CardPlay {
                card, zone: Zone::CommandZone, payment: ManaPool::empty()
            });
        
        // TODO: some enforcement here, before we go into the strategies
        hand.chain(commanders)
    }

    pub fn legal_land_drops(&self) -> impl Iterator<Item = CardPlay> + use<'_> {
        let hand = self.hand
            .iter()
            .filter(|c| c.data().card_type == CardType::Land)
            .unique_by(|c| c.name())
            .map(|card| CardPlay {
                card, zone: Zone::Hand, payment: ManaPool::empty()
            });
        hand
    }

    pub fn mana_sources(&self) -> impl Iterator<Item = ManaSource> + use<'_> {
        self.permanents
            .iter()
            .filter_map(ManaSource::try_from)
    }
    
    // ===========================================
    //   Some ways to measure the game state 
    // ===========================================


    /// How much mana does the player theoretically have access to?
    /// Note: this should **not** be used for making game decisions, it's merely a heuristic.
    pub fn available_mana(&self) -> u8 {
        self.permanents
            .iter()
            .filter_map(ManaSource::try_from)
            .map(|mana_source| mana_source.highest_mana_value())
            .sum()
    }

    /// How many lands does the player have in hand?
    pub fn num_lands_in_hand(&self) -> usize {
        self.hand
            .iter()
            .filter(|c| c.data().card_type == CardType::Land)
            .count()
    }

    /// How many lands does the player have in play?
    pub fn num_lands_in_play(&self) -> usize {
        self.permanents
            .iter()
            .filter(|c| c.data().card_type == CardType::Land)
            .count()
    }
}

/// For state that is reset every cleanup phase
#[derive(Debug,Clone)]
pub struct TurnState {
    pub land_drops_made: u32,
    pub tapped: UnorderedPile,
}


impl TurnState {
    pub fn new() -> TurnState {
        TurnState {
            land_drops_made: 0,
            tapped: UnorderedPile::empty(),
        }
    }
    pub fn reset(&mut self) {
        self.land_drops_made = 0;
        self.tapped.clear();
    }
    pub fn mark_as_tapped(&mut self, card: Card) {
        if self.is_tapped(card) {
            log::error!("card is already tapped: {card:?}");
            return;
        }
        self.tapped.add(card);
    }
    pub fn is_tapped(&mut self, card: Card) -> bool {
        self.tapped.iter().contains(&card)
    }
}
