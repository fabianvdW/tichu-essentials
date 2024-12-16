use bitcode::{Decode, Encode};
use crate::hand;
use crate::tichu_hand::*;
use crate::bsw_binary_format::binary_format_constants::*;
use crate::street_detection_tricks::phoenix_used_as_street_extension;

pub type TaggedCardIndex = u8; //Lower 6 bits are CardIndex, upper 2 CardIndex are Tag

pub trait TaggeCardIndexT {
    fn construct(player: PlayerIDInternal, card_index: CardIndex) -> Self;
    fn get_player(&self) -> PlayerIDInternal;
    fn get_card(&self) -> CardIndex;
}
impl TaggeCardIndexT for TaggedCardIndex {
    fn construct(player: PlayerIDInternal, card_index: CardIndex) -> Self {
        card_index | (player as u8) << 6
    }

    fn get_player(&self) -> PlayerIDInternal {
        ((self >> 6) & 0b11u8) as PlayerIDInternal
    }

    fn get_card(&self) -> CardIndex {
        self & 0x3F
    }
}

#[derive(Encode, Decode, Default)]
pub struct Trick {
    pub trick_type: TrickType,
    pub trick_log: Vec<Vec<TaggedCardIndex>>,
}
pub struct TrickIterator<'a> {
    trick: &'a Trick,
    current_index: usize,
}
impl<'a> Iterator for TrickIterator<'a> {
    type Item = (PlayerIDInternal, Hand);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.trick.trick_log.len() {
            let player = self.trick.get_player(self.current_index);
            let hand = self.trick.get_hand(self.current_index);
            self.current_index += 1;
            Some((player, hand))
        } else {
            None
        }
    }
}
impl Trick {
    pub fn iter(&self) -> TrickIterator {
        TrickIterator { trick: self, current_index: 0 }
    }
    pub fn get_player(&self, index: usize) -> PlayerIDInternal {
        self.trick_log[index][0].get_player()
    }
    pub fn get_hand(&self, index: usize) -> Hand {
        self.trick_log[index].iter().fold(0u64, |acc, inc| acc | hand!(inc.get_card()))
    }
    pub fn played_cards(&self) -> Hand {
        let mut res = 0u64;
        for i in 0..self.trick_log.len() {
            res |= self.get_hand(i);
        }
        res
    }

    pub fn has_to_gift_trick(&self) -> bool {
        self.played_cards() & hand!(DRAGON) != 0
            && self.get_hand(self.trick_log.len() - 1).hand_type().unwrap().get_trick_type() < TRICK_BOMB4
    }

    pub fn get_starting_player(&self) -> PlayerIDInternal {
        self.get_player(0)
    }

    pub fn get_trick_winner(&self) -> PlayerIDInternal {
        if self.trick_type == TRICK_DOG {
            TEAMMATE_PLAYERS[self.get_starting_player() as usize]
        } else {
            self.get_player(self.trick_log.len() - 1)
        }
    }
    fn hand_is_two_pairs_plus_phoenix(hand: Hand) -> bool { //Only call this on full houses!
        assert!(hand.is_fullhouse().is_some());
        if hand & hand!(PHOENIX) == 0 {
            return false;
        }
        let normals = hand & MASK_NORMAL_CARDS;
        let mut true_pairs: Hand = (normals >> BLUE | normals >> GREEN | normals >> RED) & normals;
        let pair_one_card = get_card_type(true_pairs.get_lsb_card());
        true_pairs &= !MASK_FOUR_OF_KIND[pair_one_card as usize - 1];
        true_pairs != 0
    }
    pub fn integrity_check(&self, player_hands: &mut [Hand; 4], game: u32, round_num: usize, trick_num: usize) {
        assert!(self.trick_log.len() > 0);
        assert!(self.trick_type != TRICK_DOG || self.trick_log.len() == 1);
        for card_vec in self.trick_log.iter() {
            assert!(card_vec.len() > 0);
            assert!(card_vec.iter().all(|x| x.get_player() == card_vec[0].get_player()));
        }
        //Check that hand type of every played hand matches the trick type. In case of bombs, trick type can upgrade!
        //Also check that every card that is played can be played by player.
        //checks that no player plays twice in a row unless a bomb is involved.
        let mut prev_player = None;
        let mut trick_type = self.trick_type;
        let mut prev_hand: Option<HandType> = None;
        for (i, (player, hand)) in self.iter().enumerate() {
            assert_eq!(hand & player_hands[player as usize], hand);
            player_hands[player as usize] ^= hand;
            if hand.hand_type().is_none() {
                println!("{} {} {}", game, round_num, trick_num);
                println!("Current hand {} for player {}: {}", i, player, hand.pretty_print());
                println!("No HandType: ");
                println!("Previous HandType: {:?} {}", prev_hand, trick_type);
            }
            let hand_type = hand.hand_type().unwrap();
            if !hand_type.matches_trick_type(trick_type) {
                println!("{} {} {}", game, round_num, trick_num);
                println!("Current hand {} for player {}: {}", i, player, hand.pretty_print());
                println!("HandType: {:?}, trick_type: {}", hand_type, trick_type);
                println!("Previous HandType: {:?} {}", prev_hand, trick_type);
            }
            assert!(hand_type.matches_trick_type(trick_type));
            let new_trick_type = hand_type.get_trick_type();
            assert!(prev_player.is_none() || prev_player.unwrap() != player || new_trick_type >= TRICK_BOMB4);
            assert!(i > 0 || new_trick_type == trick_type);
            if trick_type == new_trick_type {
                //Check that the new hand is actually playable
                if let Some(mut prev_hand_type) = prev_hand {
                    //If the trick type is street, and the previous hand contains a phoenix that extends the street,
                    //we allow prev_hand_type to be one smaller (since we have no indication of how phoenix is played).
                    if trick_type >= TRICK_STREET5 && trick_type <= TRICK_STREET14 && phoenix_used_as_street_extension(self.get_hand(i - 1)) {
                        //Lower prev_hand_type if possible
                        if let HandType::Street(lowest_card, length) = prev_hand_type {
                            if lowest_card > SPECIAL_CARD && self.get_hand(i - 1) & MASK_ACES == 0 { //Phoenix already used as low card if the street can't be extended further than ace.
                                prev_hand_type = HandType::Street(lowest_card - 1, length);
                            }
                        } else {
                            assert!(false);
                        }
                    }
                    //If the trick type is fullhouse, and the full house consists of two pairs + phoenix,
                    // we allow the phoenix to be used as the lower card instead of the default upper card (since we have no indication of how phoenix is played).
                    if trick_type == TRICK_FULLHOUSE && Trick::hand_is_two_pairs_plus_phoenix(self.get_hand(i - 1)) {
                        if let HandType::FullHouse(lower_card, higher_card) = prev_hand_type {
                            prev_hand_type = HandType::FullHouse(higher_card, lower_card);
                        } else {
                            assert!(false);
                        }
                    }
                    if !hand_type.is_bigger_than_same_handtype(&prev_hand_type) {
                        println!("{} {} {}", game, round_num, trick_num);
                        println!("Current hand {} for player {}: {}", i, player, hand.pretty_print());
                        println!("HandType: {:?}", hand_type);
                        println!("Previous HandType: {:?}", prev_hand_type);
                    }
                    assert!(hand_type.is_bigger_than_same_handtype(&prev_hand_type));
                }
            }
            trick_type = new_trick_type;
            prev_hand = Some(hand_type);
            prev_player = Some(player);
        }
    }
}