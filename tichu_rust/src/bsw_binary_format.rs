use crate::hand;
use crate::tichu_hand::*;
use bitcode::{Decode, Encode};
//TODO: Improve this mess! We can start with:
//TODO: Parse Zugfolge into RoundLog
//TODO: Check Player Ranks and RoundResult with Zugfolge
//TODO: Tests for Parser
pub const LEFT_OUT_EXCHANGE_MASK: u64 = 0x3Fu64;
pub const PARTNER_OUT_EXCHANGE_MASK: u64 = 0x3Fu64 << 6;
pub const RIGHT_OUT_EXCHANGE_MASK: u64 = 0x3Fu64 << 12;
pub const LEFT_IN_EXCHANGE_MASK: u64 = 0x3Fu64 << 18;
pub const PARTNER_IN_EXCHANGE_MASK: u64 = 0x3Fu64 << 24;
pub const RIGHT_IN_EXCHANGE_MASK: u64 = 0x3Fu64 << 30;

pub type TichuCall = u8;
pub const CALL_NONE: TichuCall = 0u8;
pub const CALL_TICHU: TichuCall = 1u8;
pub const CALL_GRAND_TICHU: TichuCall = 2u8;

pub const CALL_PLAYER_0_MASK: u64 = 0b11u64 << 36;
pub const CALL_PLAYER_1_MASK: u64 = 0b11u64 << 38;
pub const CALL_PLAYER_2_MASK: u64 = 0b11u64 << 40;
pub const CALL_PLAYER_3_MASK: u64 = 0b11u64 << 42;

pub type PlayerIDGlobal = u32;
pub type PlayerIDInternal = u8;
pub const PLAYER_0: PlayerIDInternal = 0u8; //Team 1
pub const PLAYER_1: PlayerIDInternal = 1u8; //Team 2
pub const PLAYER_2: PlayerIDInternal = 2u8; //Team 1
pub const PLAYER_3: PlayerIDInternal = 3u8; //Team 2
pub const PLAYER_ID_MASK: u64 = 0b11u64 << 44;

pub type Rank = u8;
pub const RANK_1: Rank = 0u8;
pub const RANK_2: Rank = 1u8;
pub const RANK_3: Rank = 2u8;
pub const RANK_4: Rank = 3u8;
pub const RANK_PLAYER_0_MASK: u64 = 0b11u64 << 46;
pub const RANK_PLAYER_1_MASK: u64 = 0b11u64 << 48;
pub const RANK_PLAYER_2_MASK: u64 = 0b11u64 << 50;
pub const RANK_PLAYER_3_MASK: u64 = 0b11u64 << 52;

pub type Score = i16;
pub const CARD_SCORE_MASK: u64 = 0xFFu64 << 54;
///8 Bits indicating the value of card point difference collected in the round for team 1 (0-150)
#[derive(Encode, Decode, Default)]
pub struct PlayerRoundHand {
    pub first_8: Hand,
    pub first_14: Hand,
    pub extras: u64,
}
#[derive(Encode, Decode, Default)]
pub struct Round {
    pub player_rounds: [PlayerRoundHand; 4],
}
impl Round {
    pub fn integrity_check(&self) {
        for i in 0..4 {
            self.player_rounds[i].integrity_check();
        }
        let (p0, p1, p2, p3) = (
            self.player_rounds.get(0).unwrap(),
            self.player_rounds.get(1).unwrap(),
            self.player_rounds.get(2).unwrap(),
            self.player_rounds.get(3).unwrap(),
        );
        //Check all cards are distributed. Child checks ensures its 14 each.
        assert_eq!(
            p0.first_14 | p1.first_14 | p2.first_14 | p3.first_14,
            MASK_ALL
        );
        //Check exchange cards
        assert_eq!(p0.right_out_exchange_card(), p1.left_in_exchange_card());
        assert_eq!(p1.right_out_exchange_card(), p2.left_in_exchange_card());
        assert_eq!(p2.right_out_exchange_card(), p3.left_in_exchange_card());
        assert_eq!(p3.right_out_exchange_card(), p0.left_in_exchange_card());

        assert_eq!(p0.left_out_exchange_card(), p3.right_in_exchange_card());
        assert_eq!(p3.left_out_exchange_card(), p2.right_in_exchange_card());
        assert_eq!(p2.left_out_exchange_card(), p1.right_in_exchange_card());
        assert_eq!(p1.left_in_exchange_card(), p0.right_in_exchange_card());

        assert_eq!(p0.partner_out_exchange_card(), p2.partner_in_exchange_card());
        assert_eq!(p2.partner_out_exchange_card(), p0.partner_in_exchange_card());
        assert_eq!(p1.partner_out_exchange_card(), p3.partner_in_exchange_card());
        assert_eq!(p3.partner_out_exchange_card(), p1.partner_in_exchange_card());

        assert_eq!((p0.extras >> 36) & 0xFF, (p1.extras >> 36) & 0xFF);
        assert_eq!((p1.extras >> 36) & 0xFF, (p2.extras >> 36) & 0xFF);
        assert_eq!((p2.extras >> 36) & 0xFF, (p3.extras >> 36) & 0xFF);

        assert_eq!(p0.extras >> 46, p1.extras >> 46);
        assert_eq!(p1.extras >> 46, p2.extras >> 46);
        assert_eq!(p2.extras >> 46, p3.extras >> 46);

        assert_eq!(p0.player_id(), PLAYER_0);
        assert_eq!(p1.player_id(), PLAYER_1);
        assert_eq!(p2.player_id(), PLAYER_2);
        assert_eq!(p3.player_id(), PLAYER_3);
    }
}
pub type TaggedCardIndex = u8; //Lower 6 bits are CardIndex, upper 2 CardIndex are Tag
pub type Tag = u8; //Either PlayerId or
pub const TAG_NEW_TRICK: Tag = 4;
pub const TAG_BOMB: Tag = 5;

pub trait TaggedCardndexT {
    fn get_tag(&self) -> Tag;
    fn get_card_index(&self) -> CardIndex;
}
#[derive(Encode, Decode)]
pub struct RoundLog {
    pub mahjong_wish: Option<CardIndex>,
    pub dragon_player_gift: Option<PlayerIDInternal>,
    pub log: Vec<Vec<Trick>>,
    //TODO: This and all subclcasses
}
#[derive(Encode, Decode)]
pub struct Trick {
    pub trick_log: Vec<TaggedCardIndex>,
}
pub enum Team {
    Team1,
    Team2,
}
impl Team {
    pub fn get_players(&self) -> (PlayerIDInternal, PlayerIDInternal) {
        match self {
            Team::Team1 => (PLAYER_0, PLAYER_2),
            Team::Team2 => (PLAYER_1, PLAYER_3),
        }
    }
}
#[derive(Encode, Decode)]
pub struct Game {
    pub rounds: Vec<Round>,
    pub round_logs: Vec<RoundLog>,
    pub player_ids: [PlayerIDGlobal; 4],
}
impl Game {
    pub fn get_winner(&self) -> Team {
        let mut score_team_0: Score = 0;
        let mut score_team_1: Score = 0;
        for round in &self.rounds {
            let round_scores = round.player_rounds[PLAYER_0 as usize].round_score_relative();
            score_team_0 += round_scores.0;
            score_team_1 += round_scores.1;
        }
        if score_team_0 > score_team_1 {
            Team::Team1
        } else {
            Team::Team2
        }
    }
}

impl PlayerRoundHand {
    pub fn integrity_check(&self) {
        assert_eq!(self.first_8.count_ones(), 8);
        assert_eq!(self.first_8 & MASK_ALL, self.first_8);
        assert_eq!(self.first_14.count_ones(), 14);
        assert_eq!(self.first_14 & MASK_ALL, self.first_14);
        assert_eq!(self.final_14().count_ones(), 14);
        assert_eq!(self.final_14() & MASK_ALL, self.final_14());
        assert_eq!((hand!(self.left_out_exchange_card(),self.partner_out_exchange_card(),self.right_out_exchange_card()) & self.first_14).count_ones(), 3);
        assert_eq!((hand!(self.left_in_exchange_card(),self.partner_in_exchange_card(),self.right_in_exchange_card()) & self.first_14).count_ones(), 0);
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
    pub fn round_score_relative(&self) -> (Score, Score) {
        //Reported relative to own team;
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
        let relative_score = self.round_score_relative();
        relative_score.0 - relative_score.1
    }
}
