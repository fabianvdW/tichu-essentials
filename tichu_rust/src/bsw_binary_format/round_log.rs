use std::fmt::{Debug, Formatter};
use bitcode::{Decode, Encode};
use datasize::DataSize;
use crate::tichu_hand::*;
use crate::bsw_binary_format::binary_format_constants::*;
use crate::bsw_binary_format::{round::Round};
use crate::bsw_binary_format::round_log::RoundLogIntegrityError::{Child, StartTrickIsNotNextInLine};
use crate::bsw_binary_format::round_log::TrickIntegrityError::EmptyTrickLog;
use crate::bsw_binary_format::trick::{Trick, TrickIntegrityError};
use crate::hand;


#[derive(Encode, Decode, Default, DataSize)]
pub struct RoundLog {
    pub mahjong_wish: Option<CardIndex>,
    pub dragon_player_gift: Option<PlayerIDInternal>,
    pub log: Vec<u8>, // Vec<CardIndex> basically, with some extra seperating Bytes
    //Seperator bytes are hardcoded below; -> Trick::serialize_into.
}
pub type TaggedCardIndex = u8; //Lower 6 bits are CardIndex, upper 2 bits are Tag

pub trait TaggedCardIndexT {
    fn construct(player: PlayerIDInternal, card_index: CardIndex) -> Self;
    fn get_player(&self) -> PlayerIDInternal;
    fn get_card(&self) -> CardIndex;
}
impl TaggedCardIndexT for TaggedCardIndex {
    fn construct(player: PlayerIDInternal, card_index: CardIndex) -> Self {
        card_index | (player << 6)
    }

    fn get_player(&self) -> PlayerIDInternal {
        ((self >> 6) & 0b11u8) as PlayerIDInternal
    }

    fn get_card(&self) -> CardIndex {
        self & 0x3Fu8
    }
}

pub const SEPERATOR_NEW_TRICK: u8 = 14; //After this byte, it follows a byte that contains the trick type.
pub const SEPERATOR_NEW_MOVE: u8 = 15; //Indicates that a new move follows in the current trick.

pub struct RoundLogIterator<'a> {
    round_log: &'a RoundLog,
    current_index: usize,

}
impl<'a> RoundLogIterator<'a> {
    fn start_new_trick(&mut self) -> TrickType {
        self.current_index += 1;
        let res = self.round_log.log[self.current_index];
        self.current_index += 1;
        res
    }
    fn has_move_in_trick(&self) -> bool {
        self.current_index < self.round_log.log.len() && self.round_log.log[self.current_index] != SEPERATOR_NEW_MOVE && self.round_log.log[self.current_index] != SEPERATOR_NEW_TRICK
    }
    fn next_move_in_trick(&mut self) -> Option<(PlayerIDInternal, Hand)> {
        let mut hand = 0u64;
        let mut player = None;
        while self.has_move_in_trick(){
            let t_car_index = self.round_log.log[self.current_index];
            hand |= hand!(t_car_index.get_card());
            if player.is_none() {
                player = Some(t_car_index.get_player());
            }
            assert_eq!(player.unwrap(), t_car_index.get_player());
            self.current_index += 1;
        }
        if self.current_index < self.round_log.log.len() && self.round_log.log[self.current_index] == SEPERATOR_NEW_MOVE {
            self.current_index += 1;
        }
        if hand != 0 {
            Some((player.unwrap(), hand))
        } else {
            None
        }
    }
    pub fn next_trick(&mut self) -> Option<Trick>{
        if self.current_index >= self.round_log.log.len() {
            return None;
        }
        assert_eq!(self.round_log.log[self.current_index], SEPERATOR_NEW_TRICK);
        let trick_type = self.start_new_trick();
        let mut res = Trick{trick_type, trick_log: Vec::new()};
        while let Some(_move) = self.next_move_in_trick(){
            res.trick_log.push(_move);
        }
        Some(res)

    }
}
#[derive(Debug)]
pub enum RoundLogIntegrityError {
    StartTrickIsNotNextInLine { trick_num: usize, starting_player: PlayerIDInternal, should_start: PlayerIDInternal },
    Child(usize, TrickIntegrityError),
}

#[derive(Debug)]
pub struct RoundNotFinishedPlayingError;

impl RoundLog {
    pub fn iter(&self) -> RoundLogIterator {
        RoundLogIterator { round_log: self, current_index: 0 }
    }
    pub fn integrity_check(&self, round: &Round) -> Result<(), RoundLogIntegrityError> {
        let mut player_hands = round.get_starting_hands();
        let mut prev_trick_winner: Option<PlayerIDInternal> = None;
        let mut iter = self.iter();
        let mut trick_num = 0;
        while let Some(trick) = iter.next_trick(){
            if let Some(prev) = prev_trick_winner {
                if trick.get_starting_player() != prev {
                    return Err(StartTrickIsNotNextInLine { trick_num, starting_player: trick.get_starting_player(), should_start: prev });
                }
            }
            trick.integrity_check(&mut player_hands).map_err(|x| Child(trick_num, x))?;
            let mut trick_winner = trick.get_trick_winner();
            while player_hands[trick_winner as usize] == 0u64 {
                trick_winner = (trick_winner + 1) % 4;
            }
            prev_trick_winner = Some(trick_winner);
            trick_num += 1;
        }
        Ok(())
    }
    pub fn try_fix_dragon_gifting(&mut self, round: &Round) -> Option<bool> {
        //The BSW dataset has a bug where the dragon is sometimes gifted to the player that plays it
        //or is gifted to an enemy that is no longer playing.
        //(no bombs involved). However, we can resolve the dragon gifting if there is only
        //one enemy player still playing.
        let mut player_hands = [round.player_rounds[0].final_14(), round.player_rounds[1].final_14(), round.player_rounds[2].final_14(), round.player_rounds[3].final_14()];
        let mut iter = self.iter();
        while let Some(trick)  = iter.next_trick() {
            for (player, hand) in trick.trick_log.iter() {
                player_hands[*player as usize] ^= hand;
            }
            if trick.has_to_gift_trick() {
                let gift_player = self.dragon_player_gift.unwrap();
                let winner_player = trick.get_trick_winner();
                //Dragon gifted to own team or to a player that has finished already.
                if winner_player == gift_player || winner_player == (gift_player + 2) % 4 || player_hands[gift_player as usize] == 0 {
                    if player_hands[(winner_player as usize + 1) % 4] == 0 {
                        //We can gift the dragon to (winner_player + 3) % 4
                        self.dragon_player_gift = Some((winner_player + 3) % 4);
                        return Some(true);
                    } else if player_hands[(winner_player as usize + 3) % 4] == 0 {
                        //We can gift the dragon to (winner_player + 1) % 4
                        self.dragon_player_gift = Some((winner_player + 1) % 4);
                        return Some(true);
                    } else {
                        return Some(false);
                    }
                }
            }
        }
        None
    }
    pub fn play_round(&self, round: &Round) -> Result<([Rank; 4], [Score; 4], bool), RoundNotFinishedPlayingError> { //Ranks, CardPoints, double_win
        let mut player_hands = round.get_starting_hands();
        let mut player_scores = [0; 4];
        let mut player_ranks = [RANK_4; 4];
        let mut next_rank = RANK_1;
        let mut iter = self.iter();
        while let Some(trick) = iter.next_trick(){
            let card_points = trick.played_cards().get_card_points();
            if trick.has_to_gift_trick() {
                player_scores[self.dragon_player_gift.unwrap() as usize] += card_points;
            } else {
                player_scores[trick.get_trick_winner() as usize] += card_points;
            }
            for (player, hand) in trick.trick_log.iter() {
                player_hands[*player as usize] ^= hand;
                if player_hands[*player as usize] == 0 {
                    player_ranks[*player as usize] = next_rank;
                    next_rank += 1;
                }
            }
        }
        let is_double_win = player_ranks[PLAYER_0 as usize] + player_ranks[PLAYER_2 as usize] <= RANK_1 + RANK_2
            || player_ranks[PLAYER_1 as usize] + player_ranks[PLAYER_3 as usize] <= RANK_1 + RANK_2;
        if is_double_win {
            return Ok((player_ranks, [0; 4], true));
        }
        if next_rank <= RANK_3 {
            return Err(RoundNotFinishedPlayingError);
        }
        //Else, gift card points of player 4 to first
        let first_player = player_ranks.iter().position(|x| *x == RANK_1).unwrap();
        let third_player = player_ranks.iter().position(|x| *x == RANK_3).unwrap();
        let fourth_player = player_ranks.iter().position(|x| *x == RANK_4).unwrap();
        player_scores[first_player] += player_scores[fourth_player];
        player_scores[fourth_player] = 0;
        player_scores[third_player] += player_hands[fourth_player].get_card_points();
        Ok((player_ranks, player_scores, false))
    }
    pub fn to_debug_str(&self, round: &Round) -> String {
        let player_hands = round.get_starting_hands();
        let mut res_str = String::new();
        res_str.push_str(&format!("P0: {}\n", player_hands[0].pretty_print()));
        res_str.push_str(&format!("P1: {}\n", player_hands[1].pretty_print()));
        res_str.push_str(&format!("P2: {}\n", player_hands[2].pretty_print()));
        res_str.push_str(&format!("P3: {}\n", player_hands[3].pretty_print()));
        let mut trick_num = 0;
        let mut iter = self.iter();
        while let Some(trick) = iter.next_trick() {
            res_str.push_str(&format!("Trick {} with type {}:\n", trick_num, trick.trick_type));
            for (move_idx, (player, hand)) in trick.trick_log.iter().enumerate() {
                res_str.push_str(&format!("Move {}, Player {}: {}\n", move_idx, player, hand.pretty_print()));
            }
            res_str.push_str(&format!("Trick {} trick winner: {}\n-----------------\n", trick_num, trick.get_trick_winner()));
            trick_num += 1;
        }
        res_str
    }
}