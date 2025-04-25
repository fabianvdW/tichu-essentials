use crate::analysis::{format_slice_abs_relative, format_slice_abs_relative2, format_slice_abs_relative2_i64};
use crate::bsw_binary_format::binary_format_constants::{PlayerIDInternal, CALL_GRAND_TICHU, CALL_NONE, CALL_TICHU, RANK_1, RANK_3};
use crate::bsw_binary_format::binary_format_constants::Team::{Team1, Team2};
use crate::bsw_database::DataBase;
use crate::hand;
use crate::tichu_hand::{Hand, DRAGON};

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

    let mut cardpoints_by_category = [[[0i64; 2]; 4];2];
    let mut category_rounds = [[[0; 2]; 4];2];
    let category_descriptions = ["1. + 3.", "1. + 4.", "2. + 3.", "2. + 4."];
    for game in db.games.iter() {
        wins[0] += matches!(game.get_winner(), Some(Team1)) as usize;
        wins[1] += matches!(game.get_winner(), Some(Team2)) as usize;
        if game.rounds.len() == 0 {
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
        if pr0.is_double_win_team_1() || pr0.is_double_win_team_2() { continue; }
        if (0..4).any(|player_id| pr0.player_call(player_id) != CALL_NONE) { continue; }
        for team in 0..2 {
            let pr_a = round.player_rounds.get(team).unwrap();
            let pr_b = round.player_rounds.get(team + 2).unwrap();
            let rank1 = pr0.player_rank(team as PlayerIDInternal);
            let rank2 = pr0.player_rank((team + 2) as PlayerIDInternal);
            let team_ranks = [rank1.min(rank2), rank1.max(rank2)];
            let cardpoints_team = if team == 0 { pr0.round_score().0 } else { pr0.round_score().1 } as i64;
            let category = if team_ranks[0] == RANK_1{team_ranks[1]-RANK_3} else{2+team_ranks[1]-RANK_3} as usize;
            let dragon_in_team =((pr_a.final_14() | pr_b.final_14()) & hand!(DRAGON) != 0) as usize;
            category_rounds[dragon_in_team][category][team] += 1;
            cardpoints_by_category[dragon_in_team][category][team] += cardpoints_team;
        }
    }

    println!("Tichu successes: {}", format_slice_abs_relative2(&tichu_successes, &tichu_calls));
    println!("GTichu successes: {}", format_slice_abs_relative2(&gtichu_successes, &gtichu_calls));
    println!("Wins(Team): {}", format_slice_abs_relative(&wins, db.games.len()));
    println!("Draws: {}", format_slice_abs_relative(&[draws], db.games.len()));
    println!("DoubleWins(Team): {}", format_slice_abs_relative(&double_wins, rounds));
    for category in 0..4 {
        for dr in 0..2{
            let dragon_str = if dr == 0{"Dragon in enemy team"}else{"Dragon in Team"};
            println!("Expected card points given category {}({}): {}", category_descriptions[category], dragon_str, format_slice_abs_relative2_i64(&cardpoints_by_category[dr][category], &category_rounds[dr][category]));
        }
    }
}