use colored::Colorize;
use phf::phf_map;

// Generic trait each datastructure for a Tichu Hand should implement
pub trait TichuHand {
    fn contains_straight_bomb(&self) -> bool;

    fn contains_four_of_kind_bomb(&self) -> bool;
    fn pretty_print(&self) -> String;
    fn debug_print(&self) -> String;
}

// Actual data structure we use is a u64:
pub type Hand = u64;
#[macro_export]
macro_rules! hand {
   ($ ($x: expr), +) => {
        {
            let mut temp : Hand = 0u64;
            $(
                temp |=  1u64 << $x;
            )*
            temp
        }
   };
}
// ----------------------- Cards and CardIndex -------------------
pub type CardIndex = usize;

pub const YELLOW: CardIndex = 0;
pub const BLUE: CardIndex = 16;
pub const GREEN: CardIndex = 32;
pub const RED: CardIndex = 48;

pub const DOG: CardIndex = 0;
pub const PHOENIX: CardIndex = 16;
pub const DRAGON: CardIndex = 32;
pub const MAHJONG: CardIndex = 48;

pub const TWO: CardIndex = 1;
pub const THREE: CardIndex = 2;
pub const FOUR: CardIndex = 3;
pub const FIVE: CardIndex = 4;
pub const SIX: CardIndex = 5;
pub const SEVEN: CardIndex = 6;
pub const EIGHT: CardIndex = 7;
pub const NINE: CardIndex = 8;
pub const TEN: CardIndex = 9;
pub const JACK: CardIndex = 10;
pub const QUEEN: CardIndex = 11;
pub const KING: CardIndex = 12;
pub const ACE: CardIndex = 13;

pub const fn get_color(card: CardIndex) -> CardIndex {
    debug_assert!((1u64 << card) & MASK_SPECIAL_CARDS == 0u64); //Special cards don't really have a color
    (card >> 4) * 16
}

pub const fn get_card_type(card: CardIndex) -> CardIndex {
    debug_assert!((1u64 << card) & MASK_SPECIAL_CARDS == 0u64); //Special cards have card type 0 which does not allow for distinguishment
    card & 0b1111
}

//----------------------------Masks----------------------
pub const MASK_SPECIAL_CARDS: Hand = hand!(DOG, PHOENIX, DRAGON, MAHJONG);
pub const MASK_NORMAL_CARDS: Hand = !MASK_SPECIAL_CARDS;

pub const MASK_TWOS: Hand = hand!(TWO + YELLOW, TWO + BLUE, TWO + GREEN, TWO + RED);
pub const MASK_THREES: Hand = hand!(THREE + YELLOW, THREE + BLUE, THREE + GREEN, THREE + RED);
pub const MASK_FOURS: Hand = hand!(FOUR + YELLOW, FOUR + BLUE, FOUR + GREEN, FOUR + RED);
pub const MASK_FIVES: Hand = hand!(FIVE + YELLOW, FIVE + BLUE, FIVE + GREEN, FIVE + RED);
pub const MASK_SIXS: Hand = hand!(SIX + YELLOW, SIX + BLUE, SIX + GREEN, SIX + RED);
pub const MASK_SEVENS: Hand = hand!(SEVEN + YELLOW, SEVEN + BLUE, SEVEN + GREEN, SEVEN + RED);
pub const MASK_EIGHTS: Hand = hand!(EIGHT + YELLOW, EIGHT + BLUE, EIGHT + GREEN, EIGHT + RED);
pub const MASK_NINES: Hand = hand!(NINE + YELLOW, NINE + BLUE, NINE + GREEN, NINE + RED);
pub const MASK_TENS: Hand = hand!(TEN + YELLOW, TEN + BLUE, TEN + GREEN, TEN + RED);
pub const MASK_JACKS: Hand = hand!(JACK + YELLOW, JACK + BLUE, JACK + GREEN, JACK + RED);
pub const MASK_QUEENS: Hand = hand!(QUEEN + YELLOW, QUEEN + BLUE, QUEEN + GREEN, QUEEN + RED);
pub const MASK_KINGS: Hand = hand!(KING + YELLOW, KING + BLUE, KING + GREEN, KING + RED);
pub const MASK_ACES: Hand = hand!(ACE + YELLOW, ACE + BLUE, ACE + GREEN, ACE + RED);

//--------------------------------------------------------------------------
static TICHU_ONE_ENCODING: phf::Map<char, CardIndex> = phf_map! {
    '6' => DRAGON,
    '5' => PHOENIX,
    '4' => ACE+RED,
    '3' => ACE+GREEN,
    '2' => ACE+BLUE,
    '1' => ACE+YELLOW,
    'X' => KING+RED,
    'W' => KING+GREEN,
    'V' => KING+BLUE,
    'U' => KING+YELLOW,
    'T' => QUEEN+RED,
    'S' => QUEEN+GREEN,
    'R' => QUEEN+BLUE,
    'Q' => QUEEN+YELLOW,
    'P' => JACK+RED,
    'O' => JACK+GREEN,
    'N' => JACK+BLUE,
    'M' => JACK+YELLOW,
    'L' => TEN+RED,
    'K' => TEN+GREEN,
    'J' => TEN+BLUE,
    'I' => TEN+YELLOW,
    'H' => NINE+RED,
    'G' => NINE+GREEN,
    'F' => NINE+BLUE,
    'E' => NINE+YELLOW,
    'D' => EIGHT+RED,
    'C' => EIGHT+GREEN,
    'B' => EIGHT+BLUE,
    'A' => EIGHT+YELLOW,
    'z' => SEVEN+RED,
    'y' => SEVEN+GREEN,
    'x' => SEVEN+BLUE,
    'w' => SEVEN+YELLOW,
    'v' => SIX+RED,
    'u' => SIX+GREEN,
    't' => SIX+BLUE,
    's' => SIX+YELLOW,
    'r' => FIVE+RED,
    'q' => FIVE+GREEN,
    'p' => FIVE+BLUE,
    'o' => FIVE+YELLOW,
    'n' => FOUR+RED,
    'm' => FOUR+GREEN,
    'l' => FOUR+BLUE,
    'k' => FOUR+YELLOW,
    'j' => THREE+RED,
    'i' => THREE+GREEN,
    'h' => THREE+BLUE,
    'g' => THREE+YELLOW,
    'f' => TWO+RED,
    'e' => TWO+GREEN,
    'd' => TWO+BLUE,
    'c' => TWO+YELLOW,
    'b' => MAHJONG,
    'a' => DOG
};

pub fn tichu_one_str_to_hand(hand_str: &str) -> Hand {
    let mut hand: Hand = 0u64;
    for c in hand_str.chars() {
        hand |= 1u64 << TICHU_ONE_ENCODING[&c];
    }
    hand
}
//TODO Ideas: Magic bitboards for determining which street bomb in case of bombs
// For Bomb detection: Be smart about special cards(they don't influence bombs) (52 choose 14) + (52 choose 13) + ... + (52 choose 10) << (56 choose 14).

static CARD_TO_CHAR: phf::Map<u32, &'static str> = phf_map! {
    0u32 => "↺",
    16u32 => "🐦",
    32u32 => "🐉",
    48u32 => "1",
    1u32 => "2",
    2u32 => "3",
    3u32 => "4",
    4u32 => "5",
    5u32 => "6",
    6u32 => "7",
    7u32 => "8",
    8u32 => "9",
    9u32 => "T",
    10u32 => "J",
    11u32 => "Q",
    12u32 => "K",
    13u32 => "A"
};
pub fn card_to_colored_string(card: CardIndex) -> String {
    if (1u64 << card) & MASK_SPECIAL_CARDS != 0u64 {
        CARD_TO_CHAR[&(card as u32)].to_string()
    } else {
        let card_in_char = CARD_TO_CHAR[&(get_card_type(card) as u32)];
        match get_color(card) {
            YELLOW => card_in_char.yellow().to_string(),
            BLUE => card_in_char.blue().to_string(),
            GREEN => card_in_char.green().to_string(),
            RED => card_in_char.red().to_string(),
            _ => unreachable!(),
        }
    }
}

impl TichuHand for Hand {
    fn contains_straight_bomb(&self) -> bool {
        let straight_cards = self & MASK_NORMAL_CARDS;
        (straight_cards & (straight_cards << 1) & (straight_cards << 2) & (straight_cards << 3) & (straight_cards << 4)) != 0u64
    }

    fn contains_four_of_kind_bomb(&self) -> bool {
        (self & MASK_TWOS).count_ones() == 4
            || (self & MASK_THREES).count_ones() == 4
            || (self & MASK_FOURS).count_ones() == 4
            || (self & MASK_FIVES).count_ones() == 4
            || (self & MASK_SIXS).count_ones() == 4
            || (self & MASK_SEVENS).count_ones() == 4
            || (self & MASK_EIGHTS).count_ones() == 4
            || (self & MASK_NINES).count_ones() == 4
            || (self & MASK_TENS).count_ones() == 4
            || (self & MASK_JACKS).count_ones() == 4
            || (self & MASK_QUEENS).count_ones() == 4
            || (self & MASK_KINGS).count_ones() == 4
            || (self & MASK_ACES).count_ones() == 4
    }

    fn pretty_print(&self) -> String {
        let mut res_str = String::new();
        for y in 0..16 {
            for x in 0..4 {
                let shift: CardIndex = (y + 1) % 16 + 16 * x;
                if ((self >> shift) & 1u64) != 0u64 {
                    res_str.push_str(&card_to_colored_string(shift));
                }
            }
        }
        res_str
    }

    fn debug_print(&self) -> String {
        let mut res_str: String = String::new();
        for y in 0..14 {
            res_str.push_str("|");
            for x in 0..4 {
                let shift: CardIndex = 61 - (y + 16 * x);
                res_str.push_str(&format!("\t{} ", shift));
                if ((self >> shift) & 1u64) != 0u64 {
                    res_str.push_str(&card_to_colored_string(shift));
                } else {
                    res_str.push_str(" ");
                }
                res_str.push_str("\t|");
            }
            res_str.push_str("\n");
        }
        res_str
    }
}
