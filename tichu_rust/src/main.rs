#![ allow(unused_imports)]
pub mod tichu_hand;
pub mod enumerate_hands;
pub mod countable_properties;
pub mod enumeration_results;
pub mod bsw_database;
pub mod street_detection_tricks;
pub mod pair_street_detection_trick;
pub mod bsw_binary_format;
pub mod analysis;

use std::collections::HashMap;
use crate::analysis::bomb_stats::{evaluate_bomb_stats, evaluate_bombs_in_play};
use crate::analysis::exchange_stats::evaluate_exchange_stats;
use crate::analysis::first_8_transition_probability::calculate_transition_probabilities;
use crate::analysis::gt_stats::{evaluate_gt_call_rates, evaluate_gt_stats, evaluate_gt_win_probs, HandCategory};
use crate::analysis::general_stats::{evaluate_general_stats, evaluate_general_stats_onlyr0};
use crate::analysis::parsing_stats::evaluate_parsing_stats;
use crate::analysis::street_stats::{evaluate_lose_four_to_queen, evaluate_lose_tichujana_hand, evaluate_streets_in_play, evaluate_streets_size_four_become_real_streets};
use crate::bsw_binary_format::binary_format_constants::CALL_GRAND_TICHU;
use crate::tichu_hand::*;
use crate::bsw_database::DataBase;
fn filter_db(db: DataBase){
    let games_wrs = db.collect_winrate_players();
    let mut good_players = HashMap::new();
    for (player, (games, wins)) in games_wrs{
        if games >= 100 && games as f64 * 0.55 <= wins as f64{
            good_players.insert(player, db.players[player].clone());
        }
    }
    println!("All players: {}", db.players.len());
    println!("Good players: {}", good_players.len());
    let mut new_db = DataBase{games: Vec::new(), players: db.players.clone()};
    new_db.games = db.games.into_iter().filter(|game| game.player_ids.iter().all(|player_id| good_players.contains_key(&(*player_id as usize)))).collect();
    println!("Filterd games: {} and {} rounds", new_db.games.len(),  new_db.games.iter().fold(0, |acc, inc| acc + inc.rounds.len() ));
    new_db.write("bsw_filtered.db").unwrap();

}

//4. GT statistics: Get started
fn main() {
    //let db = DataBase::from_bsw().unwrap();
    //db.write("bsw.db").unwrap();

    let db = DataBase::read("bsw.db").unwrap();
    println!("Loaded {} games and {} rounds!", db.games.len(), db.games.iter().fold(0, |acc, inc| acc + inc.rounds.len() ));
    //filter_db(db);
    //evaluate_parsing_stats(&db);
    //evaluate_general_stats(&db);
    evaluate_general_stats_onlyr0(&db);
    //evaluate_bomb_stats(&db);
    //evaluate_bombs_in_play(&db);
    //evaluate_streets_in_play(&db);
    //evaluate_streets_size_four_become_real_streets(&db);
    //evaluate_lose_tichujana_hand(&db);
    //evaluate_lose_four_to_queen(&db);
    //evaluate_exchange_stats(&db);
    //evaluate_gt_stats(&db);
    //evaluate_gt_win_probs(&db);
    //evaluate_gt_call_rates(enumeration_results::count_gt_hand_category());




    //enumeration_results::count_bombs_0_1();
    //enumeration_results::count_straight_bombs_0_1();
    //enumeration_results::count_gt_hands();
    //enumeration_results::count_gt_bombs_0_1();
    //enumeration_results::count_gt_hand_has_four_aces();
    //enumeration_results::count_gt_hand_category();
    //enumeration_results::count_first14_hand_category();
    //enumeration_results::count_bombs_0_1();
    //enumeration_results::count_longest_straight_distribution();
    //enumeration_results::count_longest_straight_flush_distribution();
    //HandCategory::print_category_lists();
    //calculate_transition_probabilities();
}

#[cfg(test)]
mod tests {
    use crate::countable_properties::{CountAll, CountBombs0_1};
    use crate::enumerate_hands::count_special_card_invariant_property;
    use crate::tichu_hand::*;
    use crate::street_detection_tricks::{is_street_fast, phoenix_used_as_street_extension};
    use crate::pair_street_detection_trick::{is_pair_street_fast};
    use super::hand;

    #[test]
    fn simple_hand_print() {
        let hand: Hand = hand!(ACE+RED, ACE+GREEN, ACE+BLUE, TEN+YELLOW, DRAGON, MAHJONG, PHOENIX);
        println!("{}", hand.debug_print());
        println!("{}", hand.pretty_print());
    }

    #[test]
    fn a_few_tichu_one_hands() {
        let hand: Hand = tichu_one_str_to_hand("gizHsF2t");
        println!("gizHsF2t: {}", hand.pretty_print());
        let hand: Hand = tichu_one_str_to_hand("OS3PX6oU");
        println!("OS3PX6oU: {}", hand.pretty_print());
        let hand: Hand = tichu_one_str_to_hand("4WIq5LRT");
        println!("4WIq5LRT: {}", hand.pretty_print());
        let hand: Hand = tichu_one_str_to_hand("QGCEVfvr");
        println!("QGCEVfvr: {}", hand.pretty_print());
        let hand: Hand = tichu_one_str_to_hand("gizHsF2tpAaDkK");
        println!("gizHsF2tpAaDkK: {}", hand.pretty_print());
        let hand: Hand = tichu_one_str_to_hand("OS3PX6oUuynelN");
        println!("OS3PX6oUuynelN: {}", hand.pretty_print());
        let hand: Hand = tichu_one_str_to_hand("4WIq5LRTBMxmcJ");
        println!("4WIq5LRTBMxmcJ: {}", hand.pretty_print());
        let hand: Hand = tichu_one_str_to_hand("QGCEVfvrh1djbw");
        println!("QGCEVfvrh1djbw: {}", hand.pretty_print());
    }

    #[test]
    fn bomb_detection() {
        let hand: Hand = hand!(ACE+RED, ACE+GREEN, ACE+BLUE, TEN+YELLOW, DRAGON, MAHJONG, PHOENIX);
        assert!(!hand.contains_four_of_kind_bomb());
        assert!(!hand.contains_straight_bomb());
        let hand: Hand = hand!(ACE+RED, ACE+GREEN, ACE+BLUE, ACE+YELLOW, DRAGON, MAHJONG, PHOENIX);
        assert!(hand.contains_four_of_kind_bomb());
        assert!(!hand.contains_straight_bomb());
        let hand: Hand = hand!(TWO+RED, THREE+RED, FOUR+RED, FIVE+RED, MAHJONG);
        assert!(!hand.contains_four_of_kind_bomb());
        assert!(!hand.contains_straight_bomb());
        let hand: Hand = hand!(TWO+RED, THREE+RED, FOUR+RED, FIVE+RED, SIX+RED, MAHJONG);
        assert!(!hand.contains_four_of_kind_bomb());
        assert!(hand.contains_straight_bomb());
    }

    #[test]
    fn gt_card_counts() {
        assert_eq!(count_special_card_invariant_property::<CountAll, 8>(CountAll).property_counted[0], 1420494075);
        assert_eq!(count_special_card_invariant_property::<CountBombs0_1, 8>(CountBombs0_1).property_counted[1], 4229667);
    }

    #[test]
    fn hand_type_pairs() {
        assert!(matches!(hand!(TWO+RED, TWO+BLUE).hand_type(), Some(HandType::Pairs(card)) if card == TWO));
        assert!(matches!(hand!(TWO+RED, TWO+GREEN).hand_type(), Some(HandType::Pairs(card)) if card == TWO));
        assert!(matches!(hand!(TWO+RED, TWO+YELLOW).hand_type(), Some(HandType::Pairs(card)) if card == TWO));
        assert!(matches!(hand!(TWO+YELLOW, TWO+BLUE).hand_type(), Some(HandType::Pairs(card)) if card == TWO));
        assert!(matches!(hand!(TWO+RED, PHOENIX).hand_type(), Some(HandType::Pairs(card)) if card == TWO));
        assert!(hand!(TWO+RED, THREE+BLUE).hand_type() == None);
        assert!(hand!(PHOENIX, MAHJONG).hand_type() == None);
    }

    #[test]
    fn hand_type_triplets() {
        assert!(matches!(hand!(TWO+RED, TWO+BLUE, TWO+YELLOW).hand_type(), Some(HandType::Triplets(card)) if card == TWO));
        assert!(matches!(hand!(TWO+RED, TWO+GREEN, TWO+YELLOW).hand_type(), Some(HandType::Triplets(card)) if card == TWO));
        assert!(matches!(hand!(TWO+GREEN, TWO+BLUE, TWO+YELLOW).hand_type(), Some(HandType::Triplets(card)) if card == TWO));
        assert!(matches!(hand!(TWO+RED, TWO+BLUE, TWO+GREEN).hand_type(), Some(HandType::Triplets(card)) if card == TWO));
        assert!(matches!(hand!(TWO+RED, PHOENIX, TWO+YELLOW).hand_type(), Some(HandType::Triplets(card)) if card == TWO));
        assert!(matches!(hand!(DOG, PHOENIX, TWO+YELLOW).hand_type(), None));
        assert!(matches!(hand!(DRAGON, PHOENIX, TWO+YELLOW).hand_type(), None));
        assert!(matches!(hand!(MAHJONG, PHOENIX, TWO+YELLOW).hand_type(), None));
        assert!(matches!(hand!(THREE+YELLOW, PHOENIX, TWO+YELLOW).hand_type(), None));
    }

    #[test]
    fn hand_type_bomb4() {
        assert!(matches!(hand!(ACE+RED, ACE+GREEN, ACE+BLUE, PHOENIX).hand_type(), None));
        assert!(matches!(hand!(ACE+RED, ACE+GREEN, ACE+BLUE, MAHJONG).hand_type(), None));
        assert!(matches!(hand!(ACE+RED, ACE+GREEN, ACE+BLUE, DRAGON).hand_type(), None));
        assert!(matches!(hand!(ACE+RED, ACE+GREEN, ACE+BLUE, DOG).hand_type(), None));
        assert!(matches!(hand!(ACE+RED, ACE+GREEN, ACE+BLUE, TWO+YELLOW).hand_type(), None));
        assert!(matches!(hand!(ACE+RED, ACE+GREEN, ACE+BLUE, ACE+YELLOW).hand_type(), Some(HandType::Bomb4(card)) if card == ACE));
    }

    #[test]
    fn hand_type_pairstreet4() {
        assert!(matches!(hand!(ACE+RED, ACE+GREEN, DOG, PHOENIX).hand_type(), None));
        assert!(matches!(hand!(ACE+RED, ACE+GREEN, KING+BLUE, PHOENIX).hand_type(), Some(HandType::PairStreet(card, length)) if card == KING && length == 4));
        assert!(matches!(hand!(ACE+RED, ACE+GREEN, MAHJONG, PHOENIX).hand_type(), None));
    }

    #[test]
    fn hand_type_fullhouse() {
        assert!(matches!(hand!(ACE+RED, ACE+GREEN, ACE+BLUE, ACE+YELLOW, PHOENIX).hand_type(), None));
        assert!(matches!(hand!(ACE+RED, ACE+GREEN, ACE+BLUE, KING+YELLOW, PHOENIX).hand_type(), Some(HandType::FullHouse(card, card2)) if card == KING && card2 == ACE));
        assert!(matches!(hand!(ACE+RED, ACE+GREEN, KING+BLUE, KING+YELLOW, PHOENIX).hand_type(), Some(HandType::FullHouse(card, card2)) if card == KING && card2 == ACE));
        assert!(matches!(hand!(TWO+RED, KING+GREEN, KING+BLUE, KING+YELLOW, PHOENIX).hand_type(), Some(HandType::FullHouse(card, card2)) if card == TWO && card2 == KING));
        assert!(matches!(hand!(DOG, KING+GREEN, KING+BLUE, KING+YELLOW, PHOENIX).hand_type(), None));
        assert!(matches!(hand!(MAHJONG, KING+GREEN, KING+BLUE, KING+YELLOW, PHOENIX).hand_type(), None));
        assert!(matches!(hand!(TWO+RED, TWO+GREEN, KING+GREEN, KING+BLUE, PHOENIX).hand_type(), Some(HandType::FullHouse(card, card2)) if card == TWO && card2 == KING));
        assert!(matches!(hand!(SIX+RED, SIX+BLUE, EIGHT+GREEN, EIGHT+BLUE, EIGHT+YELLOW).hand_type(), Some(HandType::FullHouse(card, card2)) if card == SIX && card2 == EIGHT));
    }

    #[test]
    fn is_street_test(){
        assert_eq!(is_street_fast(hand!(DOG)), None);
        assert_eq!(is_street_fast(hand!(DOG, MAHJONG, PHOENIX, DRAGON, KING+YELLOW)), None);
        assert_eq!(is_street_fast(hand!(TWO+BLUE, MAHJONG, PHOENIX, THREE+RED, FOUR+YELLOW)), Some(SPECIAL_CARD));
        assert_eq!(is_street_fast(hand!(TWO+BLUE, MAHJONG, PHOENIX, THREE+RED, FOUR+YELLOW, FIVE+BLUE)), Some(SPECIAL_CARD));
        assert_eq!(is_street_fast(hand!(TWO+BLUE, MAHJONG, PHOENIX, THREE+RED, FIVE+BLUE)), Some(SPECIAL_CARD));
        assert_eq!(is_street_fast(hand!(TWO+BLUE, MAHJONG, PHOENIX, THREE+RED, FIVE+BLUE, SIX+RED)), Some(SPECIAL_CARD));
        assert_eq!(is_street_fast(hand!(TWO+BLUE, FOUR+RED, THREE+RED, FIVE+BLUE, SIX+RED)), Some(TWO));
        assert_eq!(is_street_fast(hand!(TWO+BLUE, FOUR+RED, THREE+RED, FIVE+RED)), None);
        assert_eq!(is_street_fast(hand!(TWO+BLUE, FOUR+RED, THREE+RED, FIVE+RED, SEVEN+RED)), None);
        assert_eq!(is_street_fast(hand!(TWO+BLUE, FOUR+RED, THREE+RED, SIX+RED)), None);
        assert_eq!(is_street_fast(hand!(MAHJONG, PHOENIX, THREE+RED, FIVE+BLUE, SIX+RED)), None);
        assert_eq!(is_street_fast(hand!(MAHJONG, PHOENIX, THREE+RED, FIVE+BLUE, FIVE+RED)), None);
        assert_eq!(is_street_fast(hand!(MAHJONG, PHOENIX, FOUR+RED, FIVE+BLUE, SIX+RED)), None);
    }
    #[test]
    fn is_street_extension_phoenix(){
        assert_eq!(phoenix_used_as_street_extension(hand!(PHOENIX)), false);
        assert_eq!(phoenix_used_as_street_extension(hand!(TWO+BLUE, MAHJONG, PHOENIX, THREE+RED, FOUR+YELLOW)), true);
        assert_eq!(phoenix_used_as_street_extension(hand!(TWO+BLUE, MAHJONG, PHOENIX, THREE+RED, FOUR+YELLOW, FIVE+BLUE)), true);
        assert_eq!(phoenix_used_as_street_extension(hand!(TWO+BLUE, MAHJONG, PHOENIX, THREE+RED, FIVE+BLUE)), false);
        assert_eq!(phoenix_used_as_street_extension(hand!(TWO+BLUE, MAHJONG, PHOENIX, THREE+RED, FIVE+BLUE, SIX+RED)), false);
        assert_eq!(phoenix_used_as_street_extension(hand!(TWO+BLUE, MAHJONG, PHOENIX, THREE+RED, FIVE+BLUE)), false);
    }

    #[test]
    fn hand_type_street(){
        assert!(matches!(hand!(DOG, MAHJONG, PHOENIX, DRAGON, KING+YELLOW).hand_type(), None));
        assert!(matches!(hand!(TWO+BLUE, MAHJONG, PHOENIX, THREE+RED, FOUR+YELLOW).hand_type(), Some(HandType::Street(card, length)) if card == SPECIAL_CARD && length == 5));
        assert!(matches!(hand!(TWO+BLUE, MAHJONG, PHOENIX, THREE+RED, FOUR+YELLOW, FIVE+BLUE).hand_type(), Some(HandType::Street(card, length)) if card == SPECIAL_CARD && length == 6));
        assert!(matches!(hand!(TWO+BLUE, MAHJONG, PHOENIX, THREE+RED, FIVE+BLUE, SIX+RED).hand_type(), Some(HandType::Street(card, length)) if card == SPECIAL_CARD && length == 6));
        assert!(matches!(hand!(TWO+BLUE, FOUR+RED, THREE+RED, FIVE+BLUE, SIX+RED).hand_type(), Some(HandType::Street(card, length)) if card == TWO && length == 5));
        assert!(matches!(hand!(TWO+RED, FOUR+RED, THREE+RED, FIVE+RED, SIX+RED, SEVEN+RED).hand_type(), Some(HandType::BombStreet(card, length)) if card == TWO && length == 6));
        assert!(matches!(hand!(TWO+RED, FOUR+RED, THREE+RED, FIVE+RED, SIX+RED, SEVEN+RED, PHOENIX).hand_type(), Some(HandType::Street(card, length)) if card == TWO && length == 7));
        assert!(matches!(hand!(TWO+BLUE, FOUR+RED, THREE+RED, SIX+RED).hand_type(), None));
        assert!(matches!(hand!(MAHJONG, PHOENIX, THREE+RED, FIVE+BLUE, SIX+RED).hand_type(), None));
        assert!(matches!(hand!(MAHJONG, PHOENIX, THREE+RED, FIVE+BLUE, FIVE+RED).hand_type(), None));
        assert!(matches!(hand!(MAHJONG, PHOENIX, FOUR+RED, FIVE+BLUE, SIX+RED).hand_type(), None));
        assert!(matches!(hand!(ACE+RED, KING+RED, QUEEN+RED, JACK+RED, TEN+RED, NINE+RED, PHOENIX).hand_type(), Some(HandType::Street(card, length)) if card == EIGHT && length == 7));
        assert!(matches!(MASK_RED.hand_type(), Some(HandType::BombStreet(card, length)) if card == TWO && length == 13));
        assert!(matches!((hand!(MAHJONG)|MASK_RED).hand_type(), Some(HandType::Street(card, length)) if card == SPECIAL_CARD && length == 14));
        assert!(matches!((hand!(PHOENIX)|MASK_RED).hand_type(), None)); //Phoenix can't subs in for MAHJONG

    }
    #[test]
    fn is_pair_street_fast_test(){
        assert_eq!(is_pair_street_fast(hand!(PHOENIX, ACE+YELLOW, ACE+BLUE, KING+YELLOW)), Some(KING));
        assert_eq!(is_pair_street_fast(hand!(PHOENIX, ACE+YELLOW, ACE+BLUE, KING+YELLOW, KING+RED)), None);
        assert_eq!(is_pair_street_fast(hand!(PHOENIX, ACE+YELLOW, ACE+BLUE, KING+YELLOW, MAHJONG)), None);
        assert_eq!(is_pair_street_fast(hand!(PHOENIX, ACE+YELLOW, DOG, KING+YELLOW)), None);
        assert_eq!(is_pair_street_fast(hand!(PHOENIX, ACE+YELLOW, KING+RED, KING+YELLOW)), Some(KING));
        assert_eq!(is_pair_street_fast(hand!(PHOENIX, ACE+YELLOW, ACE+BLUE, QUEEN+YELLOW)), None);
        assert_eq!(is_pair_street_fast(hand!(PHOENIX, ACE+YELLOW, KING+BLUE, QUEEN+YELLOW)), None);
        assert_eq!(is_pair_street_fast(hand!(PHOENIX, ACE+YELLOW, KING+BLUE, QUEEN+YELLOW, JACK+RED, JACK+BLUE)), None);
        assert_eq!(is_pair_street_fast(hand!(PHOENIX, ACE+YELLOW, ACE+BLUE, KING+YELLOW, QUEEN+BLUE, QUEEN+RED)), Some(QUEEN));
        assert_eq!(is_pair_street_fast(hand!(PHOENIX, ACE+YELLOW, ACE+BLUE, KING+YELLOW, QUEEN+RED)), None);
        assert_eq!(is_pair_street_fast(hand!(ACE+RED, ACE+YELLOW, ACE+BLUE, KING+YELLOW, KING+RED, KING+BLUE)), None);
        assert_eq!(is_pair_street_fast(hand!(TWO+RED, TWO+YELLOW, FOUR+BLUE, FOUR+YELLOW)), None);
        assert_eq!(is_pair_street_fast(hand!(TWO+RED, TWO+YELLOW, FOUR+BLUE, FOUR+YELLOW, KING+RED, KING+BLUE)), None);
        assert_eq!(is_pair_street_fast(hand!(TWO+RED, TWO+YELLOW, THREE+RED, THREE+YELLOW, FOUR+BLUE, FOUR+GREEN, FIVE+YELLOW, PHOENIX, SIX+BLUE, SIX+YELLOW)), Some(TWO));
    }

    #[test]
    fn pair_street_hand_type(){
        assert!(matches!(hand!(PHOENIX, ACE+YELLOW, ACE+BLUE, KING+YELLOW).hand_type(), Some(HandType::PairStreet(card, length)) if card == KING && length == 4));
        assert!(matches!(hand!(PHOENIX, ACE+YELLOW, ACE+BLUE, KING+YELLOW, MAHJONG).hand_type(), None));
        assert!(matches!(hand!(PHOENIX, ACE+YELLOW, DOG, KING+YELLOW).hand_type(), None));
        assert!(matches!(hand!(PHOENIX, ACE+YELLOW, KING+RED, KING+YELLOW).hand_type(), Some(HandType::PairStreet(card, length)) if card == KING && length == 4));
        assert!(matches!(hand!(PHOENIX, ACE+YELLOW, ACE+BLUE, QUEEN+YELLOW).hand_type(), None));
        assert!(matches!(hand!(PHOENIX, ACE+YELLOW, ACE+BLUE, KING+YELLOW, QUEEN+BLUE, QUEEN+RED).hand_type(), Some(HandType::PairStreet(card, length)) if card == QUEEN && length == 6));
        assert!(matches!(hand!(PHOENIX, ACE+YELLOW, ACE+BLUE, KING+YELLOW, QUEEN+RED).hand_type(), None));
        assert!(matches!(hand!(ACE+RED, ACE+YELLOW, ACE+BLUE, KING+YELLOW, KING+RED, KING+BLUE).hand_type(), None));
        assert!(matches!(hand!(TWO+RED, TWO+YELLOW, FOUR+BLUE, FOUR+YELLOW, KING+RED, KING+BLUE).hand_type(), None));
        assert!(matches!(hand!(TWO+RED, TWO+YELLOW, THREE+RED, THREE+YELLOW, FOUR+BLUE, FOUR+GREEN, FIVE+YELLOW, PHOENIX, SIX+BLUE, SIX+YELLOW).hand_type(), Some(HandType::PairStreet(card, length)) if card == TWO && length == 10));
    }
}