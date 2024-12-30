use crate::analysis::{format_slice_abs_relative, format_slice_abs_relative2_i64};
use crate::bsw_binary_format::binary_format_constants::CALL_GRAND_TICHU;
use crate::bsw_binary_format::player_round_hand::PlayerRoundHand;
use crate::bsw_database::DataBase;
use crate::tichu_hand::{CardIndex, MAHJONG, SPECIAL_CARD, PHOENIX, DRAGON, DOG};

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

    //Swap away cards
    let get_card_type = |card: CardIndex| {
        let res = card & 0b1111;
        if res == SPECIAL_CARD {
            match card {
                PHOENIX => 14,
                DRAGON => 15,
                MAHJONG => 16,
                DOG => 0,
                _ => unreachable!()
            }
        } else {
            res
        }
    };
    let mut receiv_left_cards = [[0; 4]; 17];
    let mut receiv_partner_cards = [[0; 4]; 17];
    let mut receiv_right_cards = [[0; 4]; 17];

    let mut receiv_left_cards_gt = [[0; 4]; 17];
    let mut receiv_partner_cards_gt = [[0; 4]; 17];
    let mut receiv_right_cards_gt = [[0; 4]; 17];
    let rounds = db.games.iter().fold(0, |acc, inc| acc + inc.rounds.len());
    let mut gt_rounds = [0; 4];

    for game in db.games.iter() {
        for (round, _) in game.rounds.iter() {
            for player_id in 0..4 {
                let prh = &round.player_rounds[player_id as usize];
                let receiv_left = get_card_type(prh.left_in_exchange_card());
                let receiv_partner = get_card_type(prh.partner_in_exchange_card());
                let receiv_right = get_card_type(prh.right_in_exchange_card());

                receiv_left_cards[receiv_left as usize][player_id as usize] += 1;
                receiv_partner_cards[receiv_partner as usize][player_id as usize] += 1;
                receiv_right_cards[receiv_right as usize][player_id as usize] += 1;

                if prh.player_call(player_id) == CALL_GRAND_TICHU {
                    gt_rounds[player_id as usize] += 1;
                    receiv_left_cards_gt[receiv_left as usize][player_id as usize] += 1;
                    receiv_partner_cards_gt[receiv_partner as usize][player_id as usize] += 1;
                    receiv_right_cards_gt[receiv_right as usize][player_id as usize] += 1;
                }
            }
        }
    }

    println!("Exchange card in from left: {:?}", receiv_left_cards.map(|x| x.iter().sum::<usize>() as f64 / (4. * rounds as f64)));
    println!("Exchange card in from partner: {:?}", receiv_partner_cards.map(|x| x.iter().sum::<usize>() as f64 / (4. * rounds as f64)));
    println!("Exchange card in from right: {:?}", receiv_right_cards.map(|x| x.iter().sum::<usize>() as f64 / (4. * rounds as f64)));

    println!("Exchange card in from left given GT call: {:?}", receiv_left_cards_gt.map(|x| x.iter().zip(gt_rounds.iter()).fold(
        0., |acc, inc| acc + (*inc.0 as f64) / (*inc.1 as f64)) / 4.
    ));
    println!("Exchange card in from partner given GT call: {:?}", receiv_partner_cards_gt.map(|x| x.iter().zip(gt_rounds.iter()).fold(
        0., |acc, inc| acc + (*inc.0 as f64) / (*inc.1 as f64)) / 4.
    ));println!("Exchange card in from right given GT call: {:?}", receiv_right_cards_gt.map(|x| x.iter().zip(gt_rounds.iter()).fold(
        0., |acc, inc| acc + (*inc.0 as f64) / (*inc.1 as f64)) / 4.
    ));

}
