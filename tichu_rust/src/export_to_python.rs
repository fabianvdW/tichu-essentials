#![allow(unused_imports)]
pub mod analysis;
pub mod bsw_binary_format;
pub mod bsw_database;
pub mod countable_properties;
pub mod enumerate_hands;
pub mod enumeration_results;
pub mod pair_street_detection_trick;
pub mod street_detection_tricks;
pub mod tichu_hand;

use crate::analysis::exchange_stats::get_exchange_card_type;
use crate::bsw_binary_format::binary_format_constants::{
    PlayerIDInternal, Rank, Score, TichuCall, CALL_PLAYER_0_MASK, CALL_PLAYER_1_MASK,
    CALL_PLAYER_2_MASK, CALL_PLAYER_3_MASK, CARD_SCORE_MASK, LEFT_IN_EXCHANGE_MASK,
    LEFT_OUT_EXCHANGE_MASK, PARTNER_IN_EXCHANGE_MASK, PARTNER_OUT_EXCHANGE_MASK, PLAYER_0,
    PLAYER_2, PLAYER_ID_MASK, RANK_1, RANK_2, RANK_PLAYER_0_MASK, RANK_PLAYER_1_MASK,
    RANK_PLAYER_2_MASK, RANK_PLAYER_3_MASK, RIGHT_IN_EXCHANGE_MASK, RIGHT_OUT_EXCHANGE_MASK,
};
use crate::bsw_binary_format::player_round_hand::PlayerRoundHand;
use crate::bsw_database::DataBase;
use crate::tichu_hand::{
    CardIndex, Hand, TichuHand, BLUE, DOG, DRAGON, GREEN, MAHJONG, MASK_ALL, MASK_FOUR_OF_KIND,
    PHOENIX, RED, SPECIAL_CARD, YELLOW,
};
use numpy::{PyArray2, PyArrayMethods, PyReadwriteArray2, ToPyArray};
use pyo3::prelude::*;
use pyo3::PyResult;

/// Print a readable representation of a Tichu hand
///
/// Args:
///     hand (Hand): The hand to print
///
/// Returns:
///     str: A string representation of the hand
#[pyfunction]
pub fn print_hand(hand: Hand) -> String {
    hand.pretty_print()
}

/// A Python wrapper for PlayerRoundHand representing a player's round in Tichu
///
/// This class provides access to a player's cards, exchanges, calls, and round statistics
/// throughout different phases of a Tichu round.
#[pyclass]
#[derive(Clone)]
pub struct PyPlayerRoundHand(PlayerRoundHand);

#[pymethods]
impl PyPlayerRoundHand {
    /// Create a new PyPlayerRoundHand instance
    #[new]
    pub fn new() -> PyPlayerRoundHand {
        PyPlayerRoundHand(PlayerRoundHand::default())
    }

    /// Get the first 8 cards dealt to the player
    ///
    /// Returns:
    ///     Hand: The first 8 cards of the initial deal
    #[getter(first_8)]
    pub fn first_8(&self) -> Hand {
        self.0.first_8
    }

    /// Get additional round information encoded as a bitmap
    ///
    /// Returns:
    ///     int: Encoded round information
    #[getter]
    pub fn extras(&self) -> u64 {
        self.0.extras
    }

    /// Get the 14 cards initially dealt to the player
    ///
    /// Returns:
    ///     Hand: The complete initial deal before exchanges
    #[getter]
    pub fn first_14(&self) -> Hand {
        self.0.first_14
    }

    /// Get the player's final hand after exchanges
    ///
    /// Returns:
    ///     Hand: The player's complete hand after all exchanges
    #[getter]
    pub fn final_14(&self) -> Hand {
        self.0.final_14()
    }

    /// Get the card given to the left player during exchange
    ///
    /// Returns:
    ///     CardIndex: The index of the card passed left
    pub fn left_out_exchange_card(&self) -> CardIndex {
        self.0.left_out_exchange_card()
    }

    /// Get the card given to the partner during exchange
    ///
    /// Returns:
    ///     CardIndex: The index of the card passed to partner
    pub fn partner_out_exchange_card(&self) -> CardIndex {
        self.0.partner_out_exchange_card()
    }

    /// Get the card given to the right player during exchange
    ///
    /// Returns:
    ///     CardIndex: The index of the card passed right
    pub fn right_out_exchange_card(&self) -> CardIndex {
        self.0.right_out_exchange_card()
    }

    /// Get the card received from the left player during exchange
    ///
    /// Returns:
    ///     CardIndex: The index of the card received from left
    pub fn left_in_exchange_card(&self) -> CardIndex {
        self.0.left_in_exchange_card()
    }

    /// Get the card received from the partner during exchange
    ///
    /// Returns:
    ///     CardIndex: The index of the card received from partner
    pub fn partner_in_exchange_card(&self) -> CardIndex {
        self.0.partner_in_exchange_card()
    }

    /// Get the card received from the right player during exchange
    ///
    /// Returns:
    ///     CardIndex: The index of the card received from right
    pub fn right_in_exchange_card(&self) -> CardIndex {
        self.0.right_in_exchange_card()
    }

    /// Get a player's Tichu call
    ///
    /// Args:
    ///     player_id (PlayerIDInternal): The ID of the player
    ///
    /// Returns:
    ///     TichuCall: The type of Tichu call made by the player
    ///     0 <=> No Call Made
    ///     1 <=> Tichu Call
    ///     2 <=> Grand Tichu Call
    pub fn player_call(&self, player_id: PlayerIDInternal) -> TichuCall {
        self.0.player_call(player_id)
    }

    /// Get this player's ID
    ///
    /// Returns:
    ///     PlayerIDInternal: The ID of this player (0-3)
    ///
    pub fn player_id(&self) -> PlayerIDInternal {
        self.0.player_id()
    }

    /// Get a player's finishing rank in the round
    ///
    /// Args:
    ///     player_id (PlayerIDInternal): The ID of the player (0-3)
    ///
    /// Returns:
    ///     Rank: The rank (order of finishing) for the specified player
    ///     0 <=> Rank 1
    ///     1 <=> Rank 2
    ///     2 <=> Rank 3
    ///     3 <=> Rank 4
    pub fn player_rank(&self, player_id: PlayerIDInternal) -> Rank {
        self.0.player_rank(player_id)
    }

    /// Check if Team 1 achieved a double win
    ///
    /// Returns:
    ///     bool: True if Team 1 (players 0 and 2) finished 1-2
    pub fn is_double_win_team_1(&self) -> bool {
        self.0.is_double_win_team_1()
    }

    /// Check if Team 2 achieved a double win
    ///
    /// Returns:
    ///     bool: True if Team 2 (players 1 and 3) finished 1-2
    pub fn is_double_win_team_2(&self) -> bool {
        self.0.is_double_win_team_2()
    }

    /// Get the round scores for both teams
    ///
    /// Returns:
    ///     tuple[Score, Score]: A tuple of (Team 1 score, Team 2 score)
    pub fn round_score(&self) -> (Score, Score) {
        self.0.round_score()
    }

    /// Get the relative score gain for this player's team
    ///
    /// Returns:
    ///     Score: The net points gained by this player's team
    pub fn round_score_relative_gain(&self) -> Score {
        self.0.round_score_relative_gain()
    }
}

/// A simplified interface for reading and accessing BSW (BrettSpielWelt) files
///
/// This class provides access to Tichu rounds stored in BSW format, allowing iteration
/// over rounds and access to player data within each round.
#[pyclass]
pub struct BSWSimple {
    pub rounds: Vec<[PyPlayerRoundHand; 4]>,
}

#[pymethods]
impl BSWSimple {
    /// Create a new BSWSimple instance from a BSW file
    ///
    /// Args:
    ///     path (str): Path to the BSW file to load
    ///
    /// Returns:
    ///     BSWSimple: A new instance containing the loaded rounds
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
                    PyPlayerRoundHand(round.player_rounds[3].clone()),
                ])
            }
        }
        BSWSimple { rounds: rounds }
    }

    /// Get the total number of rounds
    ///
    /// Returns:
    ///     int: The number of rounds in the BSW file
    fn len(&self) -> usize {
        self.rounds.len()
    }

    /// Get a specific round's data
    ///
    /// Args:
    ///     index (int): The index of the round to retrieve
    ///
    /// Returns:
    ///     Optional[List[PyPlayerRoundHand]]: The round data for all players,
    ///     or None if the index is out of bounds
    fn get_round(&self, index: usize) -> Option<[PyPlayerRoundHand; 4]> {
        self.rounds.get(index).cloned()
    }
}

/// A Python module for analyzing Tichu games
///
/// This module provides tools for reading and analyzing Tichu game data,
/// including hand analysis, round statistics, and game outcomes.
#[pymodule]
fn tichu_rustipy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPlayerRoundHand>()?;
    m.add_class::<BSWSimple>()?;
    m.add_function(wrap_pyfunction!(print_hand, m)?)?;
    Ok(())
}
