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

pub const TEAMMATE_PLAYERS: [PlayerIDInternal; 4] = [PLAYER_2, PLAYER_3, PLAYER_0, PLAYER_1];

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
pub const CARD_SCORE_MASK: u64 = 0xFFu64 << 54; //8 Bits indicating the value of card point difference collected in the round for team 1 (0-150)

#[derive(PartialEq)]
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