use colored::Colorize;
use phf::phf_map;
use crate::bsw_binary_format::binary_format_constants::Score;
use crate::pair_street_detection_trick::is_pair_street_fast;
use crate::street_detection_tricks::{is_street_fast};

// Generic trait each datastructure for a Tichu Hand should implement
pub trait TichuHand {
    fn get_lsb_card(&self) -> CardIndex;
    fn hand_type(&self) -> Option<HandType>;
    fn is_fullhouse(&self) -> Option<HandType>; //Only ever returns Fullhouse
    fn contains_straight_bomb(&self) -> bool;
    fn contains_four_of_kind_bomb(&self) -> bool;
    fn pretty_print(&self) -> String;
    fn debug_print(&self) -> String;
    fn pop_some_card(&mut self) -> CardIndex;

    fn get_card_points(&self) -> Score;

    fn get_high_card_amt(&self) -> u32;

    fn count_triplets(&self) -> u32;
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

//We use weak typing for everything here (mostly implicit Enums).
//TODO: Strong typing without runtime loss should be possible I think (with unsafe and debug assert for enum conversion)
// ----------------------- Cards and CardIndex -------------------
pub type CardIndex = u8;
pub type CardType = u8;
pub type Color = u8;

pub const YELLOW: Color = 0;
pub const BLUE: Color = 16;
pub const GREEN: Color = 32;
pub const RED: Color = 48;

pub const PHOENIX: CardType = 0; //Currently its important that the phoenix is always the lsb, see for instance hand_type

pub const DOG: CardIndex = 16;
pub const DRAGON: CardIndex = 32;
pub const MAHJONG: CardIndex = 48;

pub const SPECIAL_CARD: CardType = 0;
pub const TWO: CardType = 1;
pub const THREE: CardType = 2;
pub const FOUR: CardType = 3;
pub const FIVE: CardType = 4;
pub const SIX: CardType = 5;
pub const SEVEN: CardType = 6;
pub const EIGHT: CardType = 7;
pub const NINE: CardType = 8;
pub const TEN: CardType = 9;
pub const JACK: CardType = 10;
pub const QUEEN: CardType = 11;
pub const KING: CardType = 12;
pub const ACE: CardType = 13;

pub const fn get_color(card: CardIndex) -> Color {
    debug_assert!((1u64 << card) & MASK_SPECIAL_CARDS == 0u64); //Special cards don't really have a color
    (card >> 4) * 16
}

pub const fn get_card_type(card: CardIndex) -> CardType {
    //Returns 0 == Dog for all Special Cards!!
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

pub const MASK_FOUR_OF_KIND: [Hand; 13] = [MASK_TWOS, MASK_THREES, MASK_FOURS, MASK_FIVES, MASK_SIXS, MASK_SEVENS, MASK_EIGHTS, MASK_NINES, MASK_TENS, MASK_JACKS, MASK_QUEENS, MASK_KINGS, MASK_ACES, ];

pub const MASK_YELLOW: Hand = hand!(
    TWO + YELLOW,
    THREE + YELLOW,
    FOUR + YELLOW,
    FIVE + YELLOW,
    SIX + YELLOW,
    SEVEN + YELLOW,
    EIGHT + YELLOW,
    NINE + YELLOW,
    TEN + YELLOW,
    JACK + YELLOW,
    QUEEN + YELLOW,
    KING + YELLOW,
    ACE + YELLOW
);
pub const MASK_BLUE: Hand = MASK_YELLOW << BLUE;
pub const MASK_GREEN: Hand = MASK_YELLOW << GREEN;
pub const MASK_RED: Hand = MASK_YELLOW << RED;
pub const MASK_ALL: Hand = MASK_YELLOW
    | MASK_BLUE
    | MASK_GREEN
    | MASK_RED
    | MASK_SPECIAL_CARDS;

//--------------------------------------------------------------------------
pub static TICHU_ONE_ENCODING: phf::Map<char, CardIndex> = phf_map! {
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

static CARD_TO_CHAR: phf::Map<CardIndex, &'static str> = phf_map! {
    16u8 => "↺",
    0u8 => "🐦",
    32u8 => "🐉",
    48u8 => "1",
    1u8 => "2",
    2u8 => "3",
    3u8 => "4",
    4u8 => "5",
    5u8 => "6",
    6u8 => "7",
    7u8 => "8",
    8u8 => "9",
    9u8 => "T",
    10u8 => "J",
    11u8 => "Q",
    12u8 => "K",
    13u8 => "A"
}; //If this is used only in non-speed related things, might be worth it to kick out the dependency and just use match.
pub fn card_to_colored_string(card: CardIndex) -> String {
    if (1u64 << card) & MASK_SPECIAL_CARDS != 0u64 {
        CARD_TO_CHAR[&card].to_string()
    } else {
        let card_in_char = CARD_TO_CHAR[&(get_card_type(card))];
        match get_color(card) {
            YELLOW => card_in_char.yellow().to_string(),
            BLUE => card_in_char.blue().to_string(),
            GREEN => card_in_char.green().to_string(),
            RED => card_in_char.red().to_string(),
            _ => unreachable!(),
        }
    }
}
//--------------------------------------TrickType + HandType--------------------------
pub type TrickType = u8;
pub const TRICK_SINGLETON: TrickType = 0;
pub const TRICK_PAIRS: TrickType = 1;
pub const TRICK_TRIPLETS: TrickType = 2;
pub const TRICK_PAIRSTREET4: TrickType = 4;
pub const TRICK_PAIRSTREET6: TrickType = 5;
pub const TRICK_PAIRSTREET8: TrickType = 6;
pub const TRICK_PAIRSTREET10: TrickType = 7;
pub const TRICK_PAIRSTREET12: TrickType = 8;
pub const TRICK_PAIRSTREET14: TrickType = 9;
pub const TRICK_STREET5: TrickType = 10;
pub const TRICK_STREET6: TrickType = 11;
pub const TRICK_STREET7: TrickType = 12;
pub const TRICK_STREET8: TrickType = 13;
pub const TRICK_STREET9: TrickType = 14;
pub const TRICK_STREET10: TrickType = 15;
pub const TRICK_STREET11: TrickType = 16;
pub const TRICK_STREET12: TrickType = 17;
pub const TRICK_STREET13: TrickType = 18;
pub const TRICK_STREET14: TrickType = 19;

pub const TRICK_FULLHOUSE: TrickType = 20;

pub const TRICK_DOG: TrickType = 21;
pub const TRICK_BOMB4: TrickType = 22;
pub const TRICK_BOMB5: TrickType = 23;
pub const TRICK_BOMB6: TrickType = 24;
pub const TRICK_BOMB7: TrickType = 25;
pub const TRICK_BOMB8: TrickType = 26;
pub const TRICK_BOMB9: TrickType = 27;
pub const TRICK_BOMB10: TrickType = 28;
pub const TRICK_BOMB11: TrickType = 29;
pub const TRICK_BOMB12: TrickType = 30;
pub const TRICK_BOMB13: TrickType = 31;


#[derive(PartialEq, Debug)]
pub enum HandType {
    Dog,
    Singleton(CardType, CardIndex),
    Pairs(CardType),
    Triplets(CardType),
    PairStreet(CardType, u8), //Value of lowest pair, length
    Street(CardType, u8), //Value of lowest card, length
    FullHouse(CardType, CardType), //Value of pair, Value of triplet
    Bomb4(CardType),
    BombStreet(CardType, u8), //Value of lowest card, length
}
impl HandType {
    pub fn is_bigger_than_same_handtype(&self, other: &HandType) -> bool {
        match (other, self) {
            (HandType::Singleton(c1, c1_idx), HandType::Singleton(c2, c2_idx)) => {
                c1 < c2 || (*c1 > 0 || *c1_idx == MAHJONG) && *c2_idx == PHOENIX || *c1 > 0 && *c2_idx == DRAGON || (*c1_idx == PHOENIX || *c1_idx == MAHJONG) && *c2_idx == DRAGON
            }
            (HandType::Pairs(c1), HandType::Pairs(c2)) => c1 < c2,
            (HandType::Triplets(c1), HandType::Triplets(c2)) => c1 < c2,
            (HandType::PairStreet(c1, s), HandType::PairStreet(c2, s2)) if s == s2 => c1 < c2,
            (HandType::Street(c1, s), HandType::Street(c2, s2)) if s == s2 => c1 < c2,
            (HandType::FullHouse(_, c1), HandType::FullHouse(_, c2)) => c1 < c2,
            (HandType::Bomb4(c1), HandType::Bomb4(c2)) => c1 < c2,
            (HandType::BombStreet(c1, s), HandType::BombStreet(c2, s2)) if s == s2 => c1 < c2,
            (_, _) => unreachable!()
        }
    }
    pub fn matches_trick_type(&self, trick_type: TrickType) -> bool {
        let self_trick_type = self.get_trick_type();
        self_trick_type < TRICK_BOMB4 && self_trick_type == trick_type || self_trick_type >= TRICK_BOMB4 && trick_type <= self_trick_type
    }
    pub fn get_trick_type(&self) -> TrickType {
        match self {
            HandType::Dog => TRICK_DOG,
            HandType::Singleton(_, _) => TRICK_SINGLETON,
            HandType::Pairs(_) => TRICK_PAIRS,
            HandType::Triplets(_) => TRICK_TRIPLETS,
            HandType::PairStreet(_, length) => TRICK_PAIRSTREET4 + (length - 4) / 2,
            HandType::Street(_, length) => TRICK_STREET5 + length - 5,
            HandType::FullHouse(_, _) => TRICK_FULLHOUSE,
            HandType::Bomb4(_) => TRICK_BOMB4, // A Bomb of 4 can always be played in tricks of lesser order
            HandType::BombStreet(_, length) => TRICK_BOMB5 + length - 5 // A Bomb of length x can always be played in tricks of lesser order
        }
    }
}
//------------------------------Hand implementation-----------------------------
impl TichuHand for Hand {
    fn get_lsb_card(&self) -> CardIndex {
        debug_assert!(*self != 0u64);
        self.trailing_zeros() as CardIndex
    }
    fn hand_type(&self) -> Option<HandType> {
        let cards = self.count_ones();
        if cards == 1 {
            let card = self.get_lsb_card();
            if card == DOG {
                return Some(HandType::Dog);
            } else {
                return Some(HandType::Singleton(get_card_type(card), card));
            }
        }
        if cards == 2 {
            //Only valid hands are pairs.
            let normals = self & MASK_NORMAL_CARDS;
            let is_pair = (self & hand!(PHOENIX) | ((normals >> BLUE | normals >> GREEN | normals >> RED) & normals)) != 0u64 && normals.count_ones() > 0;
            if is_pair {
                let pair_card = (self & MASK_NORMAL_CARDS).get_lsb_card();
                return Some(HandType::Pairs(get_card_type(pair_card)));
            }
            return None;
        }
        if cards == 3 {
            //Only valid hands are triplets. Just remove a card and check for a pair
            if let Some(HandType::Pairs(card)) = (self ^ hand!(self.get_lsb_card())).hand_type() {
                let removed_card = self.get_lsb_card();
                if removed_card == PHOENIX || card == get_card_type(removed_card) {
                    return Some(HandType::Triplets(card));
                }
                return None;
            }
            return None;
        }
        if cards % 2 == 0 {
            if let Some(card) = is_pair_street_fast(*self) {
                return Some(HandType::PairStreet(card, cards as u8));
            }
        }
        if cards == 4 {
            //Either four of kind bomb or a pair street
            if self.contains_four_of_kind_bomb() {
                return Some(HandType::Bomb4(get_card_type(self.get_lsb_card())));
            }
            //If its not a four of kind bomb, it has to be a pair street, which was checked above
            return None;
        }
        debug_assert!(cards >= 5);
        //Check for FullHouse
        if cards == 5 {
            let fh = self.is_fullhouse();
            if fh.is_some() {
                return fh;
            }
        }
        //Can be street
        if let Some(mut card_type) = is_street_fast(*self) {
            //Need to check if its a bomb street
            if (self & MASK_YELLOW).count_ones() == cards || (self & MASK_BLUE).count_ones() == cards || (self & MASK_GREEN).count_ones() == cards || (self & MASK_RED).count_ones() == cards {
                return Some(HandType::BombStreet(card_type, cards as u8));
            } else {
                if card_type + cards as u8 - 1 > ACE {
                    card_type -= 1; //Phoenix normally appended to top, but not if It can't :D
                }
                return Some(HandType::Street(card_type, cards as u8));
            }
        }
        None
    }
    fn is_fullhouse(&self) -> Option<HandType> {
        let has_phoenix: bool = self & hand!(PHOENIX) != 0u64;
        if has_phoenix {
            //Hand has to consist of two true pairs or a singleton and a triplet. Phoenix is used for larger pair.
            let normals = self & MASK_NORMAL_CARDS;
            let mut true_pairs: Hand = (normals >> BLUE | normals >> GREEN | normals >> RED) & normals;
            if true_pairs == 0u64 {
                return None;
            }
            let pair_one_card = get_card_type(true_pairs.get_lsb_card());
            true_pairs &= !MASK_FOUR_OF_KIND[pair_one_card as usize - 1];
            if true_pairs == 0u64 {
                //Either we have a triplet of pair_one_card or we don't have a fullhouse at all.
                if (MASK_FOUR_OF_KIND[pair_one_card as usize - 1] & self).count_ones() == 3 {
                    //Full House if Phoenix + other card is a pair
                    if let Some(HandType::Pairs(card)) = (self & !MASK_FOUR_OF_KIND[pair_one_card as usize - 1]).hand_type() {
                        return Some(HandType::FullHouse(card, pair_one_card));
                    }
                }
                return None;
            }
            let pair_two_card = get_card_type(true_pairs.get_lsb_card());
            return Some(HandType::FullHouse(pair_one_card.min(pair_two_card), pair_one_card.max(pair_two_card)));
        }
        //No phoenix, we have to have a pair and a triplet.
        let normals = self & MASK_NORMAL_CARDS;
        let mut true_pairs: Hand = (normals >> BLUE | normals >> GREEN | normals >> RED) & normals;
        //Expect exactly three matches
        if true_pairs.count_ones() != 3 {
            return None;
        }
        let first_card = get_card_type(true_pairs.pop_some_card());
        let second_card = get_card_type(true_pairs.pop_some_card());
        let third_card = get_card_type(true_pairs.pop_some_card());
        if first_card == second_card && second_card != third_card {
            return Some(HandType::FullHouse(third_card, first_card));
        }
        if first_card != second_card && second_card == third_card {
            return Some(HandType::FullHouse(first_card, third_card));
        }
        if first_card == third_card && second_card != third_card {
            return Some(HandType::FullHouse(second_card, third_card));
        }
        None
    }
    fn contains_straight_bomb(&self) -> bool {
        let straight_cards = self & MASK_NORMAL_CARDS;
        (straight_cards
            & (straight_cards << 1)
            & (straight_cards << 2)
            & (straight_cards << 3)
            & (straight_cards << 4))
            != 0u64
    }
    fn contains_four_of_kind_bomb(&self) -> bool {
        let normal_cards = self & MASK_NORMAL_CARDS;
        let shift_one_cards = normal_cards & (normal_cards >> BLUE);
        (shift_one_cards & (shift_one_cards >> GREEN)) != 0u64
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

    fn pop_some_card(&mut self) -> CardIndex {
        let card = self.get_lsb_card();
        *self &= *self - 1;
        card
    }

    fn get_card_points(&self) -> Score {
        (5 * (self & MASK_FIVES).count_ones() + 10 * (self & (MASK_TENS | MASK_KINGS)).count_ones()
            + 25 * (self & hand!(DRAGON)).count_ones()) as Score
            - 25 * (self & hand!(PHOENIX)).count_ones() as Score
    }

    fn get_high_card_amt(&self) -> u32 {
        (self & (MASK_KINGS | MASK_ACES | hand!(PHOENIX, DRAGON))).count_ones()
    }

    fn count_triplets(&self) -> u32 {
        let normals = self & MASK_NORMAL_CARDS;
        let true_pairs: Hand = (normals >> BLUE | normals >> GREEN | normals >> RED) & normals;
        let true_triplets = (true_pairs >> BLUE | true_pairs >> GREEN) & true_pairs;
        //bombs are counted twice!
        true_triplets.count_ones()
    }
}
