use std::fmt::{Debug, Formatter};
use bitcode::{Decode, Encode};
use datasize::DataSize;
use crate::tichu_hand::*;
use crate::bsw_binary_format::binary_format_constants::*;
use crate::bsw_binary_format::{trick::Trick, round::Round};
use crate::bsw_binary_format::trick::TrickIntegrityError;

#[derive(Encode, Decode, Default, DataSize)]
pub struct RoundLog {
    pub mahjong_wish: Option<CardIndex>,
    pub dragon_player_gift: Option<PlayerIDInternal>,
    pub log: Vec<Trick>,
}
#[derive(Debug)]
pub enum RoundLogIntegrityError{
    StartTrickIsNotNextInLine{trick_num: usize, starting_player: PlayerIDInternal, should_start: PlayerIDInternal},
    Child(usize, TrickIntegrityError),
}
#[derive(Debug)]
pub struct RoundNotFinishedPlayingError;

impl RoundLog {
    pub fn integrity_check(&self, round: &Round) -> Result<(), RoundLogIntegrityError>{
        let mut player_hands = round.get_starting_hands();
        let mut prev_player: Option<PlayerIDInternal> = None;
        for (trick_num, trick) in self.log.iter().enumerate() {
            if let Some(prev) = prev_player {
                if trick.get_starting_player() != prev {
                    return Err(RoundLogIntegrityError::StartTrickIsNotNextInLine {trick_num, starting_player: trick.get_starting_player(), should_start: prev});
                }
            }
            trick.integrity_check(&mut player_hands).map_err(|x| RoundLogIntegrityError::Child(trick_num, x))?;
            let mut trick_winner = trick.get_trick_winner();
            while player_hands[trick_winner as usize] == 0u64 {
                trick_winner = (trick_winner + 1) % 4;
            }
            prev_player = Some(trick_winner);
        }
        Ok(())
    }
    pub fn try_fix_dragon_gifting(&mut self, round: &Round) -> Option<bool> {
        //The BSW dataset has a bug where the dragon is sometimes gifted to the player that plays it
        //or is gifted to an enemy that is no longer playing.
        //(no bombs involved). However, we can resolve the dragon gifting if there is only
        //one enemy player still playing.
        let mut player_hands = [round.player_rounds[0].final_14(), round.player_rounds[1].final_14(), round.player_rounds[2].final_14(), round.player_rounds[3].final_14()];
        for trick in self.log.iter() {
            for (player, hand) in trick.iter() {
                player_hands[player as usize] ^= hand;
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
        for trick in self.log.iter() {
            let card_points = trick.played_cards().get_card_points();
            if trick.has_to_gift_trick() {
                player_scores[self.dragon_player_gift.unwrap() as usize] += card_points;
            } else {
                player_scores[trick.get_trick_winner() as usize] += card_points;
            }
            for (player, hand) in trick.iter() {
                player_hands[player as usize] ^= hand;
                if player_hands[player as usize] == 0 {
                    player_ranks[player as usize] = next_rank;
                    next_rank += 1;
                }
            }
        }
        let is_double_win = player_ranks[PLAYER_0 as usize] + player_ranks[PLAYER_2 as usize] <= RANK_1 + RANK_2
            || player_ranks[PLAYER_1 as usize] + player_ranks[PLAYER_3 as usize] <= RANK_1 + RANK_2;
        if is_double_win {
            return Ok((player_ranks, [0; 4], true));
        }
        if next_rank <= RANK_3{
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
        for (trick_num, trick) in self.log.iter().enumerate() {
            res_str.push_str(&format!("Trick {} with type {}:\n", trick_num, trick.trick_type));
            for (move_idx, (player, hand)) in trick.iter().enumerate() {
                res_str.push_str(&format!("Move {}, Player {}: {}\n", move_idx, player, hand.pretty_print()));
            }
            res_str.push_str(&format!("Trick {} trick winner: {}\n-----------------\n", trick_num, trick.get_trick_winner()));
        }
        res_str
    }
}