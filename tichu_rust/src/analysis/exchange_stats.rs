use crate::analysis::{format_slice_abs_relative2_i64};
use crate::bsw_binary_format::player_round_hand::PlayerRoundHand;
use crate::bsw_database::DataBase;
use crate::tichu_hand::MAHJONG;

pub fn evaluate_exchange_stats(db: &DataBase) {
    let exchanged_mahjong_to_enemy = |prh: &PlayerRoundHand| prh.right_out_exchange_card() == MAHJONG || prh.left_out_exchange_card() == MAHJONG;
    let exchange_1_to_enemy_rounds = (0..4).map(|player_id| db.games.iter().fold(
        0, |acc, game| {
            acc + game.rounds.iter().fold(0, |acc_2, round| {
                acc_2 + exchanged_mahjong_to_enemy(&round.0.player_rounds[player_id]) as usize
            })
        })).collect::<Vec<_>>();
    let ers_exchange_1_to_enemy = (0..4).map(|player_id| db.games.iter().fold(
        0i64, |acc, game| {
            acc + game.rounds.iter().fold(0i64, |acc_2, round| {
                acc_2 + if exchanged_mahjong_to_enemy(&round.0.player_rounds[player_id]) { round.0.player_rounds[player_id].round_score_relative_gain() as i64 } else { 0 }
            })
        })).collect::<Vec<_>>();
    println!("ERS given exchange of Mahjong to enemy: {}", &format_slice_abs_relative2_i64(&ers_exchange_1_to_enemy, &exchange_1_to_enemy_rounds));
}
