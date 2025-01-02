use generic_array::{typenum, GenericArray};
use generic_array::typenum::U80;
use crate::bsw_binary_format::binary_format_constants::{PlayerIDInternal, CALL_GRAND_TICHU, RANK_1};
use crate::bsw_database::DataBase;
use crate::countable_properties::CountableProperty;
use crate::enumerate_hands::count_special_card_sensitive_property;
use crate::hand;
use crate::tichu_hand::{Hand, DOG, DRAGON, MAHJONG, MASK_ACES, MASK_KINGS, PHOENIX};

#[derive(Clone)]
pub struct HandCategory(pub usize);
impl HandCategory {
    pub fn categorize_hand(hand: &Hand) -> Self {
        let num_aces = (hand & MASK_ACES).count_ones() as usize;
        let has_dragon = (hand & hand!(DRAGON)) != 0;
        let has_phoenix = (hand & hand!(PHOENIX)) != 0;
        let has_dog = (hand & hand!(DOG)) != 0;
        let has_mahjong = (hand & hand!(MAHJONG)) != 0;
        HandCategory::construct(num_aces, has_dragon, has_phoenix, has_dog, has_mahjong)
    }
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

pub fn evaluate_gt_stats(db: &DataBase) {
    let mut gt_categories = [[0; 4]; 80];
    let mut gt_round_score_diff_by_cat = [[0i64; 4]; 80];
    let mut gt_round_score_diff_by_first14_cat = [[0i64; 4]; 80];
    let mut gt_calls = [[0; 4]; 80];
    let mut gt_calls_first14 = [[0;4]; 80];
    let mut gt_successes = [[0; 4]; 80];

    for game in db.games.iter() {
        for (round, _) in game.rounds.iter() {
            for player_id in 0..4 {
                let prh = &round.player_rounds[player_id];
                let category = HandCategory::categorize_hand(&prh.first_8);
                gt_categories[category.0][player_id] += 1;
                if prh.player_call(player_id as PlayerIDInternal) == CALL_GRAND_TICHU {
                    gt_calls[category.0][player_id] += 1;
                    gt_successes[category.0][player_id] += (prh.player_rank(player_id as PlayerIDInternal) == RANK_1) as usize;
                    gt_round_score_diff_by_cat[category.0][player_id] += prh.round_score_relative_gain() as i64;
                    let cat_14 = HandCategory::categorize_hand(&prh.first_14);
                    gt_round_score_diff_by_first14_cat[cat_14.0][player_id] += prh.round_score_relative_gain() as i64;
                    gt_calls_first14[cat_14.0][player_id] += 1;
                }
            }
        }
    }
    println!("GT Calls abs: {:?}", gt_calls);
    println!("GT Calls_14 abs: {:?}", gt_calls_first14);
    println!("GT Round scoree diff abs: {:?}", gt_round_score_diff_by_cat);
    println!("GT Round score diff (first 14) abs: {:?}", gt_round_score_diff_by_first14_cat);
    println!("GT Categories abs: {:?}", gt_categories);
    println!("GT Successes abs: {:?}", gt_successes);

    let mut non_gt_categories = [[0; 4]; 80];
    let mut non_gt_round_score_diff = [[0i64; 4]; 80];
    for game in db.games.iter() {
        for (round, _) in game.rounds.iter() {
            for player_id in 0..4 {
                let prh = &round.player_rounds[player_id];
                let category = HandCategory::categorize_hand(&prh.first_14);
                if prh.player_call(player_id as PlayerIDInternal) == CALL_GRAND_TICHU {
                    continue;
                }
                non_gt_categories[category.0][player_id] += 1;
                non_gt_round_score_diff[category.0][player_id] += prh.round_score_relative_gain() as i64;
            }
        }
    }

    println!("Non-gt Categories abs: {:?}", non_gt_categories);
    println!("Non-gt ERS abs: {:?}", non_gt_round_score_diff);
}

pub fn evaluate_gt_call_rates(hand_category_count: GenericArray<u64, U80>) {
    let gt_hands = 1420494075;
    //1. Strategy: Call GT with Ace + Joker or both Jokers
    let strat_1_hands = hand_category_count.iter().enumerate().filter(|(cat, _)| {
        let category = HandCategory(*cat);
        category.num_aces() >= 1 && (category.has_dragon() || category.has_phoenix()) || category.has_phoenix() && category.has_dragon()
    }).map(|x| x.1).sum::<u64>();
    //2. Strategy: Call GT with Ace + Joker + 1. or Dog or both Jokers + 1. or Dog
    let strat_2_hands = hand_category_count.iter().enumerate().filter(|(cat, _)| {
        let category = HandCategory(*cat);
        let dog_or_mahjong = category.has_dog() || category.has_mahjong();
        (category.num_aces() >= 1 && (category.has_dragon() || category.has_phoenix()) || category.has_phoenix() && category.has_dragon()) && dog_or_mahjong
    }).map(|x| x.1).sum::<u64>();
    //2. Strategy: Call GT with 2Ace + Joker or both Ace + Two Jokers
    let strat_3_hands = hand_category_count.iter().enumerate().filter(|(cat, _)| {
        let category = HandCategory(*cat);
        category.num_aces() >= 2 && (category.has_dragon() || category.has_phoenix()) || category.has_phoenix() && category.has_dragon() && category.num_aces() >= 1
    }).map(|x| x.1).sum::<u64>();

    //3. Strategy is custom formula, which we have to enumerate GT Hands with again.
    let strat_4_hands = count_special_card_sensitive_property::<CountCustomGTStrategy, 8>(CountCustomGTStrategy).property_counted[1];

    println!("GT Call Rate Strat 1(Ace+J or Two J): {}", strat_1_hands as f64 / gt_hands as f64);
    println!("GT Call Rate Strat 2(Ace+J + 1/Dog or Two J + 1/Dog): {}", strat_2_hands as f64 / gt_hands as f64);
    println!("GT Call Rate Strat 3(2Ace+J or Ace+Two J): {}", strat_3_hands as f64 / gt_hands as f64);
    println!("GT Call Rate Strat 4(Custom Formula): {}", strat_4_hands as f64 / gt_hands as f64);
}

#[derive(Debug, Clone)]
pub struct CountCustomGTStrategy;
impl CountableProperty for CountCustomGTStrategy {
    type UpperBound = typenum::U2;

    fn count(&self, hand: &Hand) -> usize {
        let num_aces = (hand & MASK_ACES).count_ones();
        let num_kings = (hand & MASK_KINGS).count_ones();
        let num_dragons = (hand & hand!(DRAGON)).count_ones();
        let num_phoenixs = (hand & hand!(PHOENIX)).count_ones();
        let num_mahjongs = (hand & hand!(MAHJONG)).count_ones();
        let num_dogs = (hand & hand!(DOG)).count_ones();
        (num_aces as f64 + num_kings as f64 * 0.05 + 2. * num_dragons as f64 + 1.9 * num_phoenixs as f64 + num_mahjongs as f64 * 0.01 + num_dogs as f64 * 0.01 >= 3.) as usize
    }
}