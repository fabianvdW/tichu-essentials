use crate::countable_properties::{CountableProperty, Counter};
use crate::hand;
use crate::tichu_hand::*;
use std::time::Instant;

pub fn count_special_card_sensitive_property<
    P: CountableProperty,
    const TARGET_NUM_CARDS: u32,
>(
    kind: P,
) -> Counter<P> {
    let start = Instant::now();
    let mut global_counter = Counter::new(kind.clone());
    for special_card_bits in 0..16 {
        let mut local_counter = Counter::new(kind.clone());
        let mut other_cards = [0; 13];
        let special_card_hand = unsafe {
            use std::arch::x86_64::_pdep_u64;
            _pdep_u64(special_card_bits, hand!(DRAGON, PHOENIX, MAHJONG, DOG))
        };
        count_property_recursive_upwards::<P, TARGET_NUM_CARDS>(
            &mut other_cards,
            special_card_hand,
            special_card_hand.count_ones(),
            0,
            &mut local_counter,
        );
        global_counter = global_counter + local_counter;
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

pub fn count_special_card_invariant_property<
    P: CountableProperty,
    const TARGET_NUM_CARDS: u32,
>(
    kind: P,
) -> Counter<P> {
    let start = Instant::now();
    let special_card_amount_to_frequency: [u64; 5] = [1, 4, 6, 4, 1];
    let mut global_counter = Counter::new(kind.clone());
    let special_card_hands = [0u64, hand!(DRAGON), hand!(DRAGON, PHOENIX), hand!(DRAGON,PHOENIX, DOG), hand!(DRAGON,PHOENIX,DOG, MAHJONG)];
    for special_card_amount in (0usize..=4).rev() {
        let mut local_counter = Counter::new(kind.clone());
        let mut other_cards = [0; 13];

        count_property_recursive_upwards::<P, TARGET_NUM_CARDS>(
            &mut other_cards,
            special_card_hands[special_card_amount],
            special_card_amount as u32,
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

#[inline(always)]
fn count_property_recursive_upwards<P: CountableProperty, const TARGET_NUM_CARDS: u32>(
    other_cards: &mut [u32; 13],
    special_card_hand: Hand,
    cards_sum: u32,
    current_index: usize,
    counter: &mut Counter<P>,
) {
    if cards_sum == TARGET_NUM_CARDS {
        count_property_recursive_downwards::<P, TARGET_NUM_CARDS>(
            other_cards,
            special_card_hand,
            current_index,
            false,
            false,
            false,
            counter,
        );
        return;
    }
    if cards_sum > TARGET_NUM_CARDS || current_index >= 13 || cards_sum + (13 - current_index as u32) * 4 < TARGET_NUM_CARDS {
        return;
    }
    for card_amount in 0..=4 {
        other_cards[current_index] = card_amount;
        count_property_recursive_upwards::<P, TARGET_NUM_CARDS>(
            other_cards,
            special_card_hand,
            cards_sum + card_amount,
            current_index + 1,
            counter,
        );
    }
}

#[inline(always)]
const fn current_index_to_card(current_index: usize) -> CardIndex {
    current_index as CardIndex
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
#[inline(always)]
fn count_property_recursive_downwards<P: CountableProperty, const TARGET_NUM_CARDS: u32>(
    other_cards: &[u32; 13],
    hand: u64,
    current_index: usize,
    yellow_lex_gr_blue: bool,
    blue_lex_gr_green: bool,
    green_lex_gr_red: bool,
    counter: &mut Counter<P>,
) {
    if hand.count_ones() == TARGET_NUM_CARDS {
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
    macro_rules! cases {($ ($condition: expr, $add: expr, $yl_bl: expr, $bl_gr: expr, $gr_rd: expr); +) => {$(
            if $condition{
                count_property_recursive_downwards::<P, TARGET_NUM_CARDS>(other_cards, hand ^ $add, current_index-1,
                $yl_bl || yellow_lex_gr_blue, $bl_gr || blue_lex_gr_green, $gr_rd || green_lex_gr_red, counter);
            }
        )+};}
    match other_cards[current_index - 1] {
        0 => {
            cases!(true, 0, false, false, false); //Changing nothing changes no lex order.
        }
        4 => {
            #[rustfmt::skip]
            cases!(true,MASK_FOUR_OF_KIND[current_index - 1], false, false, false);
            //Changing all changes no lex order.
        }
        1 => {
            let card: CardIndex = current_index_to_card(current_index);
            cases!(
                true, hand!(card+YELLOW), true, false, false; //Yellow is always an option, which implies yellow_lex_gr_blue.
                yellow_lex_gr_blue, hand!(card+BLUE) ,true, true, false;//Blue is an option if yellow_lex_gr_blue, which implies blue_lex_gr_green.
                blue_lex_gr_green, hand!(card+GREEN), false, true, true;//Green is an option if blue_lex_gr_green, which implies green_lex_gr_red.
                green_lex_gr_red, hand!(card+RED), false, false, true//Red is an option if green_lex_gr_red.
            );
        }
        3 => {
            let card: CardIndex = current_index_to_card(current_index);
            cases!(
                yellow_lex_gr_blue, hand!(card+BLUE, card+GREEN, card+RED), true, false, false;//Not yellow is an option if yellow_lex_gr_blue.
                blue_lex_gr_green, hand!(card+YELLOW, card+GREEN, card+RED), true, true, false;//Not blue is an option if blue_lex_gr_green, which implies yellow_lex_gr_blue.
                green_lex_gr_red, hand!(card+YELLOW, card+BLUE, card+RED), false, true, true;//Not green is an option if green_lex_gr_red_resolved, which implies blue_lex_gr_green
                true, hand!(card + YELLOW, card + BLUE, card + GREEN), false, false, true//Not red is always an option, and it resolves green_red;
            );
        }
        2 => {
            let card: CardIndex = current_index_to_card(current_index);
            cases!(
                true, hand!(card + YELLOW, card + BLUE), false, true, false;//Yellow+Blue is always an option, and it implies BLUE lex > GREEN
                blue_lex_gr_green, hand!(card + YELLOW, card + GREEN), true, true, true;//Yellow+ Green is an option only if blue lex > green, and it resolves YELLOW lex > BLUE and GREEN lex > RED
                green_lex_gr_red, hand!(card + YELLOW, card + RED), true, false, true; //Yellow+Red is an option only if green lex > red , and it implies YELLOW lex > BLUE
                yellow_lex_gr_blue, hand!(card + BLUE, card + GREEN), true, false, true;//Blue+Green is an option only if yellow lex > blue, and it implies green lex > red
                yellow_lex_gr_blue && green_lex_gr_red, hand!(card + BLUE, card + RED), true, true, true; //Blue + Red is an option only if yellow lex > blue and green lex > red , and it implies blue lex > green
                blue_lex_gr_green, hand!(card + GREEN, card + RED), false, true, false//Green+Red is an option only if blue lex > green and implies nothing.
            );
        }
        _ => unreachable!(),
    }
}
