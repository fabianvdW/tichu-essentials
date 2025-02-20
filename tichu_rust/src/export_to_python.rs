#![allow(unused_imports)]
pub mod tichu_hand;
pub mod enumerate_hands;
pub mod countable_properties;
pub mod enumeration_results;
pub mod bsw_database;
pub mod street_detection_tricks;
pub mod pair_street_detection_trick;
pub mod bsw_binary_format;
pub mod analysis;

use numpy::{PyArray2, PyArrayMethods, ToPyArray, PyReadwriteArray2};
use pyo3::prelude::*;
use pyo3::PyResult;
use crate::bsw_binary_format::binary_format_constants::{PlayerIDInternal, Rank, Score, TichuCall, CALL_PLAYER_0_MASK, CALL_PLAYER_1_MASK, CALL_PLAYER_2_MASK, CALL_PLAYER_3_MASK, CARD_SCORE_MASK, LEFT_IN_EXCHANGE_MASK, LEFT_OUT_EXCHANGE_MASK, PARTNER_IN_EXCHANGE_MASK, PARTNER_OUT_EXCHANGE_MASK, PLAYER_0, PLAYER_2, PLAYER_ID_MASK, RANK_1, RANK_2, RANK_PLAYER_0_MASK, RANK_PLAYER_1_MASK, RANK_PLAYER_2_MASK, RANK_PLAYER_3_MASK, RIGHT_IN_EXCHANGE_MASK, RIGHT_OUT_EXCHANGE_MASK};
use crate::tichu_hand::{CardIndex, Hand, MASK_ALL, TichuHand, SPECIAL_CARD, PHOENIX, DRAGON, MAHJONG, DOG, YELLOW, BLUE, GREEN, RED, MASK_FOUR_OF_KIND};
use crate::bsw_binary_format::player_round_hand::PlayerRoundHand;
use crate::bsw_database::DataBase;
use crate::analysis::exchange_stats::get_exchange_card_type;

//We mostly duplicate code/delegate to the Rust structs here. This provides clear seperation
// of the limited amount of functions we export to Python, which is what I currently prefer.
// An alternative appraoch would be to feature gate #[pyclass] #[pymethods] directly
// into the Rust code by a python feature.
#[pyfunction]
pub fn print_hand(hand: Hand) -> String {
    hand.pretty_print()
}
#[pyclass]
#[derive(Clone)]
pub struct PyPlayerRoundHand(PlayerRoundHand);

#[pymethods]
impl PyPlayerRoundHand {
    #[new]
    pub fn new() -> PyPlayerRoundHand {
        PyPlayerRoundHand(PlayerRoundHand::default())
    }
    #[getter(first_8)]
    pub fn first_8(&self) -> Hand {
        self.0.first_8
    }
    #[getter]
    pub fn extras(&self) -> u64 {
        self.0.extras
    }
    #[getter]
    pub fn first_14(&self) -> Hand {
        self.0.first_14
    }
    #[getter]
    pub fn final_14(&self) -> Hand {
        self.0.final_14()
    }

    pub fn left_out_exchange_card(&self) -> CardIndex {
        self.0.left_out_exchange_card()
    }
    pub fn partner_out_exchange_card(&self) -> CardIndex {
        self.0.partner_out_exchange_card()
    }
    pub fn right_out_exchange_card(&self) -> CardIndex {
        self.0.right_out_exchange_card()
    }

    pub fn left_in_exchange_card(&self) -> CardIndex {
        self.0.left_in_exchange_card()
    }
    pub fn partner_in_exchange_card(&self) -> CardIndex {
        self.0.partner_in_exchange_card()
    }
    pub fn right_in_exchange_card(&self) -> CardIndex {
        self.0.right_in_exchange_card()
    }

    pub fn player_call(&self, player_id: PlayerIDInternal) -> TichuCall {
        self.0.player_call(player_id)
    }
    pub fn player_id(&self) -> PlayerIDInternal {
        self.0.player_id()
    }
    pub fn player_rank(&self, player_id: PlayerIDInternal) -> Rank {
        self.0.player_rank(player_id)
    }

    pub fn is_double_win_team_1(&self) -> bool {
        self.0.is_double_win_team_1()
    }
    pub fn is_double_win_team_2(&self) -> bool {
        self.0.is_double_win_team_2()
    }
    pub fn round_score(&self) -> (Score, Score) {
        self.0.round_score()
    }
    pub fn round_score_relative_gain(&self) -> Score {
        self.0.round_score_relative_gain()
    }

    pub fn round_score_relative_gain_gt_as_t(&self) -> Score {
        //Reported absolute to Team1;
        let mut score_team_1: Score = 0;
        let mut score_team_2: Score = 0;
        score_team_1 += self.0.player_0_call().min(1) as Score * 100 * {
            if self.0.player_0_rank() == RANK_1 {
                1
            } else {
                -1
            }
        };
        score_team_1 += self.0.player_2_call().min(1) as Score * 100 * {
            if self.0.player_2_rank() == RANK_1 {
                1
            } else {
                -1
            }
        };
        score_team_2 += self.0.player_1_call().min(1) as Score * 100 * {
            if self.0.player_1_rank() == RANK_1 {
                1
            } else {
                -1
            }
        };
        score_team_2 += self.0.player_3_call().min(1) as Score * 100 * {
            if self.0.player_3_rank() == RANK_1 {
                1
            } else {
                -1
            }
        };
        //Double Win or Card Points
        if self.is_double_win_team_1() {
            //Double Win for Team 1
            score_team_1 += 200;
        } else if self.is_double_win_team_2() {
            //Double Win for Team 2
            score_team_2 += 200;
        } else {
            let card_score: Score = ((self.0.extras & CARD_SCORE_MASK) >> 54) as Score;
            score_team_1 += card_score - 25;
            score_team_2 += 125 - card_score;
        }
        if self.player_id() == PLAYER_0 || self.player_id() == PLAYER_2 {
            score_team_1 - score_team_2
        } else {
            score_team_2 - score_team_1
        }
    }
}

#[pyclass]
pub struct BSWSimple {
    pub rounds: Vec<[PyPlayerRoundHand; 4]>,
}
#[pymethods]
impl BSWSimple {
    #[new]
    pub fn new(path: &str) -> BSWSimple {
        let db = DataBase::read(path).unwrap();
        let num_rounds = db.games.iter().fold(0, |acc, inc| acc + inc.rounds.len());
        let mut rounds = Vec::with_capacity(num_rounds);
        for game in db.games.iter() {
            for (round, _) in game.rounds.iter() {
                rounds.push([
                    PyPlayerRoundHand(round.player_rounds[0].clone()),
                    PyPlayerRoundHand(round.player_rounds[1].clone()),
                    PyPlayerRoundHand(round.player_rounds[2].clone()),
                    PyPlayerRoundHand(round.player_rounds[3].clone())
                ])
            }
        }
        BSWSimple { rounds: rounds }
    }
    fn len(&self) -> usize {
        self.rounds.len()
    }
    fn get_round(&self, index: usize) -> Option<[PyPlayerRoundHand; 4]> {
        self.rounds.get(index).cloned()
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn tichu_rustipy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPlayerRoundHand>()?;
    m.add_class::<BSWSimple>()?;
    m.add_function(wrap_pyfunction!(print_hand, m)?)?;
    Ok(())
}