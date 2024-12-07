use crate::countable_properties::{CountAll, CounterBombs0_1, CounterBombsFourOfKind0_1, CounterBombsStraights0_1};
use crate::enumerate_hands::count_special_card_invariant_property;

pub fn count_tichu_hands(){
    count_special_card_invariant_property(CountAll); //5804731963800
}
pub fn count_bombs_0_1(){
    count_special_card_invariant_property(CounterBombs0_1); // 294663199638 out of 5804731963800 (0.0507625849868014)
}
pub fn count_four_of_kind_bombs_0_1(){
    count_special_card_invariant_property(CounterBombsFourOfKind0_1); //204703407480 out of 5804731963800 (0.035264919854455)
}
pub fn count_straight_bombs_0_1(){
    count_special_card_invariant_property(CounterBombsStraights0_1); // 92272299834 out of 5804731963800 (0.0158960483290937)
}