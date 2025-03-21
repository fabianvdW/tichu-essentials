use crate::analysis::{format_slice_abs_relative, format_slice_abs_relative2, format_slice_abs_relative2_i64};
use crate::bsw_binary_format::binary_format_constants::{PlayerIDInternal, CALL_GRAND_TICHU, CALL_NONE, CALL_TICHU, RANK_1};
use crate::bsw_binary_format::player_round_hand::PlayerRoundHand;
use crate::bsw_database::DataBase;
use crate::hand;
use crate::tichu_hand::{get_card_type, CardIndex, CardType, Hand, HandType, TichuHand, MASK_FOUR_OF_KIND, SPECIAL_CARD, TWO};

pub fn evaluate_bombs_in_play(db: &DataBase){
    let mut bombs_played: usize = 0;
    let mut four_bombs: usize = 0;
    let mut straight_bombs: usize = 0;
    for game in db.games.iter(){
        for (_, round_log) in game.rounds.iter(){
            let mut iter = round_log.iter();
            while let Some(trick) = iter.next_trick(){
                for (_, hand) in trick.trick_log.iter(){
                    let hand_type = hand.hand_type().unwrap();
                    match hand_type{
                        HandType::Bomb4(_) => {bombs_played +=1; four_bombs += 1;},
                        HandType::BombStreet(_, _)=>{bombs_played += 1; straight_bombs += 1;},
                        _=>{}
                    }
                }
            }
        }
    }
    println!("Bombs played: {}", bombs_played);
    println!("Four of kind bombs: {}", format_slice_abs_relative(&[four_bombs], bombs_played));
    println!("Straight bombs: {}", format_slice_abs_relative(&[straight_bombs], bombs_played));

}
pub fn evaluate_bomb_stats(db: &DataBase) {
    //Evaluate bomb probability, first 8, first 14, final 14 for each player.
    let rounds = db.games.iter().fold(0, |acc, inc| acc + inc.rounds.len());
    let contains_4_bomb = |hand: Hand| hand.contains_four_of_kind_bomb() as usize;
    let contains_straight_bomb = |hand: Hand| hand.contains_straight_bomb() as usize;
    let contains_bomb = |hand: Hand| (hand.contains_straight_bomb() || hand.contains_four_of_kind_bomb()) as usize;
    let bombs_first_8 = (0..4).map(|player_id| db.games.iter().fold(0, |acc, inc| acc + inc.rounds.iter().fold(
        0, |acc_2, inc_2| acc_2 + contains_bomb(inc_2.0.player_rounds[player_id].first_8),
    ))).collect::<Vec<_>>();
    let bombs_first_14 = (0..4).map(|player_id| db.games.iter().fold(0, |acc, inc| acc + inc.rounds.iter().fold(
        0, |acc_2, inc_2| acc_2 + contains_bomb(inc_2.0.player_rounds[player_id].first_14),
    ))).collect::<Vec<_>>();
    let bombs_final_14 = (0..4).map(|player_id| db.games.iter().fold(0, |acc, inc| acc + inc.rounds.iter().fold(
        0, |acc_2, inc_2| acc_2 + contains_bomb(inc_2.0.player_rounds[player_id].final_14()),
    ))).collect::<Vec<_>>();
    let four_bombs_final_14 = (0..4).map(|player_id| db.games.iter().fold(0, |acc, inc| acc + inc.rounds.iter().fold(
        0, |acc_2, inc_2| acc_2 + contains_4_bomb(inc_2.0.player_rounds[player_id].final_14()),
    ))).collect::<Vec<_>>();
    let straight_bombs_final_14 = (0..4).map(|player_id| db.games.iter().fold(0, |acc, inc| acc + inc.rounds.iter().fold(
        0, |acc_2, inc_2| acc_2 + contains_straight_bomb(inc_2.0.player_rounds[player_id].final_14()),
    ))).collect::<Vec<_>>();
    println!("Bombs under first 8. {}", format_slice_abs_relative(&bombs_first_8, rounds));
    println!("Bombs under first 14: {}", format_slice_abs_relative(&bombs_first_14, rounds));
    println!("Bombs under final 14: {}", format_slice_abs_relative(&bombs_final_14, rounds));
    println!("4-Bombs under final 14: {}", format_slice_abs_relative(&four_bombs_final_14, rounds));
    println!("Straight-Bombs under final 14: {}", format_slice_abs_relative(&straight_bombs_final_14, rounds));

    //Probability of bomb in team?
    let bombs_team_rounds = (0..2).map(|team_id| db.games.iter().fold(0, |acc, inc| acc + inc.rounds.iter().fold(
        0, |acc_2, inc_2| acc_2 +
            (contains_bomb(inc_2.0.player_rounds[team_id].final_14()) + contains_bomb(inc_2.0.player_rounds[team_id + 2].final_14())).min(1),
    ))).collect::<Vec<_>>();
    println!("Bomb in team: {}", format_slice_abs_relative(&bombs_team_rounds, rounds));

    //Probability of bomb in round?
    let bombs_round = db.games.iter().fold(0, |acc, inc| acc + inc.rounds.iter().fold(
        0, |acc_2, inc_2| acc_2 +
            (contains_bomb(inc_2.0.player_rounds[0].final_14())
                + contains_bomb(inc_2.0.player_rounds[1].final_14())
                + contains_bomb(inc_2.0.player_rounds[2].final_14())
                + contains_bomb(inc_2.0.player_rounds[3].final_14())
            ).min(1),
    ));
    println!("Bomb in round: {}", format_slice_abs_relative(&[bombs_round], rounds));

    //Probability of bomb in opp when calling or having bomb, also Expected round score differences when a bomb is in the team.
    //Probability of own bomb when calling
    let mut bombs_self_when_tcall = [0; 4];
    let mut bombs_self_when_gtcall = [0; 4];
    let mut bombs_self_when_tcall_two_hc_or_less = [0; 4];
    let mut bombs_opp_when_call_no_bomb = [0; 4];
    let mut bombs_opp_when_call = [0; 4];
    let mut bombs_opp_when_bomb = [0; 4];

    let mut call_rounds = [0; 4];
    let mut gtcall_rounds = [0; 4];
    let mut gtcall_and_enemy_bomb_rounds = [0; 4];
    let mut gtcall_and_enemy_bomb_successes = [0; 4];
    let mut tcall_rounds = [0; 4];
    let mut tcall_and_bomb_rounds = [0; 4];
    let mut tcall_and_bomb_successes = [0; 4];
    let mut tcall_and_enemy_bomb_rounds = [0; 4];
    let mut tcall_and_enemy_bomb_successes = [0; 4];
    let mut call_rounds_no_bomb = [0; 4];
    let mut tcall_rounds_two_hc_or_less = [0; 4];

    //Probability of bomb given triplet
    let mut triplet_rounds = [[0; 4]; 5]; //Outer array: triplet count, inner: player_id
    let mut bomb_self_given_triplet = [[0; 4]; 5]; //Outer array: triplet count, inner: player_id
    let mut bomb_opp_given_triplet = [[0; 4]; 5]; //OUter array: triplet count, inner: player_id

    let mut triplet_post_exch_rounds = [[0; 4]; 4];
    let mut bomb_opp_given_post_exch_triplet = [[0; 4]; 4]; //OUter array: triplet count, inner: player_id

    let mut round_score_diff_given_bomb = [0; 2];
    for game in db.games.iter() {
        for (round, _) in game.rounds.iter() {
            let p0_bombs = contains_bomb(round.player_rounds[0].final_14());
            let p1_bombs = contains_bomb(round.player_rounds[1].final_14());
            let p2_bombs = contains_bomb(round.player_rounds[2].final_14());
            let p3_bombs = contains_bomb(round.player_rounds[3].final_14());
            let p_bombs = [p0_bombs, p1_bombs, p2_bombs, p3_bombs];
            let bombs_team_1 = (p0_bombs + p2_bombs).min(1);
            let bombs_team_2 = (p1_bombs + p3_bombs).min(1);
            let bombs = [bombs_team_1, bombs_team_2];
            for player_id in 0..4 {
                let call = round.player_rounds[0].player_call(player_id);
                if call == CALL_NONE {
                    continue;
                }
                call_rounds[player_id as usize] += 1;
                if p_bombs[player_id as usize] == 0 {
                    call_rounds_no_bomb[player_id as usize] += 1;
                    bombs_opp_when_call_no_bomb[player_id as usize] += bombs[((player_id + 1) % 2) as usize];
                }
                bombs_opp_when_call[player_id as usize] += bombs[((player_id + 1) % 2) as usize];
                if call == CALL_TICHU {
                    tcall_rounds[player_id as usize] += 1;
                    bombs_self_when_tcall[player_id as usize] += bombs[(player_id % 2) as usize];
                    if p_bombs[player_id as usize] == 1 {
                        tcall_and_bomb_rounds[player_id as usize] += 1;
                        tcall_and_bomb_successes[player_id as usize] += (round.player_rounds[0].player_rank(player_id) == RANK_1) as usize;
                    }
                    if bombs[((player_id + 1) % 2) as usize] == 1 {
                        tcall_and_enemy_bomb_rounds[player_id as usize] += 1;
                        tcall_and_enemy_bomb_successes[player_id as usize] += (round.player_rounds[0].player_rank(player_id) == RANK_1) as usize;
                    }
                }
                if call == CALL_TICHU && round.player_rounds[player_id as usize].final_14().get_high_card_amt() <= 2 {
                    tcall_rounds_two_hc_or_less[player_id as usize] += 1;
                    bombs_self_when_tcall_two_hc_or_less[player_id as usize] += bombs[(player_id % 2) as usize];
                }
                if call == CALL_GRAND_TICHU {
                    gtcall_rounds[player_id as usize] += 1;
                    bombs_self_when_gtcall[player_id as usize] += bombs[(player_id % 2) as usize];
                    if bombs[((player_id + 1) % 2) as usize] == 1 {
                        gtcall_and_enemy_bomb_rounds[player_id as usize] += 1;
                        gtcall_and_enemy_bomb_successes[player_id as usize] += (round.player_rounds[0].player_rank(player_id) == RANK_1) as usize;
                    }
                }
                //Triplets
                if contains_bomb(round.player_rounds[player_id as usize].first_14) > 0{
                    continue;
                }
                let triplets_on_hand = round.player_rounds[player_id as usize].first_14.count_triplets().min(3);
                triplet_rounds[triplets_on_hand as usize][player_id as usize] += 1;
                if p_bombs[player_id as usize] == 1 {
                    bomb_self_given_triplet[triplets_on_hand as usize][player_id as usize] += 1;
                }
                bomb_opp_given_triplet[triplets_on_hand as usize][player_id as usize] += bombs[((player_id + 1) % 2) as usize];

                //Post exch stats
                if p_bombs[player_id as usize] == 1 {
                    continue;
                }
                let triplets_on_hand_post_exch = round.player_rounds[player_id as usize].final_14().count_triplets().min(3);
                triplet_post_exch_rounds[triplets_on_hand_post_exch as usize][player_id as usize] += 1;
                bomb_opp_given_post_exch_triplet[triplets_on_hand_post_exch as usize][player_id as usize] += bombs[((player_id + 1) % 2) as usize];
            }
            if bombs_team_1 > 0 {
                round_score_diff_given_bomb[0] += round.player_rounds[0].round_score_relative_gain() as i64;
            }
            if bombs_team_2 > 0 {
                round_score_diff_given_bomb[1] += round.player_rounds[1].round_score_relative_gain() as i64;
            }
            if p0_bombs > 0 {
                bombs_opp_when_bomb[0] += bombs_team_2;
            }
            if p1_bombs > 0 {
                bombs_opp_when_bomb[1] += bombs_team_1;
            }
            if p2_bombs > 0 {
                bombs_opp_when_bomb[2] += bombs_team_2;
            }
            if p3_bombs > 0 {
                bombs_opp_when_bomb[3] += bombs_team_1;
            }
        }
    }
    println!("ERS given Bomb: {}", format_slice_abs_relative2_i64(&round_score_diff_given_bomb, &bombs_team_rounds));
    println!("Bomb on hand given TCall: {}", format_slice_abs_relative2(&bombs_self_when_tcall, &tcall_rounds));
    println!("Bomb on hand given GTCall: {}", format_slice_abs_relative2(&bombs_self_when_gtcall, &gtcall_rounds));
    println!("Bomb on hand given TCall & <=2 HC: {}", format_slice_abs_relative2(&bombs_self_when_tcall_two_hc_or_less, &tcall_rounds_two_hc_or_less));
    println!("Bomb in opponent given any Call: {}", format_slice_abs_relative2(&bombs_opp_when_call, &call_rounds));
    println!("Bomb in opponent given Call & No-self bomb: {}", format_slice_abs_relative2(&bombs_opp_when_call_no_bomb, &call_rounds_no_bomb));
    println!("Bomb in opponent given Bomb: {}", format_slice_abs_relative2(&bombs_opp_when_bomb, &bombs_final_14));
    println!("Tichu Call Rate given self bomb: {}", format_slice_abs_relative2(&tcall_and_bomb_rounds, &bombs_final_14));
    println!("Tichu Success Rate given self bomb: {}", format_slice_abs_relative2(&tcall_and_bomb_successes, &tcall_and_bomb_rounds));
    println!("Tichu Success Rate given enemy bomb: {}", format_slice_abs_relative2(&tcall_and_enemy_bomb_successes, &tcall_and_enemy_bomb_rounds));
    println!("GTichu Success Rate given enemy bomb: {}", format_slice_abs_relative2(&gtcall_and_enemy_bomb_successes, &gtcall_and_enemy_bomb_rounds));
    for triplet_amt in 0..4{
        println!("Bomb on hand given {} triplets on hand before exch: {}",triplet_amt, format_slice_abs_relative2(&bomb_self_given_triplet[triplet_amt], &triplet_rounds[triplet_amt]));
        println!("Bomb in opponent given {} triplets on hand before exch: {}",triplet_amt, format_slice_abs_relative2(&bomb_opp_given_triplet[triplet_amt], &triplet_rounds[triplet_amt]));
        println!("Bomb in opponent given {} triplets on hand post exch: {}",triplet_amt, format_slice_abs_relative2(&bomb_opp_given_post_exch_triplet[triplet_amt], &triplet_post_exch_rounds[triplet_amt]));
    }

    //Probability of bomb when following even_odd duplicate strategy
    let lo_card = |prh: &PlayerRoundHand| get_card_type(prh.left_out_exchange_card());
    let ro_card = |prh: &PlayerRoundHand| get_card_type(prh.right_out_exchange_card());
    let mut bombs_opp_when_exch = [0; 2];
    let mut bombs_opp_when_not_exch = [0; 2];
    let mut exch_rounds = [0; 2];
    let mut not_exch_rounds = [0; 2];
    let mut round_score_diff_given_exch = [0; 2];

    let mut double_exch_strat_rounds = [0; 2];
    let mut bombs_opp_when_double_exch_strat = [0; 2];
    let mut round_score_diff_given_double_exch = [0; 2];

    let mut small_big_exch_strat_rounds = [0; 2];
    let mut bombs_opp_when_small_big_exch_strat = [0; 2];
    let mut round_score_diff_given_small_big_exch = [0; 2];
    for game in db.games.iter() {
        for (round, _) in game.rounds.iter() {
            let bombs_team_1 = (contains_bomb(round.player_rounds[0].final_14()) + contains_bomb(round.player_rounds[2].final_14())).min(1);
            let bombs_team_2 = (contains_bomb(round.player_rounds[1].final_14()) + contains_bomb(round.player_rounds[3].final_14())).min(1);
            //Check if team_1 follows strategy
            let pr0 = round.player_rounds.get(0).unwrap();
            let pr1 = round.player_rounds.get(1).unwrap();
            let pr2 = round.player_rounds.get(2).unwrap();
            let pr3 = round.player_rounds.get(3).unwrap();
            let team_1_exch = is_even_odd_or_duplicate_strategy((pr0.first_14, pr2.first_14), (lo_card(pr0), ro_card(pr2)), (ro_card(pr0), lo_card(pr2)));
            let team_2_exch = is_even_odd_or_duplicate_strategy((pr1.first_14, pr3.first_14), (lo_card(pr1), ro_card(pr3)), (ro_card(pr1), lo_card(pr3)));
            if team_1_exch {
                exch_rounds[0] += 1;
                bombs_opp_when_exch[0] += bombs_team_2;
                round_score_diff_given_exch[0] += round.player_rounds[0].round_score_relative_gain() as i64;
            } else {
                not_exch_rounds[0] += 1;
                bombs_opp_when_not_exch[0] += bombs_team_2;
            }
            if team_2_exch {
                exch_rounds[1] += 1;
                bombs_opp_when_exch[1] += bombs_team_1;
                round_score_diff_given_exch[1] += round.player_rounds[1].round_score_relative_gain() as i64;
            } else {
                not_exch_rounds[1] += 1;
                bombs_opp_when_not_exch[1] += bombs_team_1;
            }

            //Other Exchange strategies:
            let team_1_double_exch_strat = is_duplicate_strategy((pr0.first_14, pr2.first_14), (lo_card(pr0), ro_card(pr2)), (ro_card(pr0), lo_card(pr2)));
            let team_2_double_exch_strat = is_duplicate_strategy((pr1.first_14, pr3.first_14), (lo_card(pr1), ro_card(pr3)), (ro_card(pr1), lo_card(pr3)));
            if team_1_double_exch_strat {
                double_exch_strat_rounds[0] += 1;
                bombs_opp_when_double_exch_strat[0] += bombs_team_2;
                round_score_diff_given_double_exch[0] += round.player_rounds[0].round_score_relative_gain() as i64;
            }
            if team_2_double_exch_strat {
                double_exch_strat_rounds[1] += 1;
                bombs_opp_when_double_exch_strat[1] += bombs_team_1;
                round_score_diff_given_double_exch[1] += round.player_rounds[1].round_score_relative_gain() as i64;
            }

            let team_1_small_big_exch_strat = is_smaller_bigger_strategy((pr0.first_14, pr2.first_14), (lo_card(pr0), ro_card(pr2)), (ro_card(pr0), lo_card(pr2)));
            let team_2_small_big_exch_strat = is_smaller_bigger_strategy((pr1.first_14, pr3.first_14), (lo_card(pr1), ro_card(pr3)), (ro_card(pr1), lo_card(pr3)));
            if team_1_small_big_exch_strat {
                small_big_exch_strat_rounds[0] += 1;
                bombs_opp_when_small_big_exch_strat[0] += bombs_team_2;
                round_score_diff_given_small_big_exch[0] += round.player_rounds[0].round_score_relative_gain() as i64;
            }
            if team_2_small_big_exch_strat {
                small_big_exch_strat_rounds[1] += 1;
                bombs_opp_when_small_big_exch_strat[1] += bombs_team_1;
                round_score_diff_given_small_big_exch[1] += round.player_rounds[1].round_score_relative_gain() as i64;
            }
        }
    }
    println!("ERS given Exchange Strat: {}", format_slice_abs_relative2_i64(&round_score_diff_given_exch, &exch_rounds));
    println!("Bomb in opponent given Exchange Strat: {}", format_slice_abs_relative2(&bombs_opp_when_exch, &exch_rounds));
    println!("Bomb in opponent given ! Exchange Strat: {}", format_slice_abs_relative2(&bombs_opp_when_not_exch, &not_exch_rounds));
    println!("ERS given Double Exchange Strat: {}", format_slice_abs_relative2_i64(&round_score_diff_given_double_exch, &double_exch_strat_rounds));
    println!("Bomb in opponent given Double Exchange Strat: {}", format_slice_abs_relative2(&bombs_opp_when_double_exch_strat, &double_exch_strat_rounds));
    println!("ERS given BigSmall Exchange Strat: {}", format_slice_abs_relative2_i64(&round_score_diff_given_small_big_exch, &small_big_exch_strat_rounds));
    println!("Bomb in opponent given small big Exchange Strat: {}", format_slice_abs_relative2(&bombs_opp_when_small_big_exch_strat, &small_big_exch_strat_rounds));

    //How many bombs are due to one exchange card in particular?
    let mut bomb_spawn_team = [0; 2];
    let mut bomb_spawn_team_exch = [0; 2];
    let mut bomb_spawn_team_poor_exch = [0; 2];
    let mut bombs_by_partner = [0; 2];
    let mut bombs_by_partner_exch = [0; 2];
    let mut bombs_by_partner_poor_exch = [0; 2];
    let mut bombs_with_one_exch_card = [0; 2];
    let mut bombs_with_one_exch_card_exch = [0; 2];
    let mut bombs_with_one_exch_card_poor_exch = [0; 2];
    let mut bombs_all_exch = [0; 2];
    let mut bombs_all_exch_poor_exch = [0; 2];
    let mut bombs_all_exch_exch = [0; 2];

    for game in db.games.iter() {
        for (round, _) in game.rounds.iter() {
            for team in 0..2 {
                let bombs_team = (contains_bomb(round.player_rounds[team].final_14()) + contains_bomb(round.player_rounds[2 + team].final_14())).min(1);
                if bombs_team > 0 {
                    let oteam = 1 - team;
                    let o1 = round.player_rounds.get(oteam).unwrap();
                    let o2 = round.player_rounds.get(oteam + 2).unwrap();
                    //Check strategy of other team
                    let poor_exch = !is_even_odd_or_duplicate_strategy((o1.first_14, o2.first_14), (lo_card(o1), ro_card(o2)), (ro_card(o1), lo_card(o2))) as usize;

                    let pr = if contains_bomb(round.player_rounds[team].final_14()) > 0 { round.player_rounds.get(team) } else { round.player_rounds.get(team + 2) }.unwrap();
                    if contains_bomb(pr.first_14) > 0 {
                        bomb_spawn_team[team] += 1;
                        bomb_spawn_team_exch[team] += 1 - poor_exch;
                        bomb_spawn_team_poor_exch[team] += poor_exch;
                        continue;
                    }
                    let mut hand = pr.first_14 ^ hand!(pr.right_out_exchange_card(), pr.left_out_exchange_card(), pr.partner_out_exchange_card());
                    if contains_bomb(hand ^ hand!(pr.partner_in_exchange_card())) > 0 {
                        bombs_by_partner[team] += 1;
                        bombs_by_partner_exch[team] += 1 - poor_exch;
                        bombs_by_partner_poor_exch[team] += poor_exch;
                        continue;
                    }
                    hand ^= hand!(pr.partner_in_exchange_card());
                    if contains_bomb(hand ^ hand!(pr.left_in_exchange_card())) > 0 || contains_bomb(hand ^ hand!(pr.right_in_exchange_card())) > 0 {
                        bombs_with_one_exch_card[team] += 1;
                        bombs_with_one_exch_card_exch[team] += 1 - poor_exch;
                        bombs_with_one_exch_card_poor_exch[team] += poor_exch;
                        continue;
                    }
                    bombs_all_exch[team] += 1;
                    bombs_all_exch_exch[team] += 1 - poor_exch;
                    bombs_all_exch_poor_exch[team] += poor_exch;
                }
            }
        }
    }
    println!("% Bomb spawn: {}", format_slice_abs_relative(&bomb_spawn_team, rounds));
    println!("% Bomb by partner: {}", format_slice_abs_relative(&bombs_by_partner, rounds));
    println!("% Bomb by one exch card: {}", format_slice_abs_relative(&bombs_with_one_exch_card, rounds));
    println!("% Bomb by all exch card: {}", format_slice_abs_relative(&bombs_all_exch, rounds));

    println!("% Bomb spawn (given Good Opp Exch): {}", format_slice_abs_relative2(&bomb_spawn_team_exch, &exch_rounds));
    println!("% Bomb by partner (given GOE): {}", format_slice_abs_relative2(&bombs_by_partner_exch, &exch_rounds));
    println!("% Bomb by one exch card (given GOE): {}", format_slice_abs_relative2(&bombs_with_one_exch_card_exch, &exch_rounds));
    println!("% Bomb by all exch card (given GOE): {}", format_slice_abs_relative2(&bombs_all_exch_exch, &exch_rounds));

    println!("% Bomb spawn (given Poor Opp Exch): {}", format_slice_abs_relative2(&bomb_spawn_team_poor_exch, &not_exch_rounds));
    println!("% Bomb by partner (given POE): {}", format_slice_abs_relative2(&bombs_by_partner_poor_exch, &not_exch_rounds));
    println!("% Bomb by one exch card (given POE): {}", format_slice_abs_relative2(&bombs_with_one_exch_card_poor_exch, &not_exch_rounds));
    println!("% Bomb by all exch card (given POE): {}", format_slice_abs_relative2(&bombs_all_exch_poor_exch, &not_exch_rounds));
}

fn is_even_odd_or_duplicate_strategy(p_hands: (Hand, Hand), p_opp_1: (CardType, CardType), p_opp_2: (CardType, CardType)) -> bool {
    let is_duplicate = |p_hand: Hand, card_type: CardType| card_type >= TWO && (p_hand & MASK_FOUR_OF_KIND[(card_type - 1) as usize]).count_ones() >= 2;
    let is_even = |card_type: CardType| card_type >= TWO && (card_type + 1) % 2 == 0;
    let is_odd = |card_type: CardType| card_type >= TWO && card_type % 2 == 0;
    let matches_strategy = |h_1: Hand, h_2: Hand, exch_cards: (CardType, CardType)| {
        (is_even(exch_cards.0) || is_duplicate(h_1, exch_cards.0) || exch_cards.0 == SPECIAL_CARD) &&
            (is_odd(exch_cards.1) || is_duplicate(h_2, exch_cards.1) || exch_cards.1 == SPECIAL_CARD)
    };
    let opp_1 = matches_strategy(p_hands.0, p_hands.1, p_opp_1) || matches_strategy(p_hands.1, p_hands.0, (p_opp_1.1, p_opp_1.0));
    let opp_2 = matches_strategy(p_hands.0, p_hands.1, p_opp_2) || matches_strategy(p_hands.1, p_hands.0, (p_opp_2.1, p_opp_2.0));
    opp_1 & opp_2
}

fn is_duplicate_strategy(p_hands: (Hand, Hand), p_opp_1: (CardType, CardType), p_opp_2: (CardType, CardType)) -> bool {
    let is_duplicate = |p_hand: Hand, card_type: CardType| card_type >= TWO && (p_hand & MASK_FOUR_OF_KIND[(card_type - 1) as usize]).count_ones() >= 2;
    let matches_strategy = |h_1: Hand, h_2: Hand, exch_cards: (CardType, CardType)| {
        (is_duplicate(h_1, exch_cards.0) || exch_cards.0 == SPECIAL_CARD) &&
            (is_duplicate(h_2, exch_cards.1) || exch_cards.1 == SPECIAL_CARD)
    };
    let opp_1 = matches_strategy(p_hands.0, p_hands.1, p_opp_1) || matches_strategy(p_hands.1, p_hands.0, (p_opp_1.1, p_opp_1.0));
    let opp_2 = matches_strategy(p_hands.0, p_hands.1, p_opp_2) || matches_strategy(p_hands.1, p_hands.0, (p_opp_2.1, p_opp_2.0));
    opp_1 & opp_2
}
fn is_smaller_bigger_strategy(p_hands: (Hand, Hand), p_opp_1: (CardType, CardType), p_opp_2: (CardType, CardType)) -> bool {
    //W.l.o.g. p_opp_1.0 <= p_opp_2.0
    if p_opp_1.0 > p_opp_2.0 {
        return is_smaller_bigger_strategy((p_hands.1, p_hands.0), p_opp_2, p_opp_1);
    }
    p_opp_1.1 >= p_opp_2.1
}