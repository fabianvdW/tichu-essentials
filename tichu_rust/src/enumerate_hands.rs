use crate::countable_properties::{CountableProperty, Counter};
use crate::hand;
use crate::tichu_hand::*;
use std::time::Instant;

//First-8 Hands: 1420494075 (56 choose 8)
//Number >= 1 Bomb: 4229667,  approx 0.0029776027048898

pub fn count_special_card_invariant_property<
    P: CountableProperty,
    const TARGET_NUM_CARDS: usize,
>(
    kind: P,
) -> Counter<P> {
    let start = Instant::now();
    let special_card_amount_to_frequency: [u64; 5] = [1, 4, 6, 4, 1];
    let mut global_counter = Counter::new(kind.clone());
    for special_card_amount in (0usize..=4).rev() {
        let mut local_counter = Counter::new(kind.clone());
        let mut other_cards = [0usize; 13];
        count_property_recursive_upwards::<P, TARGET_NUM_CARDS>(
            &mut other_cards,
            special_card_amount,
            special_card_amount,
            0,
            &mut local_counter,
        );
        global_counter =
            global_counter + local_counter * special_card_amount_to_frequency[special_card_amount];
        println!("{}", global_counter);
    }
    let duration = start.elapsed();
    println!("Time elapsed is: {:?}", duration);
    println!(
        "Searched {:.2} hands per second",
        global_counter.hands_evaluated as f64 / duration.as_secs_f64()
    );
    global_counter
}

fn count_property_recursive_upwards<P: CountableProperty, const TARGET_NUM_CARDS: usize>(
    other_cards: &mut [usize; 13],
    special_card_amount: usize,
    cards_sum: usize,
    current_index: usize,
    counter: &mut Counter<P>,
) {
    if cards_sum == TARGET_NUM_CARDS {
        count_property_recursive_downwards::<P, TARGET_NUM_CARDS>(
            other_cards,
            0u64,
            special_card_amount,
            current_index,
            false,
            false,
            false,
            counter,
        );
        return;
    }
    if cards_sum > TARGET_NUM_CARDS || current_index >= 13 {
        return;
    }
    for card_amount in 0..=4 {
        other_cards[current_index] = card_amount;
        count_property_recursive_upwards::<P, TARGET_NUM_CARDS>(
            other_cards,
            special_card_amount,
            cards_sum + card_amount,
            current_index + 1,
            counter,
        );
    }
}

#[inline(always)]
const fn current_index_to_card(current_index: usize) -> CardIndex {
    current_index
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
fn count_property_recursive_downwards<P: CountableProperty, const TARGET_NUM_CARDS: usize>(
    other_cards: &[usize; 13],
    hand: u64,
    bits_set: usize,
    current_index: usize,
    yellow_lex_gr_blue: bool,
    blue_lex_gr_green: bool,
    green_lex_gr_red: bool,
    counter: &mut Counter<P>,
) {
    if bits_set == TARGET_NUM_CARDS {
        //Now, count number of different columns
        let column_equal_identifier: u8 = (!yellow_lex_gr_blue as u8)
            + ((!blue_lex_gr_green as u8) << 1)
            + ((!green_lex_gr_red as u8) << 2);
        let mult = COLUMN_EQUAL_ID_TO_COMBINATIONS[column_equal_identifier as usize];
        counter.count_hand(&hand, mult);
        return;
    }
    //We enforce yellow lex >= blue lex >= green lex >= red. If any of the bools is set to true, the property cant be violated anymore.
    //Explicit case enumeration, I did not have a better idea :( But it works! :) And it is straightforward to follow on a case by base basis
    macro_rules! cases {($ ($condition: expr, $add: expr, $bits_set: expr, $yl_bl: expr, $bl_gr: expr, $gr_rd: expr); +) => {$(
            if $condition{
                count_property_recursive_downwards::<P, TARGET_NUM_CARDS>(other_cards, hand ^ $add, bits_set + $bits_set, current_index-1,
                $yl_bl || yellow_lex_gr_blue, $bl_gr || blue_lex_gr_green, $gr_rd || green_lex_gr_red, counter);
            }
        )+};}
    match other_cards[current_index - 1] {
        0 => {
            cases!(true, 0, 0, false, false, false); //Changing nothing changes no lex order.
        }
        4 => {
            #[rustfmt::skip]
            cases!(true,MASK_FOUR_OF_KIND[current_index - 1], 4, false, false, false);
            //Changing all changes no lex order.
        }
        1 => {
            let card: CardIndex = current_index_to_card(current_index);
            cases!(
                true, hand!(card+YELLOW), 1, true, false, false; //Yellow is always an option, which implies yellow_lex_gr_blue.
                yellow_lex_gr_blue, hand!(card+BLUE), 1 ,true, true, false;//Blue is an option if yellow_lex_gr_blue, which implies blue_lex_gr_green.
                blue_lex_gr_green, hand!(card+GREEN), 1, false, true, true;//Green is an option if blue_lex_gr_green, which implies green_lex_gr_red.
                green_lex_gr_red, hand!(card+RED), 1, false, false, true//Red is an option if green_lex_gr_red.
            );
        }
        3 => {
            let card: CardIndex = current_index_to_card(current_index);
            cases!(
                yellow_lex_gr_blue, hand!(card+BLUE, card+GREEN, card+RED), 3, true, false, false;//Not yellow is an option if yellow_lex_gr_blue.
                blue_lex_gr_green, hand!(card+YELLOW, card+GREEN, card+RED), 3, true, true, false;//Not blue is an option if blue_lex_gr_green, which implies yellow_lex_gr_blue.
                green_lex_gr_red, hand!(card+YELLOW, card+BLUE, card+RED), 3, false, true, true;//Not green is an option if green_lex_gr_red_resolved, which implies blue_lex_gr_green
                true, hand!(card + YELLOW, card + BLUE, card + GREEN), 3, false, false, true//Not red is always an option, and it resolves green_red;
            );
        }
        2 => {
            let card: CardIndex = current_index_to_card(current_index);
            cases!(
                true, hand!(card + YELLOW, card + BLUE), 2, false, true, false;//Yellow+Blue is always an option, and it implies BLUE lex > GREEN
                blue_lex_gr_green, hand!(card + YELLOW, card + GREEN), 2, true, true, true;//Yellow+ Green is an option only if blue lex > green, and it resolves YELLOW lex > BLUE and GREEN lex > RED
                green_lex_gr_red, hand!(card + YELLOW, card + RED), 2, true, false, true; //Yellow+Red is an option only if green lex > red , and it implies YELLOW lex > BLUE
                yellow_lex_gr_blue, hand!(card + BLUE, card + GREEN), 2, true, false, true;//Blue+Green is an option only if yellow lex > blue, and it implies green lex > red
                yellow_lex_gr_blue && green_lex_gr_red, hand!(card + BLUE, card + RED), 2, true, true, true; //Blue + Red is an option only if yellow lex > blue and green lex > red , and it implies blue lex > green
                blue_lex_gr_green, hand!(card + GREEN, card + RED), 2, false, true, false//Green+Red is an option only if blue lex > green and implies nothing.
            );
        }
        _ => unreachable!(),
    }
}
