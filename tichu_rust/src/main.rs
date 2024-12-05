use crate::tichu_hand::*;

pub mod tichu_hand;

fn main() {
    let hand: Hand= hand!(ACE+RED, ACE+GREEN, ACE+BLUE, TEN+YELLOW, DRAGON, MAHJONG, PHOENIX);
    println!("{}",hand.debug_print());
    println!("{}", hand.pretty_print());
}

#[cfg(test)]
mod tests {
    use super::*;

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
}