use crate::tichu_hand::*;
use crate::hand;

pub const fn is_pair_street_fast(hand: Hand) -> Option<CardType> {
    if hand.count_ones() < 4 || hand.count_ones() > 14 || hand.count_ones() % 2 != 0{
        return None;
    }
    let has_phoenix = hand & hand!(PHOENIX);
    let singles = (hand | (hand >> RED) | (hand >> GREEN) | (hand >> BLUE)) & MASK_YELLOW;
    let lowest_non_special_card = singles.trailing_zeros() as CardType;

    let true_pairs: Hand = (hand >> BLUE | hand >> GREEN | hand >> RED) & hand & MASK_NORMAL_CARDS; //Bitboard of all pairs. We ought to shift this also to the yellow column.
    let true_pairs_in_yellow: Hand = (true_pairs | (true_pairs >> BLUE) | (true_pairs >> GREEN)) & MASK_YELLOW;
    if  singles.count_ones() + true_pairs_in_yellow.count_ones() + has_phoenix as u32 != hand.count_ones() || (true_pairs_in_yellow ^ singles).count_ones() != has_phoenix as u32{
        return None;
    }
    let singles_span = 64 - singles.leading_zeros() - singles.trailing_zeros();
    //We are a pair street if the singles form a street, as the pairs are equivalent to singles with the phoenix(checked above).
    if singles_span <= singles.count_ones(){
        Some(lowest_non_special_card)
    } else {
        None
    }
}