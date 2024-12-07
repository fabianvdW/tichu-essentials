use crate::tichu_hand::{Hand, TichuHand};

use generic_array::{typenum, ArrayLength, GenericArray};
pub trait CountableProperty {
    type UpperBound: ArrayLength;
    fn count(&self, hand: &Hand) -> usize;
}
pub struct Counter<P: CountableProperty> {
    property: P,
    hands_evaluated: u64,
    hands_counted: u64,
    property_counted: GenericArray<u64, P::UpperBound>,
}
impl<P: CountableProperty> Counter<P> {
    fn new(property: P) -> Self {
        Counter {
            property,
            hands_evaluated: 0,
            hands_counted: 0,
            property_counted: GenericArray::default(),
        }
    }
}

struct CounterBombs0_1;
struct CounterBombsFourOfKind0_1;
struct CounterBombsStraights0_1;

impl CountableProperty for CounterBombs0_1 {
    type UpperBound = typenum::U2;
    fn count(&self, hand: &Hand) -> usize {
        (hand.contains_four_of_kind_bomb() || hand.contains_straight_bomb()) as usize
    }
}

impl CountableProperty for CounterBombsFourOfKind0_1 {
    type UpperBound = typenum::U2;
    fn count(&self, hand: &Hand) -> usize {
        hand.contains_four_of_kind_bomb() as usize
    }
}

impl CountableProperty for CounterBombsStraights0_1 {
    type UpperBound = typenum::U2;
    fn count(&self, hand: &Hand) -> usize {
        hand.contains_straight_bomb() as usize
    }
}
