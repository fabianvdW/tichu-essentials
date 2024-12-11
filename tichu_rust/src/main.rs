pub mod tichu_hand;
pub mod enumerate_hands;
pub mod countable_properties;
pub mod enumeration_results;
pub mod bsw_binary_format;
pub mod bsw_database;
mod street_detection_tricks;

use crate::tichu_hand::*;
use crate::bsw_database::DataBase;

fn main() {
    let db = DataBase::from_bsw().unwrap();
    //db.write("bsw.db").unwrap();

    //enumeration_results::count_bombs_0_1();
    //enumeration_results::count_straight_bombs_0_1();
    //enumeration_results::count_gt_hands();
    //enumeration_results::count_gt_bombs_0_1();
}

#[cfg(test)]
mod tests {
    use crate::countable_properties::{CountAll, CountBombs0_1};
    use crate::enumerate_hands::count_special_card_invariant_property;
    use crate::tichu_hand::*;
    use crate::street_detection_tricks::is_street_fast;
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
    }

    #[test]
    fn is_street_test(){
        assert_eq!(is_street_fast(hand!(DOG)), false);
        assert_eq!(is_street_fast(hand!(DOG, MAHJONG, PHOENIX, DRAGON, KING+YELLOW)), false);
        assert_eq!(is_street_fast(hand!(TWO+BLUE, MAHJONG, PHOENIX, THREE+RED, FOUR+YELLOW)), true);
        assert_eq!(is_street_fast(hand!(TWO+BLUE, MAHJONG, PHOENIX, THREE+RED, FOUR+YELLOW, FIVE+BLUE)), true);
        assert_eq!(is_street_fast(hand!(TWO+BLUE, MAHJONG, PHOENIX, THREE+RED, FIVE+BLUE)), true);
        assert_eq!(is_street_fast(hand!(TWO+BLUE, MAHJONG, PHOENIX, THREE+RED, FIVE+BLUE, SIX+RED)), true);
        assert_eq!(is_street_fast(hand!(TWO+BLUE, FOUR+RED, THREE+RED, FIVE+BLUE, SIX+RED)), true);
        assert_eq!(is_street_fast(hand!(TWO+BLUE, FOUR+RED, THREE+RED, SIX+RED)), false);
        assert_eq!(is_street_fast(hand!(MAHJONG, PHOENIX, THREE+RED, FIVE+BLUE, SIX+RED)), false);
    }
}