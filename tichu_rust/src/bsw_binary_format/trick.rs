use bitcode::{Decode, Encode};
use datasize::DataSize;
use crate::hand;
use crate::tichu_hand::*;
use crate::bsw_binary_format::binary_format_constants::*;
use crate::bsw_binary_format::round_log::{RoundLog, TaggedCardIndex, TaggedCardIndexT, SEPERATOR_NEW_MOVE, SEPERATOR_NEW_TRICK};
use crate::bsw_binary_format::trick::TrickIntegrityError::{DogTrickTooLong, EmptyPlayedHand, EmptyTrickLog, HandNoType, HandNotAvailable, HandTooSmall, HandWrongTrickType, ImplementationBug, TwiceInARowNoBomb};
use crate::street_detection_tricks::phoenix_used_as_street_extension;


#[derive(Debug)]
pub enum TrickIntegrityError {
    EmptyTrickLog,
    DogTrickTooLong,
    EmptyPlayedHand(usize), //Move Index into trick_log
    HandNotAvailable { hand: String, available_hand: String, player: PlayerIDInternal, move_idx: usize },
    HandNoType(String, PlayerIDInternal, usize),
    HandWrongTrickType(String, PlayerIDInternal, usize, HandType, TrickType),
    TwiceInARowNoBomb(usize, PlayerIDInternal),
    ImplementationBug(usize, TrickType, HandType),
    HandTooSmall { hand: String, hand_type: HandType, prev_hand: String, prev_hand_type: HandType, move_idx: usize },
}

#[derive(Encode, Decode, Default, DataSize)]
pub struct Trick {
    pub trick_type: TrickType,
    pub trick_log: Vec<(PlayerIDInternal, Hand)>,
}

impl Trick {
    pub fn serialize_into(&self, round_log: &mut RoundLog) {
        round_log.log.push(SEPERATOR_NEW_TRICK);
        round_log.log.push(self.trick_type);
        for (player, hand) in self.trick_log.iter() {
            let mut hand = *hand;
            while hand != 0u64 {
                round_log.log.push(TaggedCardIndex::construct(*player, hand.pop_some_card()));
            }
            round_log.log.push(SEPERATOR_NEW_MOVE);
        }
    }
    pub fn integrity_check(&self, player_hands: &mut [Hand; 4]) -> Result<(), TrickIntegrityError> {
        if self.trick_log.len() == 0 { return Err(EmptyTrickLog); };
        if self.trick_type == TRICK_DOG && self.trick_log.len() != 1 { return Err(DogTrickTooLong); };
        for (i, (_, hand)) in self.trick_log.iter().enumerate() {
            if *hand == 0 { return Err(EmptyPlayedHand(i)); };
        }
        //Check that hand type of every played hand matches the trick type. In case of bombs, trick type can upgrade!
        //Also check that every card that is played can be played by player.
        //checks that no player plays twice in a row unless a bomb is involved.
        let mut prev_player = None;
        let mut trick_type = self.trick_type;
        let mut prev_hand: Option<HandType> = None;
        for (move_idx, (player, hand)) in self.trick_log.iter().enumerate() {
            if hand & player_hands[*player as usize] != *hand {
                return Err(HandNotAvailable { hand: hand.pretty_print(), available_hand: player_hands[*player as usize].pretty_print(), player: *player, move_idx });
            }
            player_hands[*player as usize] ^= hand;
            if hand.hand_type().is_none() {
                return Err(HandNoType(hand.pretty_print(), *player, move_idx));
            }
            let hand_type = hand.hand_type().unwrap();
            if !hand_type.matches_trick_type(trick_type) {
                return Err(HandWrongTrickType(hand.pretty_print(), *player, move_idx, hand_type, trick_type));
            }
            let new_trick_type = hand_type.get_trick_type();
            if prev_player.is_some() && prev_player.unwrap() == player && new_trick_type < TRICK_BOMB4 {
                return Err(TwiceInARowNoBomb(move_idx, *player));
            }
            if trick_type == new_trick_type {
                //Check that the new hand is actually playable
                if let Some(mut prev_hand_type) = prev_hand {
                    //If the trick type is street, and the previous hand contains a phoenix that extends the street,
                    //we allow prev_hand_type to be one smaller (since we have no indication of how phoenix is played).
                    if trick_type >= TRICK_STREET5 && trick_type <= TRICK_STREET14 && phoenix_used_as_street_extension(self.get_hand(move_idx - 1)) {
                        //Lower prev_hand_type if possible
                        if let HandType::Street(lowest_card, length) = prev_hand_type {
                            if lowest_card > SPECIAL_CARD && self.get_hand(move_idx - 1) & MASK_ACES == 0 { //Phoenix already used as low card if the street can't be extended further than ace.
                                prev_hand_type = HandType::Street(lowest_card - 1, length);
                            }
                        } else {
                            return Err(ImplementationBug(move_idx, trick_type, prev_hand_type));
                        }
                    }
                    //If the trick type is fullhouse, and the full house consists of two pairs + phoenix,
                    // we allow the phoenix to be used as the lower card instead of the default upper card (since we have no indication of how phoenix is played).
                    if trick_type == TRICK_FULLHOUSE && Trick::hand_is_two_pairs_plus_phoenix(self.get_hand(move_idx - 1)) {
                        if let HandType::FullHouse(lower_card, higher_card) = prev_hand_type {
                            prev_hand_type = HandType::FullHouse(higher_card, lower_card);
                        } else {
                            return Err(ImplementationBug(move_idx, trick_type, prev_hand_type));
                        }
                    }
                    if !hand_type.is_bigger_than_same_handtype(&prev_hand_type) {
                        return Err(HandTooSmall { hand: hand.pretty_print(), hand_type, prev_hand: self.get_hand(move_idx - 1).pretty_print(), prev_hand_type, move_idx });
                    }
                }
            }
            trick_type = new_trick_type;
            prev_hand = Some(hand_type);
            prev_player = Some(player);
        }
        Ok(())
    }

    pub fn get_player(&self, index: usize) -> PlayerIDInternal {
        self.trick_log[index].0
    }
    pub fn get_hand(&self, index: usize) -> Hand {
        self.trick_log[index].1
    }

    pub fn played_cards(&self) -> Hand {
        self.trick_log.iter().fold(0u64, |acc, inc| acc | inc.1)
    }

    pub fn has_to_gift_trick(&self) -> bool {
        self.get_hand(self.trick_log.len() - 1) == hand!(DRAGON)
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
}