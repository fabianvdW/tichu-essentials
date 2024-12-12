use colored::Colorize;
use phf::phf_map;

// Generic trait each datastructure for a Tichu Hand should implement
pub trait TichuHand {
    fn get_some_card(&self) -> CardIndex;
    fn hand_type(&self) -> Option<HandType>;
    fn is_fullhouse(&self) -> Option<HandType>; //Only ever returns Fullhouse
    fn contains_straight_bomb(&self) -> bool;
    fn contains_four_of_kind_bomb(&self) -> bool;
    fn pretty_print(&self) -> String;
    fn debug_print(&self) -> String;
    fn pop_some_card(&mut self) -> CardIndex;
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

pub const DOG: CardType = 16;
pub const DRAGON: CardType = 32;
pub const MAHJONG: CardType = 48;

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
pub const MASK_ALL: Hand = MASK_YELLOW
    | (MASK_YELLOW << BLUE)
    | (MASK_YELLOW << GREEN)
    | (MASK_YELLOW << RED)
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
//TODO Ideas: PEXT bitboards for determining which street bomb in case of bombs

static CARD_TO_CHAR: phf::Map<u32, &'static str> = phf_map! {
    16u32 => "↺",
    0u32 => "🐦",
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
}; //If this is used only in non-speed related things, might be worth it to kick out the dependency and just use match.
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
pub const TRICK_BOMB4: TrickType = 21;
pub const TRICK_BOMB5: TrickType = 22;
pub const TRICK_BOMB6: TrickType = 23;
pub const TRICK_BOMB7: TrickType = 24;
pub const TRICK_BOMB8: TrickType = 25;
pub const TRICK_BOMB9: TrickType = 26;
pub const TRICK_BOMB10: TrickType = 27;
pub const TRICK_BOMB11: TrickType = 28;
pub const TRICK_BOMB12: TrickType = 29;
pub const TRICK_BOMB13: TrickType = 30;


#[derive(PartialEq, Debug)]
pub enum HandType {
    Singleton(CardType),
    Pairs(CardType),
    Triplets(CardType),
    PairStreet(CardType, u8), //Value of lowest pair, length
    Street(CardType, u8), //Value of lowest card, length
    FullHouse(CardType, CardType), //Value of pair, Value of triplet
    Bomb4(CardType),
    BombStreet(CardType, u8), //Value of lowest card, length
}
impl HandType {
    pub fn matches_trick_type(&self, trick_type: TrickType) -> bool {
        match self {
            HandType::Singleton(_) => trick_type == TRICK_SINGLETON,
            HandType::Pairs(_) => trick_type == TRICK_PAIRS,
            HandType::Triplets(_) => trick_type == TRICK_TRIPLETS,
            HandType::PairStreet(_, length) => trick_type == TRICK_PAIRSTREET4 + (length - 4) / 2,
            HandType::Street(_, length) => trick_type == TRICK_STREET5 + length - 5,
            HandType::FullHouse(_, _) => trick_type == TRICK_FULLHOUSE,
            HandType::Bomb4(_) => trick_type <= TRICK_BOMB4, // A Bomb of 4 can always be played in tricks of lesser order
            HandType::BombStreet(_, length) => trick_type <= TRICK_BOMB5 + length - 5 // A Bomb of length x can always be played in tricks of lesser order
        }
    }
}
//------------------------------Hand implementation-----------------------------
impl TichuHand for Hand {
    fn pop_some_card(&mut self) -> CardIndex {
        let card = self.get_some_card();
        *self &= *self - 1;
        card
    }
    fn get_some_card(&self) -> CardIndex {
        debug_assert!(*self != 0u64);
        self.trailing_zeros() as CardIndex
    }
    fn hand_type(&self) -> Option<HandType> {
        let cards = self.count_ones();
        if cards == 1 {
            return Some(HandType::Singleton(get_card_type(self.get_some_card())));
        }
        if cards == 2 {
            //Only valid hands are pairs.
            let normals = self & MASK_NORMAL_CARDS;
            let is_pair = (self & hand!(PHOENIX) | ((normals >> BLUE | normals >> GREEN | normals >> RED) & normals)) != 0u64 && normals.count_ones() > 0;
            if is_pair {
                let pair_card = (self & MASK_NORMAL_CARDS).get_some_card();
                return Some(HandType::Pairs(get_card_type(pair_card)));
            }
            return None;
        }
        if cards == 3 {
            //Only valid hands are triplets. Just remove a card and check for a pair
            if let Some(HandType::Pairs(card)) = (self ^ hand!(self.get_some_card())).hand_type() {
                let removed_card = self.get_some_card();
                if removed_card == PHOENIX || card == get_card_type(removed_card) {
                    return Some(HandType::Triplets(card));
                }
                return None;
            }
            return None;
        }
        if cards == 4 {
            //Either four of kind bomb or a pair street
            if self.contains_four_of_kind_bomb() {
                return Some(HandType::Bomb4(get_card_type(self.get_some_card())));
            }
            //If its not a four of kind bomb, it has to be a pair street.
            //Regardless of Phooenix or not, we have to have a true pair which we find by:
            let normals = self & MASK_NORMAL_CARDS;
            let true_pair: Hand = (normals >> BLUE | normals >> GREEN | normals >> RED) & normals;
            if true_pair == 0u64 {
                return None;
            }
            let first_pair_card = get_card_type(true_pair.get_some_card());
            if let Some(HandType::Pairs(card)) = (self & !MASK_FOUR_OF_KIND[first_pair_card as usize - 1]).hand_type() {
                return Some(HandType::PairStreet(first_pair_card.min(card), 4));
            }
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
        //Can be street or pair street
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
            let pair_one_card = get_card_type(true_pairs.get_some_card());
            true_pairs &= !MASK_FOUR_OF_KIND[pair_one_card as usize - 1];
            if true_pairs == 0u64 {
                //Either we have a triplet of pair_one_card or we don't have a fullhouse at all.
                if (MASK_FOUR_OF_KIND[pair_one_card as usize - 1] & self).count_ones() == 3{
                    //Full House if Phoenix + other card is a pair
                    if let Some(HandType::Pairs(card)) = (self & !MASK_FOUR_OF_KIND[pair_one_card as usize - 1]).hand_type(){
                        return Some(HandType::FullHouse(card, pair_one_card));
                    }
                }
                return None;
            }
            let pair_two_card = get_card_type(true_pairs.get_some_card());
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
}
