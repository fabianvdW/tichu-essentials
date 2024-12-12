use crate::tichu_hand::*;
use crate::hand;

pub const fn is_pair_street_fast(hand: Hand) -> Option<CardType> { //returns lowest pair card if its a pair street
    //Computes pair street or not in  like 39 Bit Ops + 4KB Table Lookup
    if hand.count_ones() < 4 || hand.count_ones() > 14 {
        return None;
    }
    let (prepared, lowest_card) = prepare_hand(hand);
    if prepared.count_ones() != hand.count_ones() {
        return None;
    }
    if PAIR_STREET_DATA_ARRAY[(prepared >> PACKING_BITS) as usize] & (1 << (prepared & PACKING_BITS_MASK)) != 0u64 {
        Some(lowest_card)
    } else {
        None
    }
}

pub const fn prepare_hand(hand: Hand) -> (u16, CardType) { //(Prepared Hand, lowest non special card)
    debug_assert!(hand.count_ones() <= 14); //This trick only works for Hands with 14 or less cards.
    debug_assert!(hand.count_ones() >= 4); //Don't call this on hands that are definitely not a pair street.
    let has_phoenix = hand & hand!(PHOENIX);
    let mut singles = (hand | (hand >> RED) | (hand >> GREEN) | (hand >> BLUE)) & MASK_YELLOW;
    let lowest_non_special_card = singles.trailing_zeros();
    singles = (singles >> lowest_non_special_card) & 0x7F; //We map the lowest card to bit 0. For the longest pair street, it will have bit 6 set as maximum.
    //We check later that the amount of cards we took match our hand count
    let true_pairs: Hand = (hand >> BLUE | hand >> GREEN | hand >> RED) & hand & MASK_NORMAL_CARDS; //Bitboard of all pairs. We ought to shift this also to the yellow column.
    let mut true_pairs_in_yellow: Hand = (true_pairs | (true_pairs >> BLUE) | (true_pairs >> GREEN)) & MASK_YELLOW;
    true_pairs_in_yellow = ((true_pairs_in_yellow >> lowest_non_special_card) & 0x7F) << 7; //Bit 7 to 13 may be set.


    ((singles | true_pairs_in_yellow | (has_phoenix << 14)) as u16, lowest_non_special_card as CardType)
}

pub const fn is_pair_street_slow(unprepared_hand: u64) -> bool {
    if unprepared_hand.count_ones() < 4 || unprepared_hand.count_ones() > 14 {
        return false;
    }
    let prepared_hand = prepare_hand(unprepared_hand).0;
    if prepared_hand.count_ones() != unprepared_hand.count_ones() {
        return false;
    }
    is_pair_street_slow_prepared(prepared_hand)
}

pub const fn is_pair_street_slow_prepared(prepared_hand: u16) -> bool {
    let mut has_phoenix = (prepared_hand >> 14) & 0b1 != 0u16; //has_phoenix is turned off at first hole.
    let mut cards: u16 = 0u16; //Lay out the cards next to each other in bit format
    let mut i: isize = 0;
    while i <= 6 {
        cards |= ((prepared_hand >> i) & 0b1) << (2 * i);
        cards |= ((prepared_hand >> (i + 7)) & 0b1) << (2 * i + 1);
        i += 1;
    }
    let mut last_card_type = 0;
    while cards >> (2 * last_card_type) != 0u16 {
        let next_in_line = (cards >> (2 * last_card_type)) & 0b11;
        if next_in_line == 0u16 {
            return false;
        }
        if next_in_line.count_ones() == 1 {
            //Only valid if has phoenix, which will file the hole
            if !has_phoenix {
                return false;
            }
            has_phoenix = false;
        }
        //Else no hole, just continue.
        last_card_type += 1;
    }

    !has_phoenix  //The phoenix has to be used to fill a hole!
}
const INDEX_BITS: usize = 15;
const PACKING_BITS: usize = 6;
const PACKING_BITS_MASK: u16 = (1 << PACKING_BITS) - 1;
const ARRAY_BITS: usize = INDEX_BITS - PACKING_BITS;

const ARRAY_ENTRIES: usize = 1 << ARRAY_BITS;

pub const PAIR_STREET_DATA_ARRAY: [u64; 1 << ARRAY_BITS] = { //4KB
    let mut arr: [u64; ARRAY_ENTRIES] = [0; ARRAY_ENTRIES];
    let mut i = 0;
    while i < ARRAY_ENTRIES {
        //Count through i * 2**PACKING_BITS to (i+1) * 2**PACKING_BITS, and pack info into array
        let mut entry = 0u64;
        let start = i * (1 << PACKING_BITS);
        let mut j = 0;
        while j < (1 << PACKING_BITS) {
            entry |= (is_pair_street_slow_prepared(j + start as u16) as u64) << j;
            j += 1;
        }
        arr[i] = entry;
        i += 1;
    }
    arr
};