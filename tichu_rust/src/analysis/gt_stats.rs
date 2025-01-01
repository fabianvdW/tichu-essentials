use crate::bsw_binary_format::binary_format_constants::{PlayerIDInternal, CALL_GRAND_TICHU, RANK_1};
use crate::bsw_database::DataBase;
use crate::hand;
use crate::tichu_hand::{Hand, DOG, DRAGON, MAHJONG, MASK_ACES, PHOENIX};

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
    let mut gt_calls = [[0; 4]; 80];
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
                }
            }
        }
    }
    println!("GT Calls abs: {:?}", gt_calls);
    println!("GT Round scoree diff abs: {:?}", gt_round_score_diff_by_cat);
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