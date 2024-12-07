use crate::countable_properties::{
    CountAll, CountBombs0_1, CountBombsFourOfKind0_1, CountBombsStraights0_1,
};
use crate::enumerate_hands::count_special_card_invariant_property;

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
