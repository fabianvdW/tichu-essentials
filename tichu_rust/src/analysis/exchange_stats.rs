use crate::analysis::{format_slice_abs_relative, format_slice_abs_relative2, format_slice_abs_relative2_i64};
use crate::bsw_binary_format::binary_format_constants::{PlayerIDInternal, CALL_GRAND_TICHU, RANK_1};
use crate::bsw_binary_format::player_round_hand::PlayerRoundHand;
use crate::bsw_database::DataBase;
use crate::{hand, tichu_hand};
use crate::tichu_hand::{Hand, CardIndex, MAHJONG, SPECIAL_CARD, PHOENIX, DRAGON, DOG, ACE, MASK_ACES, TichuHand};

pub fn get_exchange_card_type(card: CardIndex) -> u8 {
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
}
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

    let exchanged_dog_to_enemy = |prh: &PlayerRoundHand| prh.right_out_exchange_card() == DOG || prh.left_out_exchange_card() == DOG;
    let exchange_dog_to_enemy_rounds = (0..4).map(|player_id| db.games.iter().fold(
        0, |acc, game| {
            acc + game.rounds.iter().fold(0, |acc_2, round| {
                acc_2 + exchanged_dog_to_enemy(&round.0.player_rounds[player_id]) as usize
            })
        })).collect::<Vec<_>>();
    let ers_exchange_dog_to_enemy = (0..4).map(|player_id| db.games.iter().fold(
        0i64, |acc, game| {
            acc + game.rounds.iter().fold(0i64, |acc_2, round| {
                acc_2 + if exchanged_dog_to_enemy(&round.0.player_rounds[player_id]) { round.0.player_rounds[player_id].round_score_relative_gain() as i64 } else { 0 }
            })
        })).collect::<Vec<_>>();
    println!("ERS given exchange of Dog to enemy: {}", &format_slice_abs_relative2_i64(&ers_exchange_dog_to_enemy, &exchange_dog_to_enemy_rounds));

    //Dog+GT, Dog+Double Wins
    let mut ers_gt_rounds_no_dog_first14_dogfinal14 = [0; 4];
    let mut gt_successes_no_dog_first14_dogfinal14 = [0; 4];
    let mut double_wins_gt_rounds_no_dog_first14_dogfinal14 = [0; 4];
    let mut gt_rounds_no_dog_first14_dogfinal14 = [0; 4];
    let mut gt_rounds_no_dog_first14 = [0; 4];

    let mut ers_gt_rounds = [0; 4];
    let mut gt_rounds = [0; 4];
    let mut double_wins_gt_rounds = [0; 4];
    let mut gt_rounds_dog_to_partner_if_spawn = [0; 4];
    let mut gt_rounds_caller_spawns_dog = [0; 4];
    let mut double_wins_gt_rounds_caller_spawns_dog = [0; 4];

    let mut double_wins_rounds_dogfromenemy = [0; 4];
    let mut rounds_dogfromenemy = [0; 4];

    let mut double_wins = [0; 4];
    let rounds = db.games.iter().fold(0, |acc, inc| acc + inc.rounds.len());

    for game in db.games.iter() {
        for (round, _) in game.rounds.iter() {
            for player_id in 0..4 {
                let prh = &round.player_rounds[player_id];
                let is_double_win = if player_id % 2 == 0 { prh.is_double_win_team_1() } else { prh.is_double_win_team_2() };
                let is_gt_call = prh.player_call(player_id as PlayerIDInternal) == CALL_GRAND_TICHU;
                let no_dog_first14 = prh.first_14 & hand!(DOG) == 0;
                let dog_to_partner = prh.partner_out_exchange_card() == DOG;
                let dog_final14 = prh.final_14() & hand!(DOG) != 0;
                let dog_from_enemy = prh.left_in_exchange_card() == DOG || prh.right_in_exchange_card() == DOG;
                let round_score_diff = prh.round_score_relative_gain();
                let gt_success = is_gt_call && prh.player_rank(player_id as PlayerIDInternal) == RANK_1;

                double_wins[player_id] += is_double_win as usize;
                rounds_dogfromenemy[player_id] += dog_from_enemy as usize;
                double_wins_rounds_dogfromenemy[player_id] += (dog_from_enemy & is_double_win) as usize;

                double_wins_gt_rounds[player_id] += (is_double_win & is_gt_call) as usize;
                double_wins_gt_rounds_caller_spawns_dog[player_id] += (is_gt_call & !no_dog_first14 & is_double_win) as usize;
                gt_rounds_dog_to_partner_if_spawn[player_id] += (is_gt_call & dog_to_partner) as usize;
                gt_rounds_caller_spawns_dog[player_id] += (is_gt_call & !no_dog_first14) as usize;
                gt_rounds[player_id] += is_gt_call as usize;
                ers_gt_rounds[player_id] += is_gt_call as i64 * round_score_diff as i64;

                gt_rounds_no_dog_first14[player_id] += (is_gt_call & no_dog_first14) as usize;
                gt_rounds_no_dog_first14_dogfinal14[player_id] += (is_gt_call & no_dog_first14 & dog_final14) as usize;
                double_wins_gt_rounds_no_dog_first14_dogfinal14[player_id] += (is_gt_call & no_dog_first14 & dog_final14 & is_double_win) as usize;
                gt_successes_no_dog_first14_dogfinal14[player_id] += (no_dog_first14 & dog_final14 & gt_success) as usize;
                ers_gt_rounds_no_dog_first14_dogfinal14[player_id] += (is_gt_call & no_dog_first14 & dog_final14) as i64 * round_score_diff as i64;
            }
        }
    }

    println!("Double Wins: {}", format_slice_abs_relative(&double_wins, rounds));
    println!("Double Wins if dog from enemy: {}", format_slice_abs_relative2(&double_wins_rounds_dogfromenemy, &rounds_dogfromenemy));
    println!("Double Wins if GT call: {}", format_slice_abs_relative2(&double_wins_gt_rounds, &gt_rounds));
    println!("Double Wins if GT call & Caller gets dog: {}", format_slice_abs_relative2(&double_wins_gt_rounds_no_dog_first14_dogfinal14, &gt_rounds_no_dog_first14_dogfinal14));
    println!("Double Wins if GT call & Caller spawns with dog: {}", format_slice_abs_relative2(&double_wins_gt_rounds_caller_spawns_dog, &gt_rounds_caller_spawns_dog));
    println!("Pr of exch Dog to Partner if GT call & Caller Spawns with dog: {} ", format_slice_abs_relative2(&gt_rounds_dog_to_partner_if_spawn, &gt_rounds_caller_spawns_dog));

    println!("ERS if GT call: {}", format_slice_abs_relative2_i64(&ers_gt_rounds, &gt_rounds));
    println!("ERS if GT call & Caller gets dog: {}", format_slice_abs_relative2_i64(&ers_gt_rounds_no_dog_first14_dogfinal14, &gt_rounds_no_dog_first14_dogfinal14));

    println!("Pr of  Caller gets dog given Caller has no dog in first14: {}", format_slice_abs_relative2(&gt_rounds_no_dog_first14_dogfinal14, &gt_rounds_no_dog_first14));
    println!("Pr of  GT Success if Caller gets dog: {}", format_slice_abs_relative2(&gt_successes_no_dog_first14_dogfinal14, &gt_rounds_no_dog_first14_dogfinal14));


    //Swap away cards
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
                let receiv_left = get_exchange_card_type(prh.left_in_exchange_card());
                let receiv_partner = get_exchange_card_type(prh.partner_in_exchange_card());
                let receiv_right = get_exchange_card_type(prh.right_in_exchange_card());

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
    ));
    println!("Exchange card in from right given GT call: {:?}", receiv_right_cards_gt.map(|x| x.iter().zip(gt_rounds.iter()).fold(
        0., |acc, inc| acc + (*inc.0 as f64) / (*inc.1 as f64)) / 4.
    ));

    //Swapping away Aces...
    let mut swap_ace_to_partner_no_gt = [0; 4];
    let mut no_gt_rounds = [0; 4];

    let mut rounds_ace_only_high_card_no_gt = [0; 4];
    let mut swap_ace_to_partner_only_high_card_no_gt = [0; 4];

    let mut swap_ace_to_partner_with_two_aces_no_gt = [0; 4];
    for game in db.games.iter() {
        for (round, _) in game.rounds.iter() {
            for player_id in 0..4 {
                let prh = &round.player_rounds[player_id as usize];
                if prh.player_call((player_id + 2) % 4) == CALL_GRAND_TICHU {
                    continue;
                }
                let ace_to_partner = tichu_hand::get_card_type(prh.partner_out_exchange_card()) == ACE;
                let two_aces = (prh.first_14 & MASK_ACES).count_ones() >= 2;
                let ace_only_high_card = prh.first_14.get_high_card_amt() == 1 && prh.first_14 & MASK_ACES != 0;

                no_gt_rounds[player_id as usize] += 1;
                swap_ace_to_partner_no_gt[player_id as usize] += ace_to_partner as usize;

                swap_ace_to_partner_with_two_aces_no_gt[player_id as usize] += (ace_to_partner & two_aces) as usize;

                rounds_ace_only_high_card_no_gt[player_id as usize] += ace_only_high_card as usize;
                swap_ace_to_partner_only_high_card_no_gt[player_id as usize] += (ace_only_high_card & ace_to_partner) as usize;
            }
        }
    }

    println!("Pr of giving Partner Ace given no GT call: {}", format_slice_abs_relative2(&swap_ace_to_partner_no_gt, &no_gt_rounds));
    println!("Pr of giving Partner Ace given no GT call & only high card: {}", format_slice_abs_relative2(&swap_ace_to_partner_only_high_card_no_gt, &rounds_ace_only_high_card_no_gt));
    println!("Pr of having two Aces given Partner has received Ace from me & no gt call: {}", format_slice_abs_relative2(&swap_ace_to_partner_with_two_aces_no_gt, &swap_ace_to_partner_no_gt));
}
