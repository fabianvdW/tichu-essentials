use colored::Colorize;
use phf::phf_map;

// Generic trait each datastructure for a Tichu Hand should implement
pub trait TichuHand {
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
pub type CardIndex = usize;

pub const YELLOW: CardIndex = 0;
pub const BLUE: CardIndex = 1;
pub const GREEN: CardIndex = 2;
pub const RED: CardIndex = 3;
pub const fn get_color(card: CardIndex) -> CardIndex {
    debug_assert!(card > LARGEST_SPECIAL_CARD); //Special cards don't really have a color
    card % 4
}

pub const DOG: CardIndex = 0;
pub const PHOENIX: CardIndex = 1;
pub const DRAGON: CardIndex = 2;
pub const MAHJONG: CardIndex = 3;
pub const LARGEST_SPECIAL_CARD: CardIndex = 3;

pub const TWO: CardIndex = 4;
pub const THREE: CardIndex = 8;
pub const FOUR: CardIndex = 12;
pub const FIVE: CardIndex = 16;
pub const SIX: CardIndex = 20;
pub const SEVEN: CardIndex = 24;
pub const EIGHT: CardIndex = 28;
pub const NINE: CardIndex = 32;
pub const TEN: CardIndex = 36;
pub const JACK: CardIndex = 40;
pub const QUEEN: CardIndex = 44;
pub const KING: CardIndex = 48;
pub const ACE: CardIndex = 52;

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
//TODO Ideas: Magic bitboards for street detection
// For Bomb detection: Be smart about special cards(they don't influence bombs) (52 choose 14) + (52 choose 13) + ... + (52 choose 10) << (56 choose 14).
// Serialization for Hand's

static CARD_TO_CHAR: phf::Map<u32, &'static str> = phf_map! {
    0u32 => "↺",
    1u32 => "🐦",
    2u32 => "🐉",
    3u32 => "1",
    4u32 => "2",
    8u32 => "3",
    12u32 => "4",
    16u32 => "5",
    20u32 => "6",
    24u32 => "7",
    28u32 => "8",
    32u32 => "9",
    36u32 => "T",
    40u32 => "J",
    44u32 => "Q",
    48u32 => "K",
    52u32 => "A"
};
pub fn card_to_colored_string(card: CardIndex) -> String {
    let card_in_char = CARD_TO_CHAR[&(get_card_type(card) as u32)];
    if card <= LARGEST_SPECIAL_CARD {
        card_in_char.to_string()
    } else {
        match get_color(card) {
            YELLOW => card_in_char.yellow().to_string(),
            BLUE => card_in_char.blue().to_string(),
            GREEN => card_in_char.green().to_string(),
            RED => card_in_char.red().to_string(),
            _ => unreachable!(),
        }
    }
}

pub const fn get_card_type(card: CardIndex) -> CardIndex {
    if card <= LARGEST_SPECIAL_CARD {
        card
    } else {
        card - get_color(card)
    }
}

impl TichuHand for Hand {
    fn pretty_print(&self) -> String {
        let mut res_str = String::new();
        for y in 0..14 {
            for x in 0..4 {
                let shift: CardIndex = y * 4 + x;
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
                let shift: CardIndex = 55 - (y * 4 + x);
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
