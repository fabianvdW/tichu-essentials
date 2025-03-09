use crate::tichu_hand::{Hand, TichuHand, BLUE, GREEN, MASK_ACES, MASK_NORMAL_CARDS, MASK_YELLOW, RED};
use std::fmt;
use std::fmt::Debug;
use std::ops::{Add, Mul};

use generic_array::{typenum, ArrayLength, GenericArray};
use crate::analysis::gt_stats::HandCategory;
use crate::hand;

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
pub struct CountBombs0_1; //Determine if a hand contains at least one bomb or not

#[derive(Debug, Clone)]
pub struct CountBombsFourOfKind0_1; //Determine if a hand contains at least one four of kind bomb or not

#[derive(Debug, Clone)]
pub struct CountBombsStraights0_1; //Determine if a hand contains a straight bomb.


#[derive(Debug, Clone)]
pub struct CountHandCategory;

#[derive(Debug, Clone)]
pub struct CountHasFourAces0_1;

#[derive(Debug, Clone)]
pub struct CountLongestStraight;

#[derive(Debug, Clone)]
pub struct CountLongestStraightFlush;

impl CountableProperty for CountAll {
    type UpperBound = typenum::U1;

    fn count(&self, _: &Hand) -> usize {
        0
    }
}
impl CountableProperty for CountBombs0_1 {
    type UpperBound = typenum::U2;
    fn count(&self, hand: &Hand) -> usize {
        (hand.contains_four_of_kind_bomb() || hand.contains_straight_bomb()) as usize
    }
}

impl CountableProperty for CountBombsFourOfKind0_1 {
    type UpperBound = typenum::U2;
    fn count(&self, hand: &Hand) -> usize {
        hand.contains_four_of_kind_bomb() as usize
    }
}

impl CountableProperty for CountBombsStraights0_1 {
    type UpperBound = typenum::U2;
    fn count(&self, hand: &Hand) -> usize {
        hand.contains_straight_bomb() as usize
    }
}

impl CountableProperty for CountHandCategory {
    type UpperBound = typenum::U80;
    fn count(&self, hand: &Hand) -> usize {
        HandCategory::categorize_hand(hand).0
    }
}
impl CountableProperty for CountHasFourAces0_1 {
    type UpperBound = typenum::U2;
    fn count(&self, hand: &Hand) -> usize {
        ((hand & MASK_ACES).count_ones() == 4) as usize
    }
}

impl CountableProperty for CountLongestStraight {
    type UpperBound = typenum::U13;
    fn count(&self, hand: &Hand) -> usize {
        let mut hand_in_yellow = ((hand >> BLUE) | (hand >> GREEN) | (hand >> RED) | hand) & MASK_YELLOW;
        let mut straight_length = 1;
        while hand_in_yellow & (hand_in_yellow >> 1) != 0 {
            straight_length += 1;
            hand_in_yellow = hand_in_yellow & (hand_in_yellow >> 1);
        }
        straight_length - 1
    }
}

impl CountableProperty for CountLongestStraightFlush {
    type UpperBound = typenum::U13;
    fn count(&self, hand: &Hand) -> usize {
        let mut hand = hand & MASK_NORMAL_CARDS;
        let mut straight_length = 1;
        while hand & (hand >> 1) != 0 {
            straight_length += 1;
            hand = hand & (hand >> 1);
        }
        straight_length - 1
    }
}