//Calculate probability of transition from given first 8 cards to first 14 cards:
// Only a subset of all properties is looked at:
// 1. Number of Aces: 5 options
// 2. Which special cards 16 options
// => There are 80 options in total
//We calculate the probability P(A|B) where A denotes some of the 80 options under the first 14 cards given that B is some option of the first 8 cards.

#[derive(Clone)]
pub struct HandCategory(pub usize);
impl HandCategory {
    pub fn print_category_lists() {
        let mut res = [(0, false, false, false, false); 80];
        for category in 0..80 {
            let hc = HandCategory(category);
            res[category] = (hc.num_aces(), hc.has_dragon(), hc.has_phoenix(), hc.has_dog(), hc.has_mahjong());
        }
        println!("{:?}", res)
    }
    pub fn construct(num_aces: usize, has_dragon: bool, has_phoenix: bool, has_dog: bool, has_mahjong: bool) -> HandCategory {
        HandCategory((num_aces << 4) + ((has_dragon as usize) << 3) + ((has_phoenix as usize) << 2) + ((has_dog as usize) << 1) + (has_mahjong as usize))
    }
    pub fn num_aces(&self) -> usize {
        self.0 >> 4
    }

    pub fn has_dragon(&self) -> bool {
        (self.0 >> 3) & 0b1 != 0
    }

    pub fn has_phoenix(&self) -> bool {
        (self.0 >> 2) & 0b1 != 0
    }

    pub fn has_dog(&self) -> bool {
        (self.0 >> 1) & 0b1 != 0
    }

    pub fn has_mahjong(&self) -> bool {
        self.0 & 0b1 != 0
    }
}


//pub fn get_transition_probability(from_first_8: HandCategory, to_first_14: HandCategory) {}

