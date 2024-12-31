//Calculate probability of transition from given first 8 cards to first 14 cards:
// Only a subset of all properties is looked at:
// 1. Number of Aces: 5 options
// 2. Which special cards 16 options
// => There are 80 options in total
//We calculate the probability P(A|B) where A denotes some of the 80 options under the first 14 cards given that B is some option of the first 8 cards.

#[derive(Clone)]
pub struct HandQuantifier{
    num_aces: usize,
    phoenix: bool,
    dragon: bool,
    mahjong: bool,
    dog: bool
}

pub fn get_transition_probability(from_first_8: HandQuantifier, to_first_14: HandQuantifier){

}

