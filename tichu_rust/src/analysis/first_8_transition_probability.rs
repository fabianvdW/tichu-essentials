//Calculate probability of transition from given first 8 cards to first 14 cards:
//We calculate the probability P(A|B) where A denotes some of the 80 options under the first 14 cards given that B is some option of the first 8 cards.

use crate::analysis::gt_stats::HandCategory;

pub fn calculate_transition_probabilities(){
    let mut transition_matrix = [[0.;80];80];
    for cat_1 in 0..80{
        for cat_2 in 0..80{
            transition_matrix[cat_1][cat_2]= get_transition_probability(HandCategory(cat_1), HandCategory(cat_2));
        }
    }
    println!("{:?}", transition_matrix);
}
pub fn get_transition_probability(from_first_8: HandCategory, to_first_14: HandCategory) -> f64 {
    if from_first_8.num_aces() > to_first_14.num_aces() || from_first_8.0 & to_first_14.0 & 0b1111 != from_first_8.0 & 0b1111 {
        return 0.;
    }
    pr(9, HandCategory::construct(
        to_first_14.num_aces() - from_first_8.num_aces(),
        to_first_14.has_dragon() & !from_first_8.has_dragon(),
        to_first_14.has_phoenix() & !from_first_8.has_phoenix(),
        to_first_14.has_dog() & !from_first_8.has_dog(),
        to_first_14.has_mahjong() & !from_first_8.has_mahjong(),
    ), from_first_8)
}

pub fn pr(current_index: usize, category_diff: HandCategory, category_dist: HandCategory) -> f64 {
    //Probability of distributing the remaining category_diff cards into the hand (when current_index cards haven been distributed
    // and out of those, category_dist are distributed cards from our category)
    if current_index == 15 {
        if category_diff.0 == 0 {
            return 1.;
        } else {
            return 0.;
        }
    }
    let mut res = 0.;
    if category_diff.num_aces() >= 1 {
        //Probability of current card being ace, given category_dist and index:
        let p_card_ace = (4 - category_dist.num_aces()) as f64 / (56 - current_index + 1) as f64;
        res += pr(current_index + 1, HandCategory::construct(
            category_diff.num_aces() - 1,
            category_diff.has_dragon(),
            category_diff.has_phoenix(),
            category_diff.has_dog(),
            category_diff.has_mahjong(),
        ), HandCategory::construct(
            category_dist.num_aces() + 1,
            category_dist.has_dragon(),
            category_dist.has_phoenix(),
            category_dist.has_dog(),
            category_dist.has_mahjong(),
        )) * p_card_ace;
    }
    for special_card in 0..4 {
        let special_card_in_diff = (category_diff.0 >> special_card) & 0b1 != 0;
        if !special_card_in_diff {
            continue;
        }
        //Probability of current card being special card, given category_dist and index:
        assert_eq!((category_dist.0 >> special_card) & 0b1, 0);
        let p_card_special_card = 1. / (56 - current_index + 1) as f64;
        res += pr(current_index + 1, HandCategory(
            category_diff.0 ^ (1 << special_card)
        ),
                  HandCategory(
                      category_dist.0 ^ (1 << special_card)
                  )) * p_card_special_card;
    }
    //Probability of current card being neither ace nor special card, which changes nothing of diff and dist
    let p_other_card = 1. - (8 - category_dist.num_aces() - (category_dist.0 & 0b1111).count_ones() as usize) as f64 / (56 - current_index + 1) as f64;
    res + pr(current_index + 1, category_diff, category_dist) * p_other_card
}