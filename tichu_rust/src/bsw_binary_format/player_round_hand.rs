use bitcode::{Decode, Encode};
use datasize::DataSize;
use crate::bsw_binary_format::binary_format_constants::*;
use crate::hand;
use crate::tichu_hand::{CardIndex, Hand, MASK_ALL};
use self::PlayerRoundHandIntegrityError::*;

#[derive(Encode, Decode, Default, Debug, DataSize)]
pub struct PlayerRoundHand {
    pub first_8: Hand,
    pub first_14: Hand,
    pub extras: u64,
}

#[derive(Debug)]
pub enum PlayerRoundHandIntegrityError {
    First8Count(u32),
    First8Invalid,
    First14Count(u32),
    First14Invalid,
    Final14Count(u32),
    Final14Invalid,
    OutExchangeCardsNotInFirst14,
    InExchangeCardsInFirst14,
}

impl PlayerRoundHand {
    pub fn integrity_check(&self) -> Result<(), PlayerRoundHandIntegrityError> {
        if self.first_8.count_ones() != 8 { return Err(First8Count(self.first_8.count_ones())); }
        if self.first_8 & MASK_ALL != self.first_8 { return Err(First8Invalid); }
        if self.first_14.count_ones() != 14 { return Err(First14Count(self.first_14.count_ones())); }
        if self.first_14 & MASK_ALL != self.first_14 { return Err(First14Invalid); }
        if self.final_14().count_ones() != 14 { return Err(Final14Count(self.final_14().count_ones())); }
        if self.final_14() & MASK_ALL != self.final_14() { return Err(Final14Invalid); }
        if (hand!(self.left_out_exchange_card(),self.partner_out_exchange_card(),self.right_out_exchange_card()) & self.first_14).count_ones() != 3 {
            return Err(OutExchangeCardsNotInFirst14);
        }
        if (hand!(self.left_in_exchange_card(),self.partner_in_exchange_card(),self.right_in_exchange_card()) & self.first_14).count_ones() != 0 {
            return Err(InExchangeCardsInFirst14);
        }
        Ok(())
    }
    pub fn left_out_exchange_card(&self) -> CardIndex {
        (self.extras & LEFT_OUT_EXCHANGE_MASK) as CardIndex
    }
    pub fn partner_out_exchange_card(&self) -> CardIndex {
        ((self.extras & PARTNER_OUT_EXCHANGE_MASK) >> 6) as CardIndex
    }
    pub fn right_out_exchange_card(&self) -> CardIndex {
        ((self.extras & RIGHT_OUT_EXCHANGE_MASK) >> 12) as CardIndex
    }

    pub fn left_in_exchange_card(&self) -> CardIndex {
        ((self.extras & LEFT_IN_EXCHANGE_MASK) >> 18) as CardIndex
    }
    pub fn partner_in_exchange_card(&self) -> CardIndex {
        ((self.extras & PARTNER_IN_EXCHANGE_MASK) >> 24) as CardIndex
    }
    pub fn right_in_exchange_card(&self) -> CardIndex {
        ((self.extras & RIGHT_IN_EXCHANGE_MASK) >> 30) as CardIndex
    }

    pub fn final_14(&self) -> Hand {
        //println!("LEX:{}", self.left_out_exchange_card());
        self.first_14
            ^ hand!(
                self.left_out_exchange_card(),
                self.partner_out_exchange_card(),
                self.right_out_exchange_card(),
                self.left_in_exchange_card(),
                self.partner_in_exchange_card(),
                self.right_in_exchange_card()
            )
    }
    pub fn player_call(&self, player_id: PlayerIDInternal) -> TichuCall {
        ((self.extras >> (36 + 2 * player_id)) & 0b11u64) as TichuCall
    }
    pub fn player_0_call(&self) -> TichuCall {
        ((self.extras & CALL_PLAYER_0_MASK) >> 36) as TichuCall
    }
    pub fn player_1_call(&self) -> TichuCall {
        ((self.extras & CALL_PLAYER_1_MASK) >> 38) as TichuCall
    }
    pub fn player_2_call(&self) -> TichuCall {
        ((self.extras & CALL_PLAYER_2_MASK) >> 40) as TichuCall
    }
    pub fn player_3_call(&self) -> TichuCall {
        ((self.extras & CALL_PLAYER_3_MASK) >> 42) as TichuCall
    }
    pub fn player_id(&self) -> PlayerIDInternal {
        ((self.extras & PLAYER_ID_MASK) >> 44) as PlayerIDInternal
    }
    pub fn player_rank(&self, player_id: PlayerIDInternal) -> Rank {
        (self.extras >> (46 + 2 * player_id) & 0b11u64) as Rank
    }
    pub fn player_0_rank(&self) -> Rank {
        ((self.extras & RANK_PLAYER_0_MASK) >> 46) as Rank
    }
    pub fn player_1_rank(&self) -> Rank {
        ((self.extras & RANK_PLAYER_1_MASK) >> 48) as Rank
    }
    pub fn player_2_rank(&self) -> Rank {
        ((self.extras & RANK_PLAYER_2_MASK) >> 50) as Rank
    }
    pub fn player_3_rank(&self) -> Rank {
        ((self.extras & RANK_PLAYER_3_MASK) >> 52) as Rank
    }

    pub fn is_double_win_team_1(&self) -> bool {
        self.player_0_rank() + self.player_2_rank() <= RANK_1 + RANK_2
    }
    pub fn is_double_win_team_2(&self) -> bool {
        self.player_1_rank() + self.player_3_rank() <= RANK_1 + RANK_2
    }
    pub fn round_score(&self) -> (Score, Score) {
        //Reported absolute to Team1;
        let mut score_team_1: Score = 0;
        let mut score_team_2: Score = 0;
        score_team_1 += self.player_0_call() as Score * 100 * {
            if self.player_0_rank() == RANK_1 {
                1
            } else {
                -1
            }
        };
        score_team_1 += self.player_2_call() as Score * 100 * {
            if self.player_2_rank() == RANK_1 {
                1
            } else {
                -1
            }
        };
        score_team_2 += self.player_1_call() as Score * 100 * {
            if self.player_1_rank() == RANK_1 {
                1
            } else {
                -1
            }
        };
        score_team_2 += self.player_3_call() as Score * 100 * {
            if self.player_3_rank() == RANK_1 {
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
            let card_score: Score = ((self.extras & CARD_SCORE_MASK) >> 54) as Score;
            score_team_1 += card_score - 25;
            score_team_2 += 125 - card_score;
        }
        (score_team_1, score_team_2)
    }
    pub fn round_score_relative_gain(&self) -> Score {
        let round_score = self.round_score();
        if self.player_id() == PLAYER_0 || self.player_id() == PLAYER_2 {
            round_score.0 - round_score.1
        } else {
            round_score.1 - round_score.0
        }
    }
}
