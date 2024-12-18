use crate::bsw_database::DataBase;
use crate::bsw_binary_format::game;
use crate::bsw_binary_format::round;

pub fn evaluate_parsing_stats(db: &DataBase) {
    let mut dragon_changes: usize = 0;
    let mut round_result_changes: usize = 0;
    let mut round_result_changes_without_dragon: usize = 0;
    let mut round_result_changes_with_dragon: usize = 0;
    let mut affected_games = 0;
    for game in db.games.iter() {
        for (round, _) in game.rounds.iter() {
            dragon_changes += (round.parsing_flags & round::FLAG_CHANGED_DRAGON != 0) as usize;
            round_result_changes += (round.parsing_flags & round::FLAG_CHANGED_ROUND_SCORE != 0) as usize;
            round_result_changes_without_dragon += (round.parsing_flags & round::FLAG_CHANGED_ROUND_SCORE_WITHOUT_DRAGON != 0) as usize;
            round_result_changes_with_dragon += (round.parsing_flags & round::FLAG_CHANGED_DRAGON != 0
                && round.parsing_flags & round::FLAG_CHANGED_ROUND_SCORE != 0) as usize;
        }
        affected_games += (game.parsing_flags & (game::FLAG_CHANGED_ROUND_SCORE | game::FLAG_EXCLUDED_ROUND) != 0) as usize;
    }
    println!("Dragon changes: {}", dragon_changes);
    println!("Round-Result changes: {}", round_result_changes);
    println!("Round-Result changes without dragon: {}", round_result_changes_without_dragon);
    println!("Round-Result changes with dragon: {}", round_result_changes_with_dragon);
    println!("Affected games: {}", affected_games);
}