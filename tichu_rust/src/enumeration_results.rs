use generic_array::GenericArray;
use generic_array::typenum::U80;
use crate::countable_properties::{CountAll, CountBombs0_1, CountBombsFourOfKind0_1, CountBombsStraights0_1, CountHandCategory, CountHasFourAces0_1, CountLongestStraight, CountLongestStraightFlush};
use crate::enumerate_hands::{count_special_card_invariant_property, count_special_card_sensitive_property};

pub fn count_tichu_hands() {
    count_special_card_invariant_property::<CountAll, 14>(CountAll);
    //5804731963800
}
pub fn count_bombs_0_1() {
    count_special_card_invariant_property::<CountBombs0_1, 14>(CountBombs0_1);
    // 294663199638 out of 5804731963800 (0.0507625849868014)
}
pub fn count_four_of_kind_bombs_0_1() {
    count_special_card_invariant_property::<CountBombsFourOfKind0_1, 14>(
        CountBombsFourOfKind0_1,
    ); //204703407480 out of 5804731963800 (0.035264919854455)
}
pub fn count_straight_bombs_0_1() {
    count_special_card_invariant_property::<CountBombsStraights0_1, 14>(CountBombsStraights0_1);
    // 92272299834 out of 5804731963800 (0.0158960483290937)
}
pub fn count_gt_hands() {
    count_special_card_invariant_property::<CountAll, 8>(CountAll);
    // 1420494075
}
pub fn count_gt_bombs_0_1() {
    count_special_card_invariant_property::<CountBombs0_1, 8>(CountBombs0_1);
    // 4229667 out of 1420494075 (0.0029776027048898)
}

pub fn count_gt_hand_category() -> GenericArray<u64, U80> {
    count_special_card_sensitive_property::<CountHandCategory, 8>(CountHandCategory).property_counted
}
pub fn count_first14_hand_category() -> GenericArray<u64, U80> {
    count_special_card_sensitive_property::<CountHandCategory, 14>(CountHandCategory).property_counted
}

pub fn count_gt_hand_has_four_aces() {
    count_special_card_invariant_property::<CountHasFourAces0_1, 8>(CountHasFourAces0_1);
}

pub fn count_longest_straight_distribution() {
    count_special_card_invariant_property::<CountLongestStraight, 14>(CountLongestStraight);
    //[1271348424, 221667695352, 1089094274744, 1494123765200, 1224371890896, 804372256512, 474483929984, 262784360448, 135044874240, 62738923520, 25198854144, 8002732032, 1577058304]
}

pub fn count_longest_straight_flush_distribution() {
    count_special_card_invariant_property::<CountLongestStraightFlush, 14>(CountLongestStraightFlush);
    //[170226064036, 3231369839200, 1886228484214, 424635276516, 77497737660, 12663862404, 1846862262, 235769688, 25627140, 2277968, 155316, 7224, 172]
}