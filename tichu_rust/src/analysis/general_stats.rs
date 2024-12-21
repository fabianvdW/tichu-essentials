use crate::analysis::{format_slice_abs_relative, format_slice_abs_relative2};
use crate::bsw_binary_format::binary_format_constants::{CALL_GRAND_TICHU, CALL_TICHU, RANK_1};
use crate::bsw_binary_format::binary_format_constants::Team::{Team1, Team2};
use crate::bsw_database::DataBase;
use crate::tichu_hand::Hand;

pub fn evaluate_general_stats(db: &DataBase) {
    //Evaluate bomb probability, first 8, first 14, final 14 for each player.
    let rounds = db.games.iter().fold(0, |acc, inc| acc + inc.rounds.len());
    println!("Rounds: {}", rounds);
    let tichu_calls = (0..4).map(|player_id| db.games.iter().fold(0, |acc, inc| acc + inc.rounds.iter().fold(
        0, |acc_2, inc_2| acc_2 + (inc_2.0.player_rounds[0].player_call(player_id) == CALL_TICHU) as usize,
    ))).collect::<Vec<_>>();
    let gtichu_calls = (0..4).map(|player_id| db.games.iter().fold(0, |acc, inc| acc + inc.rounds.iter().fold(
        0, |acc_2, inc_2| acc_2 + (inc_2.0.player_rounds[0].player_call(player_id) == CALL_GRAND_TICHU) as usize,
    ))).collect::<Vec<_>>();


    println!("Tichu Calls {}", format_slice_abs_relative(&tichu_calls, rounds));
    println!("Grand Tichu Calls: {}", format_slice_abs_relative(&gtichu_calls, rounds));

    //Count tichu, grand tichu success rate.
    let mut tichu_successes = [0; 4];
    let mut gtichu_successes = [0; 4];
    let draws = db.games.iter().filter(|x| x.get_winner().is_none()).count();
    let mut wins = [0; 2];
    let mut double_wins = [0; 2];
    for game in db.games.iter() {
        wins[0] += matches!(game.get_winner(), Some(Team1)) as usize;
        wins[1] += matches!(game.get_winner(), Some(Team2)) as usize;
        for (round, _) in game.rounds.iter() {
            let pr0 = round.player_rounds.get(0).unwrap();
            double_wins[0] += pr0.is_double_win_team_1() as usize;
            double_wins[1] += pr0.is_double_win_team_2() as usize;
            for player_id in 0..4 {
                tichu_successes[player_id as usize] += (pr0.player_call(player_id) == CALL_TICHU && pr0.player_rank(player_id) == RANK_1) as usize;
                gtichu_successes[player_id as usize] += (pr0.player_call(player_id) == CALL_GRAND_TICHU && pr0.player_rank(player_id) == RANK_1) as usize;
            }
        }
    }

    println!("Tichu successes: {}", format_slice_abs_relative2(&tichu_successes, &tichu_calls));
    println!("GTichu successes: {}", format_slice_abs_relative2(&gtichu_successes, &gtichu_calls));
    println!("Wins(Team): {}", format_slice_abs_relative(&wins, db.games.len()));
    println!("Draws: {}", format_slice_abs_relative(&[draws], db.games.len()));
    println!("DoubleWins(Team): {}", format_slice_abs_relative(&double_wins, rounds));
}

pub fn evaluate_general_stats_onlyr0(db: &DataBase) {
    //Evaluate bomb probability, first 8, first 14, final 14 for each player.
    let rounds = db.games.iter().fold(0, |acc, inc| acc + inc.rounds.len().min(1));

    let tichu_calls = (0..4).map(|player_id| db.games.iter().fold(0, |acc, inc| acc + if inc.rounds.len() > 0 {
        (inc.rounds[0].0.player_rounds[0].player_call(player_id) == CALL_TICHU) as usize
    } else { 0 },
    )).collect::<Vec<_>>();
    let gtichu_calls = (0..4).map(|player_id| db.games.iter().fold(0, |acc, inc| acc + if inc.rounds.len() > 0 {
        (inc.rounds[0].0.player_rounds[0].player_call(player_id) == CALL_GRAND_TICHU) as usize
    } else { 0 },
    )).collect::<Vec<_>>();


    println!("Tichu Calls {}", format_slice_abs_relative(&tichu_calls, rounds));
    println!("Grand Tichu Calls: {}", format_slice_abs_relative(&gtichu_calls, rounds));

    //Count tichu, grand tichu success rate.
    let mut tichu_successes = [0; 4];
    let mut gtichu_successes = [0; 4];
    let draws = db.games.iter().filter(|x| x.get_winner().is_none()).count();
    let mut wins = [0; 2];
    let mut double_wins = [0; 2];
    for game in db.games.iter() {
        wins[0] += matches!(game.get_winner(), Some(Team1)) as usize;
        wins[1] += matches!(game.get_winner(), Some(Team2)) as usize;
        if game.rounds.len() == 0{
            continue;
        }
        let round = &game.rounds[0].0;
        let pr0 = round.player_rounds.get(0).unwrap();
        double_wins[0] += pr0.is_double_win_team_1() as usize;
        double_wins[1] += pr0.is_double_win_team_2() as usize;
        for player_id in 0..4 {
            tichu_successes[player_id as usize] += (pr0.player_call(player_id) == CALL_TICHU && pr0.player_rank(player_id) == RANK_1) as usize;
            gtichu_successes[player_id as usize] += (pr0.player_call(player_id) == CALL_GRAND_TICHU && pr0.player_rank(player_id) == RANK_1) as usize;
        }
    }

    println!("Tichu successes: {}", format_slice_abs_relative2(&tichu_successes, &tichu_calls));
    println!("GTichu successes: {}", format_slice_abs_relative2(&gtichu_successes, &gtichu_calls));
    println!("Wins(Team): {}", format_slice_abs_relative(&wins, db.games.len()));
    println!("Draws: {}", format_slice_abs_relative(&[draws], db.games.len()));
    println!("DoubleWins(Team): {}", format_slice_abs_relative(&double_wins, rounds));
}