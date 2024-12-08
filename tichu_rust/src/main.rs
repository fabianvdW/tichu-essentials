pub mod tichu_hand;
pub mod enumerate_hands;
pub mod countable_properties;
pub mod enumeration_results;
pub mod bsw_binary_format;

use bsw_binary_format::PlayerRoundHand;

fn main() {
    //enumeration_results::count_straight_bombs_0_1();
    //enumeration_results::count_gt_hands();
    //enumeration_results::count_gt_bombs_0_1();
    println!("{}", std::mem::size_of::<PlayerRoundHand>())
}

#[cfg(test)]
mod tests {
    use crate::countable_properties::{CountAll, CountBombs0_1};
    use crate::enumerate_hands::count_special_card_invariant_property;
    use crate::tichu_hand::*;
    use super::hand;

    #[test]
    fn simple_hand_print(){
        let hand: Hand= hand!(ACE+RED, ACE+GREEN, ACE+BLUE, TEN+YELLOW, DRAGON, MAHJONG, PHOENIX);
        println!("{}",hand.debug_print());
        println!("{}", hand.pretty_print());
    }

    #[test]
    fn a_few_tichu_one_hands(){
        let hand: Hand = tichu_one_str_to_hand("gizHsF2t");
        println!("gizHsF2t: {}",hand.pretty_print());
        let hand: Hand = tichu_one_str_to_hand("OS3PX6oU");
        println!("OS3PX6oU: {}",hand.pretty_print());
        let hand: Hand = tichu_one_str_to_hand("4WIq5LRT");
        println!("4WIq5LRT: {}",hand.pretty_print());
        let hand: Hand = tichu_one_str_to_hand("QGCEVfvr");
        println!("QGCEVfvr: {}",hand.pretty_print());
        let hand: Hand = tichu_one_str_to_hand("gizHsF2tpAaDkK");
        println!("gizHsF2tpAaDkK: {}",hand.pretty_print());
        let hand: Hand = tichu_one_str_to_hand("OS3PX6oUuynelN");
        println!("OS3PX6oUuynelN: {}",hand.pretty_print());
        let hand: Hand = tichu_one_str_to_hand("4WIq5LRTBMxmcJ");
        println!("4WIq5LRTBMxmcJ: {}",hand.pretty_print());
        let hand: Hand = tichu_one_str_to_hand("QGCEVfvrh1djbw");
        println!("QGCEVfvrh1djbw: {}",hand.pretty_print());

    }

    #[test]
    fn bomb_detection(){
        let hand: Hand= hand!(ACE+RED, ACE+GREEN, ACE+BLUE, TEN+YELLOW, DRAGON, MAHJONG, PHOENIX);
        assert!(!hand.contains_four_of_kind_bomb());
        assert!(!hand.contains_straight_bomb());
        let hand: Hand= hand!(ACE+RED, ACE+GREEN, ACE+BLUE, ACE+YELLOW, DRAGON, MAHJONG, PHOENIX);
        assert!(hand.contains_four_of_kind_bomb());
        assert!(!hand.contains_straight_bomb());
        let hand: Hand= hand!(TWO+RED, THREE+RED, FOUR+RED, FIVE+RED, MAHJONG);
        assert!(!hand.contains_four_of_kind_bomb());
        assert!(!hand.contains_straight_bomb());
        let hand: Hand= hand!(TWO+RED, THREE+RED, FOUR+RED, FIVE+RED, SIX+RED, MAHJONG);
        assert!(!hand.contains_four_of_kind_bomb());
        assert!(hand.contains_straight_bomb());
    }

    #[test]
    fn gt_card_counts(){
        assert_eq!(count_special_card_invariant_property::<CountAll, 8>(CountAll).property_counted[0], 1420494075);
        assert_eq!(count_special_card_invariant_property::<CountBombs0_1, 8>(CountBombs0_1).property_counted[1], 4229667);
    }
}