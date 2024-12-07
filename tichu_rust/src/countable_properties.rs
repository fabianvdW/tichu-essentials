use crate::tichu_hand::{Hand, TichuHand};
use std::fmt;
use std::fmt::Debug;
use std::ops::{Add, Mul};

use generic_array::{typenum, ArrayLength, GenericArray};

pub trait CountableProperty: Debug + Clone {
    type UpperBound: ArrayLength;
    fn count(&self, hand: &Hand) -> usize;
}
pub struct Counter<P: CountableProperty> {
    pub property: P,
    pub hands_evaluated: u64,
    pub hands_counted: u64,
    pub property_counted: GenericArray<u64, P::UpperBound>,
}
impl<P: CountableProperty> Counter<P> {
    pub fn new(property: P) -> Self {
        Counter {
            property,
            hands_evaluated: 0,
            hands_counted: 0,
            property_counted: GenericArray::default(),
        }
    }

    pub fn count_hand(&mut self, hand: &Hand, hand_multiplicity: u64) {
        self.hands_evaluated += 1;
        self.hands_counted += hand_multiplicity;
        self.property_counted[self.property.count(hand)] += hand_multiplicity;
    }
}
impl<P: CountableProperty> Add for Counter<P> {
    type Output = Self;

    fn add(mut self, other: Self) -> Self {
        for (i, value) in other.property_counted.iter().enumerate() {
            self.property_counted[i] += value
        }
        Self {
            property: self.property,
            hands_evaluated: self.hands_evaluated + other.hands_evaluated,
            hands_counted: self.hands_counted + other.hands_counted,
            property_counted: self.property_counted,
        }
    }
}

impl<P: CountableProperty> Mul<u64> for Counter<P> {
    type Output = Self;
    fn mul(mut self, other: u64) -> Self {
        for i in 0..self.property_counted.len() {
            self.property_counted[i] *= other;
        }
        Self {
            property: self.property,
            hands_evaluated: self.hands_evaluated,
            hands_counted: self.hands_counted * other,
            property_counted: self.property_counted,
        }
    }
}

impl<P: CountableProperty> fmt::Display for Counter<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut res_str = String::new();
        res_str.push_str(&format!(
            "----------Counter for {:?}-----------\n",
            self.property
        ));
        res_str.push_str(&format!("Hands evaluated: {}\n", self.hands_evaluated));
        res_str.push_str(&format!("Hands counted: {}\n", self.hands_counted));
        res_str.push_str(&format!("Property counted: {:?}\n", self.property_counted));
        write!(f, "{}", res_str)
    }
}

#[derive(Debug, Clone)]
pub struct CountAll; //Every Hand passes this
#[derive(Debug, Clone)]
pub struct CounterBombs0_1; //Determine if a hand contains at least one bomb or not

#[derive(Debug, Clone)]
pub struct CounterBombsFourOfKind0_1; //Determine if a hand contains at least one four of kind bomb or not

#[derive(Debug, Clone)]
pub struct CounterBombsStraights0_1; //Determine if a hand contains a straight bomb.

impl CountableProperty for CountAll {
    type UpperBound = typenum::U1;

    fn count(&self, _: &Hand) -> usize {
        0
    }
}
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
