use crate::hand;
use crate::tichu_hand::*;
use std::time::{Instant};


const TARGET_NUM_CARDS: usize = 8;

//First Result!
//Tichu Hands: 5804731963800 = (56 choose 14)
//Number of Tichu Hands >= 1 Bomb: 118114016196,  approx. 0.02034788461
//First-8 Hands: 1420494075 (56 choose 8)
//Number >= 1 Bomb: 1536107,  approx 0.00108138923
struct BombCounter {
    hands_evaluated: u64,
    hands_counted: u64,
    bombs_counted: u64,
}

impl Default for BombCounter {
    fn default() -> Self {
        BombCounter {
            hands_evaluated: 0u64,
            hands_counted: 0u64,
            bombs_counted: 0u64,
        }
    }
}
pub fn count_bombs() {
    let start = Instant::now();
    let special_card_amount_to_frequency: [u64; 5] = [1, 4, 6, 4, 1];
    let mut global_counter = BombCounter::default();
    for special_card_amount in (0usize..=4).rev() {
        let mut local_counter = BombCounter::default();
        let mut other_cards = [0usize; 13];
        count_bombs_recursive_upwards(
            &mut other_cards,
            special_card_amount,
            special_card_amount,
            0,
            &mut local_counter,
        );
        global_counter.hands_evaluated += local_counter.hands_evaluated;
        global_counter.hands_counted +=
            local_counter.hands_counted * special_card_amount_to_frequency[special_card_amount];
        global_counter.bombs_counted +=
            local_counter.bombs_counted * special_card_amount_to_frequency[special_card_amount];
        println!("Hands evaluated: {}", global_counter.hands_evaluated);
        println!("Hands counted: {}", global_counter.hands_counted);
        println!("Bomb Hands counted: {}", global_counter.bombs_counted);
    }
    let duration = start.elapsed();
    println!("Time elapsed in expensive_function() is: {:?}", duration);
    println!("Searched {:.2} hands per second", global_counter.hands_evaluated as f64/ duration.as_secs_f64());
}

fn count_bombs_recursive_upwards(
    other_cards: &mut [usize; 13],
    special_card_amount: usize,
    cards_sum: usize,
    current_index: usize,
    bomb_counter: &mut BombCounter,
) {
    if cards_sum == TARGET_NUM_CARDS {
        count_bombs_downards_recursively(
            other_cards,
            0u64,
            special_card_amount,
            current_index - 1,
            false,
            false,
            false,
            false,
            bomb_counter,
        );
        return;
    }
    if cards_sum > TARGET_NUM_CARDS || current_index >= 13 {
        return;
    }
    for card_amount in 0..=4 {
        other_cards[current_index] = card_amount;
        count_bombs_recursive_upwards(
            other_cards,
            special_card_amount,
            cards_sum + card_amount,
            current_index + 1,
            bomb_counter,
        );
    }
}

const fn current_index_to_card(current_index: usize) -> CardIndex {
    current_index + 1
}
const COLUMN_EQUAL_ID_TO_COMBINATIONS: [u64; 8] = [24, 12, 12, 4, 12, 6, 4, 1];
//The bits state which columns are equal. First bit: yellow_blue, Second: blue_green, Third: green_red
//See also how this array is indexed. The value of this array gives the amount of different hands that are symmetrical to the given hand
//0b000=0 => 24
//0b001=1 => 12
//0b010=2 => 12
//0b011=3 => 4
//0b100=4 => 12
//0b101=5 => 6
//0b110=6 => 4
//0b111=7 => 1
fn count_bombs_downards_recursively(
    other_cards: &[usize; 13],
    hand: u64,
    bits_set: usize,
    current_index: usize,
    yellow_blue_resolved: bool,
    blue_green_resolved: bool,
    green_red_resolved: bool,
    four_of_kind_bomb: bool,
    bomb_counter: &mut BombCounter,
) {
    if bits_set == TARGET_NUM_CARDS {
        //Now, count number of different columns
        let column_equal_identifier: u8 = (!yellow_blue_resolved as u8) + ((!blue_green_resolved as u8)<< 1) + ((!green_red_resolved as u8) << 2);
        let mult = COLUMN_EQUAL_ID_TO_COMBINATIONS[column_equal_identifier as usize];
        bomb_counter.hands_evaluated += 1;
        bomb_counter.hands_counted += 1u64 * mult;
        bomb_counter.bombs_counted +=
            (four_of_kind_bomb || hand.contains_straight_bomb()) as u64 * mult;
        return;
    }

    //Explicit case enumeration,
    // I did not have a better idea :( But it works! :) And its straightforward to follow on a case by base basis
    match other_cards[current_index] {
        0 => {
            count_bombs_downards_recursively(
                other_cards,
                hand,
                bits_set,
                current_index - 1,
                yellow_blue_resolved,
                blue_green_resolved,
                green_red_resolved,
                four_of_kind_bomb,
                bomb_counter,
            );
        }
        4 => {
            count_bombs_downards_recursively(
                other_cards,
                hand ^ MASK_FOUR_OF_KIND[current_index],
                bits_set + 4,
                current_index - 1,
                yellow_blue_resolved,
                blue_green_resolved,
                green_red_resolved,
                true,
                bomb_counter,
            );
        }
        1 => {
            let card: CardIndex = current_index_to_card(current_index);
            //Yellow is always an option.
            count_bombs_downards_recursively(
                other_cards,
                hand ^ hand!(card + YELLOW),
                bits_set + 1,
                current_index - 1,
                true,
                blue_green_resolved,
                green_red_resolved,
                false,
                bomb_counter,
            );
            //Blue is an option if yellow_blue_resolved:
            if yellow_blue_resolved {
                count_bombs_downards_recursively(
                    other_cards,
                    hand ^ hand!(card + BLUE),
                    bits_set + 1,
                    current_index - 1,
                    true,
                    true,
                    green_red_resolved,
                    false,
                    bomb_counter,
                );
            }
            //Green is an option if blue_green resolved:
            if blue_green_resolved {
                count_bombs_downards_recursively(
                    other_cards,
                    hand ^ hand!(card + GREEN),
                    bits_set + 1,
                    current_index - 1,
                    yellow_blue_resolved,
                    true,
                    true,
                    false,
                    bomb_counter,
                );
            }
            //Red is an option if green_red_resolved:
            if green_red_resolved {
                count_bombs_downards_recursively(
                    other_cards,
                    hand ^ hand!(card + RED),
                    bits_set + 1,
                    current_index - 1,
                    yellow_blue_resolved,
                    blue_green_resolved,
                    true,
                    false,
                    bomb_counter,
                );
            }
        }
        3 => {
            let card: CardIndex = current_index_to_card(current_index);
            //Not yellow is an option if yellow_blue resolved:
            if yellow_blue_resolved {
                count_bombs_downards_recursively(
                    other_cards,
                    hand ^ hand!(card + BLUE, card + GREEN, card + RED),
                    bits_set + 3,
                    current_index - 1,
                    true,
                    blue_green_resolved,
                    green_red_resolved,
                    false,
                    bomb_counter,
                );
            }
            //Not blue is an option if blue_green_resolved, which resolves yellow_blue
            if blue_green_resolved {
                count_bombs_downards_recursively(
                    other_cards,
                    hand ^ hand!(card + YELLOW, card + GREEN, card + RED),
                    bits_set + 3,
                    current_index - 1,
                    true,
                    true,
                    green_red_resolved,
                    false,
                    bomb_counter,
                );
            }
            //Not green is an option if green_red_resolved, which resolves blue_green
            if green_red_resolved {
                count_bombs_downards_recursively(
                    other_cards,
                    hand ^ hand!(card + YELLOW, card + BLUE, card + RED),
                    bits_set + 3,
                    current_index - 1,
                    yellow_blue_resolved,
                    true,
                    true,
                    false,
                    bomb_counter,
                );
            }
            //Not red is always an option, and it resolves green_red;
            count_bombs_downards_recursively(
                other_cards,
                hand ^ hand!(card + YELLOW, card + BLUE, card + GREEN),
                bits_set + 3,
                current_index - 1,
                yellow_blue_resolved,
                blue_green_resolved,
                true,
                false,
                bomb_counter,
            );
        }
        2 => {
            let card: CardIndex = current_index_to_card(current_index);
            //Options: yellow+blue, yellow+green, yellow+red, blue+green, blue+red, green+red
            //Yellow+Blue is always an option, and it resolves BLUE > GREEN
            count_bombs_downards_recursively(
                other_cards,
                hand ^ hand!(card + YELLOW, card + BLUE),
                bits_set + 2,
                current_index - 1,
                yellow_blue_resolved,
                true,
                green_red_resolved,
                false,
                bomb_counter,
            );
            //Yellow+ Green is an option only if blue_green_resolved, and it resolves YELLOW > BLUE and GREEN > RED
            if blue_green_resolved {
                count_bombs_downards_recursively(
                    other_cards,
                    hand ^ hand!(card + YELLOW, card + GREEN),
                    bits_set + 2,
                    current_index - 1,
                    true,
                    true,
                    true,
                    false,
                    bomb_counter,
                );
            }
            //Yellow+Red is an option only if green_red is resolved, and it resolves YELLOW > BLUE
            if green_red_resolved {
                count_bombs_downards_recursively(
                    other_cards,
                    hand ^ hand!(card + YELLOW, card + RED),
                    bits_set + 2,
                    current_index - 1,
                    true,
                    blue_green_resolved,
                    true,
                    false,
                    bomb_counter,
                );
            }
            //Blue+Green is an option only if yellow_blue is resolved, and it resolves green > red
            if yellow_blue_resolved {
                count_bombs_downards_recursively(
                    other_cards,
                    hand ^ hand!(card + BLUE, card + GREEN),
                    bits_set + 2,
                    current_index - 1,
                    true,
                    blue_green_resolved,
                    true,
                    false,
                    bomb_counter,
                );
            }
            //Blue + Red is an option only if yellow_blue is resolved and green_red is resolved, and it resolves blue_green
            if yellow_blue_resolved && green_red_resolved {
                count_bombs_downards_recursively(
                    other_cards,
                    hand ^ hand!(card + BLUE, card + RED),
                    bits_set + 2,
                    current_index - 1,
                    true,
                    true,
                    true,
                    false,
                    bomb_counter,
                );
            }
            //Green+Red is an option only if blue_green is resolved and resolves nothing.
            if blue_green_resolved {
                count_bombs_downards_recursively(
                    other_cards,
                    hand ^ hand!(card + GREEN, card + RED),
                    bits_set + 2,
                    current_index - 1,
                    yellow_blue_resolved,
                    true,
                    green_red_resolved,
                    false,
                    bomb_counter,
                );
            }
        }
        _ => unreachable!(),
    }
}
