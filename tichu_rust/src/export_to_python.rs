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
pub const COMBINATIONS_TO_2_14_WITH_3_BITS_SET: [u64; 364] = {
    let mut res = [0; 364];
    let mut combination_idx = 0;
    let mut i: u64 = 0;
    while i < (1 << 14) {
        if i.count_ones() == 3 {
            res[combination_idx] = i;
            combination_idx += 1;
        }
        i += 1;
    }
    res
};
#[pyfunction]
pub fn get_legal_outgoing_card_combinations(hand: Hand) -> Vec<(Hand, u8)> {
    let mut res = Vec::with_capacity(1092);
    for combination in COMBINATIONS_TO_2_14_WITH_3_BITS_SET.iter() {
        let deposited_combination_in_hand = unsafe {
            use std::arch::x86_64::_pdep_u64;
            _pdep_u64(*combination, hand)
        };
        let mut temp: Hand = deposited_combination_in_hand;
        while temp != 0 {
            let lsb_card = temp.get_lsb_card();
            res.push((deposited_combination_in_hand, get_exchange_card_type(lsb_card)));
            temp ^= hand!(lsb_card);
        }
    }
    res
}
#[pyfunction]
pub fn could_get_street_bomb(hand: Hand, out_hand: Hand, incoming_card_types: (u8, u8, u8)) -> bool{
    let to_card_bb = |card| {
        match card{
            0 => 0,
            14 => 0,
            15 => 0,
            16 => 0,
            x => MASK_FOUR_OF_KIND[x as usize -1]
        }
    };
    let hand_all = !out_hand &(hand | to_card_bb(incoming_card_types.0) | to_card_bb(incoming_card_types.1) | to_card_bb(incoming_card_types.2));
    hand_all.contains_straight_bomb()
}
#[pyfunction]
pub fn get_legal_incoming_card_combinations(hand: Hand, incoming_card_types: (u8, u8, u8)) -> Vec<Hand> {
    let mut res = vec![];
    let (l, r, p) = incoming_card_types;
    let get_possibilites = |card| {
        match card {
            0 => vec![DOG],
            14 => vec![PHOENIX],
            15 => vec![DRAGON],
            16 => vec![MAHJONG],
            x => vec![
                x + YELLOW, x + BLUE, x + GREEN, x + RED
            ]
        }
    };
    for possible_l in get_possibilites(l) {
        for possible_r in get_possibilites(r) {
            for possible_p in get_possibilites(p) {
                if hand!(possible_l) & hand != 0 {
                    continue;
                }
                if hand!(possible_r) & (hand | hand!(possible_l)) != 0 {
                    continue;
                }
                if hand!(possible_p) & (hand | hand!(possible_l, possible_r)) != 0 {
                    continue;
                }
                res.push(hand!(possible_l, possible_p, possible_r));
            }
        }
    }
    res
}
#[pyfunction]
pub fn print_hand(hand: Hand) -> String {
    hand.pretty_print()
}

#[pyfunction]
pub fn prh_to_incoming_cards(prh: &PyPlayerRoundHand) -> (u8, u8, u8) {
    let left = get_exchange_card_type(prh.left_in_exchange_card());
    let right = get_exchange_card_type(prh.right_in_exchange_card());
    let partner = get_exchange_card_type(prh.partner_in_exchange_card());
    if left < right {
        (left, right, partner)
    } else {
        (right, left, partner)
    }
}
#[pyfunction]
pub fn bulk_transform_db_into_np56_array(db: &BSWSimple) -> PyResult<Py<PyArray2<u8>>> {
    Python::with_gil(|py| {
        let n_samples = db.len() * 4;
        let arr = PyArray2::<u8>::zeros(py, [n_samples, 56], false);
        let mut buffer = unsafe { arr.as_array_mut() };
        for (round_idx, round) in db.rounds.iter().enumerate() {
            for player_id in 0..4 {
                let transformed = transform_hand_to_lower_56_bits(round[player_id].first_14());
                let row_idx = round_idx * 4 + player_id;

                // Write all 56 bits at once using bit operations
                for bit_pos in 0..56 {
                    buffer[[row_idx, bit_pos]] = ((transformed >> bit_pos) & 1) as u8;
                }
            }
        }
        let owned_arr: Py<PyArray2<u8>> = arr.to_owned().into();
        Ok(owned_arr)
    })
}
#[pyfunction]
pub fn transform_into_np56_array(hand: Hand) -> PyResult<Py<PyArray2<u8>>>{
    Python::with_gil(|py| {
        let arr = PyArray2::<u8>::zeros(py, [1, 56], false);
        let mut buffer = unsafe { arr.as_array_mut() };
        let transformed = transform_hand_to_lower_56_bits(hand);

        // Write all 56 bits at once using bit operations
        for bit_pos in 0..56 {
            buffer[[0, bit_pos]] = ((transformed >> bit_pos) & 1) as u8;
        }
        let owned_arr: Py<PyArray2<u8>> = arr.to_owned().into();
        Ok(owned_arr)
    })
}

#[pyfunction]
pub fn bulk_transform_db_into_np90_array(db: &BSWSimple) -> PyResult<Py<PyArray2<u8>>> {
    Python::with_gil(|py| {
        let n_samples = db.len() * 4;
        let arr = PyArray2::<u8>::zeros(py, [n_samples, 90], false);
        let mut buffer = unsafe { arr.as_array_mut() };
        for (round_idx, round) in db.rounds.iter().enumerate() {
            for player_id in 0..4 {
                let transformed = transform_hand_to_lower_56_bits(round[player_id].final_14());
                let row_idx = round_idx * 4 + player_id;

                // Write all 56 bits at once using bit operations
                for bit_pos in 0..56 {
                    buffer[[row_idx, bit_pos]] = ((transformed >> bit_pos) & 1) as u8;
                }
                let in_partner = get_exchange_card_type(round[player_id].partner_in_exchange_card());
                let out_partner = get_exchange_card_type(round[player_id].partner_out_exchange_card());
                buffer[[row_idx, 56+in_partner as usize]] = 1;
                buffer[[row_idx, 73+out_partner as usize]] = 1;
            }
        }
        let owned_arr: Py<PyArray2<u8>> = arr.to_owned().into();
        Ok(owned_arr)
    })
}
#[pyfunction]
pub fn prepare_batch_np90_array(
    stripped_hand: Hand,
    in_partner: u8,
    out_partner: u8,
    in_hands: Vec<Hand>,
    mut existing_array: PyReadwriteArray2<u8>
) -> PyResult<()> {
    Python::with_gil(|_py| {
        for (idx, in_hand) in in_hands.iter().enumerate() {
            let final_hand = stripped_hand ^ in_hand;
            let transformed = transform_hand_to_lower_56_bits(final_hand);

            // Write all 56 bits at once using bit operations
            for bit_pos in 0..56 {
                *existing_array.get_mut([idx, bit_pos]).unwrap() = ((transformed >> bit_pos) & 1) as u8;
            }
            *existing_array.get_mut([idx, 56+in_partner as usize]).unwrap() = 1;
            *existing_array.get_mut([idx, 73+out_partner as usize]).unwrap() = 1;

        }

        Ok(())
    })
}
#[pyfunction]
pub fn transform_into_np90_array(final_hand: Hand, in_partner: u8, out_partner: u8) -> PyResult<Py<PyArray2<u8>>>{
    Python::with_gil(|py| {
        let arr = PyArray2::<u8>::zeros(py, [1, 90], false);
        let mut buffer = unsafe { arr.as_array_mut() };
        let transformed = transform_hand_to_lower_56_bits(final_hand);

        // Write all 56 bits at once using bit operations
        for bit_pos in 0..56 {
            buffer[[0, bit_pos]] = ((transformed >> bit_pos) & 1) as u8;
        }
        buffer[[0, 56+in_partner as usize]] = 1;
        buffer[[0, 73+out_partner as usize]] = 1;
        let owned_arr: Py<PyArray2<u8>> = arr.to_owned().into();
        Ok(owned_arr)
    })
}

pub fn transform_hand_to_lower_56_bits(hand: Hand) -> u64 {
    unsafe {
        use std::arch::x86_64::_pext_u64;
        _pext_u64(hand, MASK_ALL)
    }
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
    m.add_function(wrap_pyfunction!(bulk_transform_db_into_np56_array, m)?)?;
    m.add_function(wrap_pyfunction!(transform_into_np56_array, m)?)?;
    m.add_function(wrap_pyfunction!(bulk_transform_db_into_np90_array, m)?)?;
    m.add_function(wrap_pyfunction!(could_get_street_bomb, m)?)?;
    m.add_function(wrap_pyfunction!(transform_into_np90_array, m)?)?;
    m.add_function(wrap_pyfunction!(prepare_batch_np90_array, m)?)?;
    m.add_function(wrap_pyfunction!(prh_to_incoming_cards, m)?)?;
    m.add_function(wrap_pyfunction!(get_legal_incoming_card_combinations, m)?)?;
    m.add_function(wrap_pyfunction!(get_legal_outgoing_card_combinations, m)?)?;

    Ok(())
}