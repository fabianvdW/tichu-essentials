use bitcode::{Decode, Encode};
use datasize::DataSize;
use crate::bsw_binary_format::binary_format_constants::{PlayerIDInternal, Score, PLAYER_0, PLAYER_1, PLAYER_2, PLAYER_3};
use crate::bsw_binary_format::game::ParsingFlagGame;
use crate::bsw_binary_format::player_round_hand::{PlayerRoundHand, PlayerRoundHandIntegrityError};
use crate::bsw_binary_format::round::RoundIntegrityError::{CallsMismatch, CardScoreMismatch, CardScoreTooLarge, ExchangeCardMismatch, IdMismatch, RankMismatch};
use crate::tichu_hand::{Hand, MASK_ALL};

pub type ParsingFlagRound = u8;
pub const FLAG_CHANGED_DRAGON: ParsingFlagRound = crate::bsw_binary_format::game::FLAG_CHANGED_DRAGON;
pub const FLAG_CHANGED_ROUND_SCORE: ParsingFlagRound = crate::bsw_binary_format::game::FLAG_CHANGED_ROUND_SCORE;

pub const FLAG_CHANGED_ROUND_SCORE_WITHOUT_DRAGON: ParsingFlagRound = crate::bsw_binary_format::game::FLAG_CHANGED_ROUND_SCORE_WITHOUT_DRAGON;

#[derive(Encode, Decode, Default, DataSize)]
pub struct Round {
    pub player_rounds: [PlayerRoundHand; 4],
    pub parsing_flags: ParsingFlagRound,
}
#[derive(Debug)]
pub enum RoundIntegrityError {
    Child(PlayerRoundHandIntegrityError),
    NotAllCardsDistributed(u32),
    ExchangeCardMismatch { p_out: PlayerIDInternal, p_in: PlayerIDInternal },
    CallsMismatch { p_a: PlayerIDInternal, p_b: PlayerIDInternal, calls_a: u8, calls_b: u8 },
    IdMismatch(PlayerIDInternal, u8),
    RankMismatch { p_a: PlayerIDInternal, p_b: PlayerIDInternal, ranks_a: u8, ranks_b: u8 },
    CardScoreMismatch { p_a: PlayerIDInternal, p_b: PlayerIDInternal, cardscore_a: Score, cardscore_b: Score },
    CardScoreTooLarge(Score),
}
impl Round {
    pub fn integrity_check(&self) -> Result<(), RoundIntegrityError> {
        for i in 0..4 {
            self.player_rounds[i].integrity_check().map_err(|x| RoundIntegrityError::Child(x))?;
        }
        let (p0, p1, p2, p3) = (
            self.player_rounds.get(0).unwrap(),
            self.player_rounds.get(1).unwrap(),
            self.player_rounds.get(2).unwrap(),
            self.player_rounds.get(3).unwrap(),
        );
        //Check all cards are distributed. Child checks ensures its 14 each.
        if p0.first_14 | p1.first_14 | p2.first_14 | p3.first_14 != MASK_ALL {
            return Err(RoundIntegrityError::NotAllCardsDistributed((p0.first_14 | p1.first_14 | p2.first_14 | p3.first_14).count_ones()));
        }
        //Check exchange cards
        if p0.right_out_exchange_card() != p1.left_in_exchange_card() { return Err(ExchangeCardMismatch { p_out: PLAYER_0, p_in: PLAYER_1 }); }
        if p1.right_out_exchange_card() != p2.left_in_exchange_card() { return Err(ExchangeCardMismatch { p_out: PLAYER_1, p_in: PLAYER_2 }); }
        if p2.right_out_exchange_card() != p3.left_in_exchange_card() { return Err(ExchangeCardMismatch { p_out: PLAYER_2, p_in: PLAYER_3 }); }
        if p3.right_out_exchange_card() != p0.left_in_exchange_card() { return Err(ExchangeCardMismatch { p_out: PLAYER_3, p_in: PLAYER_0 }); }

        if p0.left_out_exchange_card() != p3.right_in_exchange_card() { return Err(ExchangeCardMismatch { p_out: PLAYER_0, p_in: PLAYER_3 }); }
        if p1.left_out_exchange_card() != p0.right_in_exchange_card() { return Err(ExchangeCardMismatch { p_out: PLAYER_1, p_in: PLAYER_0 }); }
        if p2.left_out_exchange_card() != p1.right_in_exchange_card() { return Err(ExchangeCardMismatch { p_out: PLAYER_2, p_in: PLAYER_1 }); }
        if p3.left_out_exchange_card() != p2.right_in_exchange_card() { return Err(ExchangeCardMismatch { p_out: PLAYER_3, p_in: PLAYER_2 }); }

        if p0.partner_out_exchange_card() != p2.partner_in_exchange_card() { return Err(ExchangeCardMismatch { p_out: PLAYER_0, p_in: PLAYER_2 }); }
        if p1.partner_out_exchange_card() != p3.partner_in_exchange_card() { return Err(ExchangeCardMismatch { p_out: PLAYER_1, p_in: PLAYER_3 }); }
        if p2.partner_out_exchange_card() != p0.partner_in_exchange_card() { return Err(ExchangeCardMismatch { p_out: PLAYER_2, p_in: PLAYER_0 }); }
        if p3.partner_out_exchange_card() != p1.partner_in_exchange_card() { return Err(ExchangeCardMismatch { p_out: PLAYER_3, p_in: PLAYER_1 }); }


        //Check that all other fields agree on the same values
        let calls_0 = ((p0.extras >> 36) & 0xFF) as u8;
        let calls_1 = ((p1.extras >> 36) & 0xFF) as u8;
        let calls_2 = ((p2.extras >> 36) & 0xFF) as u8;
        let calls_3 = ((p3.extras >> 36) & 0xFF) as u8;
        if calls_0 != calls_1 {
            return Err(CallsMismatch { p_a: PLAYER_0, p_b: PLAYER_1, calls_a: calls_0, calls_b: calls_1 });
        }
        if calls_1 != calls_2 {
            return Err(CallsMismatch { p_a: PLAYER_1, p_b: PLAYER_2, calls_a: calls_1, calls_b: calls_2 });
        }
        if calls_2 != calls_3 {
            return Err(CallsMismatch { p_a: PLAYER_2, p_b: PLAYER_3, calls_a: calls_2, calls_b: calls_3 });
        }

        if p0.player_id() != PLAYER_0 { return Err(IdMismatch(PLAYER_0, p0.player_id())); }
        if p1.player_id() != PLAYER_1 { return Err(IdMismatch(PLAYER_1, p1.player_id())); }
        if p2.player_id() != PLAYER_2 { return Err(IdMismatch(PLAYER_2, p2.player_id())); }
        if p3.player_id() != PLAYER_3 { return Err(IdMismatch(PLAYER_3, p3.player_id())); }


        let ranks_0 = ((p0.extras >> 46) & 0xFF) as u8;
        let ranks_1 = ((p1.extras >> 46) & 0xFF) as u8;
        let ranks_2 = ((p2.extras >> 46) & 0xFF) as u8;
        let ranks_3 = ((p3.extras >> 46) & 0xFF) as u8;
        if ranks_0 != ranks_1 {
            return Err(RankMismatch { p_a: PLAYER_0, p_b: PLAYER_1, ranks_a: ranks_0, ranks_b: ranks_1 });
        }
        if ranks_1 != ranks_2 {
            return Err(RankMismatch { p_a: PLAYER_1, p_b: PLAYER_2, ranks_a: ranks_1, ranks_b: ranks_2 });
        }
        if ranks_2 != ranks_3 {
            return Err(RankMismatch { p_a: PLAYER_2, p_b: PLAYER_3, ranks_a: ranks_2, ranks_b: ranks_3 });
        }

        let cardscore_0 = (p0.extras >> 54) as Score;
        let cardscore_1 = (p1.extras >> 54) as Score;
        let cardscore_2 = (p2.extras >> 54) as Score;
        let cardscore_3 = (p3.extras >> 54) as Score;
        if cardscore_0 != cardscore_1 {
            return Err(CardScoreMismatch { p_a: PLAYER_0, p_b: PLAYER_1, cardscore_a: cardscore_0, cardscore_b: cardscore_1 });
        }
        if cardscore_1 != cardscore_2 {
            return Err(CardScoreMismatch { p_a: PLAYER_1, p_b: PLAYER_2, cardscore_a: cardscore_1, cardscore_b: cardscore_2 });
        }
        if cardscore_2 != cardscore_3 {
            return Err(CardScoreMismatch { p_a: PLAYER_2, p_b: PLAYER_3, cardscore_a: cardscore_2, cardscore_b: cardscore_3 });
        }
        if cardscore_0 > 150 {
            return Err(CardScoreTooLarge(cardscore_0));
        }

        Ok(())
    }

    pub fn get_starting_hands(&self) -> [Hand;4]{
        [self.player_rounds[0].final_14(), self.player_rounds[1].final_14(), self.player_rounds[2].final_14(), self.player_rounds[3].final_14()]
    }
}