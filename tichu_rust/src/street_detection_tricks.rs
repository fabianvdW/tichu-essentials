use crate::tichu_hand::*;
use crate::hand;

pub const fn is_street_fast(hand: Hand) -> Option<CardType> { //Returns lowest card in case of street (does not correct phoenix if that has to be lowest
    //Computes is_street or not in like 24 Bit Ops + 4KB Table lookup
    let prepared = prepare_hand(hand);
    if prepared.count_ones() == hand.count_ones() && STREET_DATA_ARRAY[(prepared >> PACKING_BITS) as usize] & (1 << (prepared & PACKING_BITS_MASK)) != 0u64 {
        Some(prepared.trailing_zeros() as CardType)
    } else {
        None
    }
}
pub const fn phoenix_used_as_street_extension(hand: Hand) -> bool { //Returns true only for streets that contain the phoenix and for which the phoenix does not fill a hole.
    if hand & hand!(PHOENIX) == 0{
        return false;
    }
    //Also check that hand without phoenix would be a street.
    let mut prepared = prepare_hand(hand ^ hand!(PHOENIX));
    let street_length = prepared.count_ones();
    let mut i = 1;
    while i < street_length{
        prepared &= prepared >> 1;
        i += 1;
    }
    prepared > 0
}

pub const fn prepare_hand(hand: Hand) -> u64 {
    //Maps all normal cards to the yellow column, sets mahjong as bit 0.  Phoenix is shifted to bit 14. Is then a binary number of the first 15 bits.
    ((hand >> BLUE) | (hand >> GREEN) | (hand >> RED) | hand) & MASK_YELLOW | ((hand & hand!(MAHJONG)) >> MAHJONG) | ((hand & hand!(PHOENIX)) << 14)
}


pub const fn is_street_slow(mut prepared_hand: u64) -> bool {
    if prepared_hand.count_ones() < 5 {
        return false;
    }
    let mut has_phoenix: bool = (prepared_hand >> 14) & 0b1 != 0u64; //has_phoenix is turned off at first hole.
    if has_phoenix && prepared_hand == 0x7FFE { //Phoenix can not substitute in for mahjong.
        return false;
    }
    prepared_hand &= 0x3FFF;
    let mut current_lsb = prepared_hand.trailing_zeros();
    //Just count the holes
    while prepared_hand.count_ones() > 1 {
        prepared_hand &= prepared_hand - 1;
        let next_lsb = prepared_hand.trailing_zeros();
        if next_lsb > current_lsb + 2 {
            return false;
        }
        if current_lsb + 1 != next_lsb && !has_phoenix {
            return false;
        }
        if current_lsb + 1 != next_lsb && has_phoenix {
            assert!(current_lsb + 2 == next_lsb);
            has_phoenix = false;
        }
        current_lsb = next_lsb;
    }
    true
}
const INDEX_BITS: usize = 15;
const PACKING_BITS: usize = 6;
const PACKING_BITS_MASK: u64 = (1 << PACKING_BITS) - 1;
const ARRAY_BITS: usize = INDEX_BITS - PACKING_BITS;
const ARRAY_ENTRIES: usize = 1 << ARRAY_BITS;

pub const STREET_DATA_ARRAY: [u64; 1 << ARRAY_BITS] = { //4KB
    let mut arr: [u64; ARRAY_ENTRIES] = [0; ARRAY_ENTRIES];
    let mut i = 0;
    while i < ARRAY_ENTRIES {
        //Count through i * 2**PACKING_BITS to (i+1) * 2**PACKING_BITS, and pack info into array
        let mut entry = 0u64;
        let start = i * (1 << PACKING_BITS);
        let mut j = 0;
        while j < (1 << PACKING_BITS) {
            entry |= (is_street_slow(j + start as u64) as u64) << j;
            j += 1;
        }
        arr[i] = entry;
        i += 1;
    }
    arr
};