use bitcode::{Decode, Encode};
use datasize::DataSize;
use crate::bsw_binary_format::binary_format_constants::*;
use crate::bsw_binary_format::round::Round;
use crate::bsw_binary_format::round_log::RoundLog;

pub type ParsingFlagGame = u8;
pub const FLAG_CHANGED_DRAGON: ParsingFlagGame = 0b1; //Atleast one Dragon gifting id was changed in one of the rounds(they sometimes point to same team or an enemy that has already finished)
pub const FLAG_CHANGED_ROUND_SCORE: ParsingFlagGame = 0b10; //The round score has changed in at least one of the rounds compared to the BSW result.
pub const FLAG_EXCLUDED_ROUND: ParsingFlagGame = 0b100; //At least one round has been excluded compared to the BSW original dataset.
pub const FLAG_NO_WINNER_BSW: ParsingFlagGame = 0b1000; //There is no winner according to the BSW parsing.
pub const FLAG_GAME_STOPPED_WITHIN_ROUND: ParsingFlagGame = 0b10000; //Zugfolge_.csv contains a started round which was not finished playing.
pub const FLAG_CHANGED_ROUND_SCORE_WITHOUT_DRAGON: ParsingFlagGame = 0b100000; //Whenever a RoundScore was changed compared to BSW result without the dragon gift changing. These games should be looked at carefully.

#[derive(Encode, Decode, DataSize)]
pub struct Game {
    pub rounds: Vec<(Round, RoundLog)>,
    pub player_ids: [PlayerIDGlobal; 4],
    pub original_bsw_id: u32,
    pub parsing_flags: ParsingFlagGame,
}
impl Game {
    pub fn get_winner(&self) -> Option<Team> {
        let mut score_team_0: Score = 0;
        let mut score_team_1: Score = 0;
        for (round, _) in &self.rounds {
            let round_scores = round.player_rounds[PLAYER_0 as usize].round_score();
            score_team_0 += round_scores.0;
            score_team_1 += round_scores.1;
        }
        if score_team_0 > score_team_1 {
            Some(Team::Team1)
        } else if score_team_0 < score_team_1 {
            Some(Team::Team2)
        } else {
            None
        }
    }
}
