use crate::hand;
use crate::tichu_hand::*;
use bitcode::{Decode, Encode};
use crate::street_detection_tricks::phoenix_used_as_street_extension;

pub const LEFT_OUT_EXCHANGE_MASK: u64 = 0x3Fu64;
pub const PARTNER_OUT_EXCHANGE_MASK: u64 = 0x3Fu64 << 6;
pub const RIGHT_OUT_EXCHANGE_MASK: u64 = 0x3Fu64 << 12;
pub const LEFT_IN_EXCHANGE_MASK: u64 = 0x3Fu64 << 18;
pub const PARTNER_IN_EXCHANGE_MASK: u64 = 0x3Fu64 << 24;
pub const RIGHT_IN_EXCHANGE_MASK: u64 = 0x3Fu64 << 30;

pub type TichuCall = u8;
pub const CALL_NONE: TichuCall = 0u8;
pub const CALL_TICHU: TichuCall = 1u8;
pub const CALL_GRAND_TICHU: TichuCall = 2u8;

pub const CALL_PLAYER_0_MASK: u64 = 0b11u64 << 36;
pub const CALL_PLAYER_1_MASK: u64 = 0b11u64 << 38;
pub const CALL_PLAYER_2_MASK: u64 = 0b11u64 << 40;
pub const CALL_PLAYER_3_MASK: u64 = 0b11u64 << 42;

pub type PlayerIDGlobal = u32;
pub type PlayerIDInternal = u8;
pub const PLAYER_0: PlayerIDInternal = 0u8; //Team 1
pub const PLAYER_1: PlayerIDInternal = 1u8; //Team 2
pub const PLAYER_2: PlayerIDInternal = 2u8; //Team 1
pub const PLAYER_3: PlayerIDInternal = 3u8; //Team 2

pub const TEAMMATE_PLAYERS: [PlayerIDInternal; 4] = [PLAYER_2, PLAYER_3, PLAYER_0, PLAYER_1];

pub const PLAYER_ID_MASK: u64 = 0b11u64 << 44;

pub type Rank = u8;
pub const RANK_1: Rank = 0u8;
pub const RANK_2: Rank = 1u8;
pub const RANK_3: Rank = 2u8;
pub const RANK_4: Rank = 3u8;
pub const RANK_PLAYER_0_MASK: u64 = 0b11u64 << 46;
pub const RANK_PLAYER_1_MASK: u64 = 0b11u64 << 48;
pub const RANK_PLAYER_2_MASK: u64 = 0b11u64 << 50;
pub const RANK_PLAYER_3_MASK: u64 = 0b11u64 << 52;

pub type Score = i16;
pub const CARD_SCORE_MASK: u64 = 0xFFu64 << 54;
///8 Bits indicating the value of card point difference collected in the round for team 1 (0-150)
#[derive(Encode, Decode, Default)]
pub struct PlayerRoundHand {
    pub first_8: Hand,
    pub first_14: Hand,
    pub extras: u64,
}
#[derive(Encode, Decode, Default)]
pub struct Round {
    pub player_rounds: [PlayerRoundHand; 4],
}
impl Round {
    pub fn integrity_check(&self) {
        for i in 0..4 {
            self.player_rounds[i].integrity_check();
        }
        let (p0, p1, p2, p3) = (
            self.player_rounds.get(0).unwrap(),
            self.player_rounds.get(1).unwrap(),
            self.player_rounds.get(2).unwrap(),
            self.player_rounds.get(3).unwrap(),
        );
        //Check all cards are distributed. Child checks ensures its 14 each.
        assert_eq!(
            p0.first_14 | p1.first_14 | p2.first_14 | p3.first_14,
            MASK_ALL
        );
        //Check exchange cards
        assert_eq!(p0.right_out_exchange_card(), p1.left_in_exchange_card());
        assert_eq!(p1.right_out_exchange_card(), p2.left_in_exchange_card());
        assert_eq!(p2.right_out_exchange_card(), p3.left_in_exchange_card());
        assert_eq!(p3.right_out_exchange_card(), p0.left_in_exchange_card());

        assert_eq!(p0.left_out_exchange_card(), p3.right_in_exchange_card());
        assert_eq!(p3.left_out_exchange_card(), p2.right_in_exchange_card());
        assert_eq!(p2.left_out_exchange_card(), p1.right_in_exchange_card());
        assert_eq!(p1.left_out_exchange_card(), p0.right_in_exchange_card());

        assert_eq!(p0.partner_out_exchange_card(), p2.partner_in_exchange_card());
        assert_eq!(p2.partner_out_exchange_card(), p0.partner_in_exchange_card());
        assert_eq!(p1.partner_out_exchange_card(), p3.partner_in_exchange_card());
        assert_eq!(p3.partner_out_exchange_card(), p1.partner_in_exchange_card());

        //Check that all other fields agree on the same values
        assert_eq!((p0.extras >> 36) & 0xFF, (p1.extras >> 36) & 0xFF);
        assert_eq!((p1.extras >> 36) & 0xFF, (p2.extras >> 36) & 0xFF);
        assert_eq!((p2.extras >> 36) & 0xFF, (p3.extras >> 36) & 0xFF);

        assert_eq!(p0.extras >> 46, p1.extras >> 46);
        assert_eq!(p1.extras >> 46, p2.extras >> 46);
        assert_eq!(p2.extras >> 46, p3.extras >> 46);

        assert_eq!(p0.player_id(), PLAYER_0);
        assert_eq!(p1.player_id(), PLAYER_1);
        assert_eq!(p2.player_id(), PLAYER_2);
        assert_eq!(p3.player_id(), PLAYER_3);

        assert_eq!(p0.extras >> 54, p1.extras >> 54);
        assert_eq!(p1.extras >> 54, p2.extras >> 54);
        assert_eq!(p2.extras >> 54, p3.extras >> 54);
    }
}
pub type TaggedHand = u64; //Lower 6 bits are CardIndex, upper 2 CardIndex are Tag

pub trait TaggedHandT {
    fn construct(player: PlayerIDInternal, hand: Hand) -> Self;
    fn get_player(&self) -> PlayerIDInternal;
    fn get_hand(&self) -> Hand;
}
impl TaggedHandT for TaggedHand {
    fn construct(player: PlayerIDInternal, hand: Hand) -> Self {
        hand | (player as u64) << 14
    }

    fn get_player(&self) -> PlayerIDInternal {
        ((self >> 14) & 0b11u64) as u8
    }

    fn get_hand(&self) -> Hand {
        self & MASK_ALL
    }
}
#[derive(Encode, Decode, Default)]
pub struct RoundLog {
    pub mahjong_wish: Option<CardIndex>,
    pub dragon_player_gift: Option<PlayerIDInternal>,
    pub log: Vec<Trick>,
}
impl RoundLog {
    pub fn try_fix_dragon_gifting(&mut self, round: &Round) -> Option<bool> {
        //The BSW dataset has a bug where the dragon is sometimes gifted to the player that plays it
        //or is gifted to an enemy that is no longer playing.
        //(no bombs involved). However, we can resolve the dragon gifting if there is only
        //one enemy player still playing.
        let mut player_hands = [round.player_rounds[0].final_14(), round.player_rounds[1].final_14(), round.player_rounds[2].final_14(), round.player_rounds[3].final_14()];
        for trick in self.log.iter() {
            for t_hand in trick.trick_log.iter() {
                let player = t_hand.get_player() as usize;
                player_hands[player] ^= t_hand.get_hand();
            }
            if trick.has_to_gift_trick() {
                let gift_player = self.dragon_player_gift.unwrap();
                let winner_player = trick.get_trick_winner();
                //Dragon gifted to own team or to a player that has finished already.
                if winner_player == gift_player || winner_player == (gift_player + 2) % 4 || player_hands[gift_player as usize] == 0 {
                    if player_hands[(winner_player as usize + 1) % 4] == 0 {
                        //We can gift the dragon to (winner_player + 3) % 4
                        self.dragon_player_gift = Some((winner_player + 3) % 4);
                        return Some(true);
                    } else if player_hands[(winner_player as usize + 3) % 4] == 0 {
                        //We can gift the dragon to (winner_player + 1) % 4
                        self.dragon_player_gift = Some((winner_player + 1) % 4);
                        return Some(true);
                    } else {
                        return Some(false);
                    }
                }

            }
        }
        None
    }
    pub fn play_round(&self, round: &Round, game: usize, round_num: usize) -> ([Rank; 4], [Score; 4], bool) { //Ranks, CardPoints, double_win
        let mut player_hands = [round.player_rounds[0].final_14(), round.player_rounds[1].final_14(), round.player_rounds[2].final_14(), round.player_rounds[3].final_14()];
        let mut player_scores = [0; 4];
        let mut player_ranks = [RANK_4; 4];
        let mut next_rank = RANK_1;
        for trick in self.log.iter() {
            let card_points = trick.played_cards().get_card_points();
            if trick.has_to_gift_trick() {
                player_scores[self.dragon_player_gift.unwrap() as usize] += card_points;
            } else {
                player_scores[trick.get_trick_winner() as usize] += card_points;
            }
            for t_hand in trick.trick_log.iter() {
                let player = t_hand.get_player() as usize;
                player_hands[player] ^= t_hand.get_hand();
                if player_hands[player] == 0 {
                    player_ranks[player] = next_rank;
                    next_rank += 1;
                }
            }
        }
        let is_double_win = player_ranks[PLAYER_0 as usize] + player_ranks[PLAYER_2 as usize] <= RANK_1 + RANK_2
            || player_ranks[PLAYER_1 as usize] + player_ranks[PLAYER_3 as usize] <= RANK_1 + RANK_2;
        if is_double_win {
            return (player_ranks, [0; 4], true);
        }
        //Else, gift card points of player 4 to first
        let first_player = player_ranks.iter().position(|x| *x == RANK_1).unwrap();
        let third_player = player_ranks.iter().position(|x| *x == RANK_3).unwrap();
        let fourth_player = player_ranks.iter().position(|x| *x == RANK_4).unwrap();
        player_scores[first_player] += player_scores[fourth_player];
        player_scores[fourth_player] = 0;
        player_scores[third_player] += player_hands[fourth_player].get_card_points();
        if player_scores.iter().sum::<Score>() != 100 {
            println!("Score mismatch in {} {}", game, round_num);
            println!("Player Scores: {:?}", player_scores);
            println!("Player Ranks: {:?}", player_ranks);
        }
        assert_eq!(player_scores.iter().sum::<Score>(), 100);
        (player_ranks, player_scores, false)
    }
    pub fn integrity_check(&self, round: &Round, game_idx: usize, round_num: usize) {
        let mut player_hands = [round.player_rounds[0].final_14(), round.player_rounds[1].final_14(), round.player_rounds[2].final_14(), round.player_rounds[3].final_14()];
        if game_idx == 1001683 && round_num == 10 {
            println!("{}", player_hands[0].pretty_print());
            println!("{}", player_hands[1].pretty_print());
            println!("{}", player_hands[2].pretty_print());
            println!("{}", player_hands[3].pretty_print());
            for (trick_num, trick) in self.log.iter().enumerate() {
                println!("Starting Trick {} with type {}", trick_num, trick.trick_type);
                for thand in &trick.trick_log {
                    println!("Player {} plays {}", thand.get_player(), thand.get_hand().pretty_print());
                }
                println!("Ending Trick {} with trick winner {}", trick_num, trick.get_trick_winner());
            }
        }
        let mut prev_player: Option<PlayerIDInternal> = None;
        for (trick_num, trick) in self.log.iter().enumerate() {
            if let Some(prev) = prev_player {
                if trick.get_starting_player() != prev {
                    println!("{} {} {}", game_idx, round_num, trick_num);
                    println!("{:?}", prev);
                    println!("{}", trick.get_starting_player());
                }
                assert_eq!(trick.get_starting_player(), prev);
            }
            trick.integrity_check(&mut player_hands, game_idx, round_num, trick_num);
            let mut trick_winner = trick.get_trick_winner();
            while player_hands[trick_winner as usize] == 0u64 {
                trick_winner = (trick_winner + 1) % 4;
            }
            prev_player = Some(trick_winner);
        }
    }
}
#[derive(Encode, Decode, Default)]
pub struct Trick {
    pub trick_type: TrickType,
    pub trick_log: Vec<TaggedHand>, //TODO: Maybe TaggedHand -> Vec<TaggedCardIndex> ?! saves some space.
}
impl Trick {
    pub fn played_cards(&self) -> Hand {
        self.trick_log.iter().fold(0u64, |acc, inc| acc | inc.get_hand())
    }

    pub fn has_to_gift_trick(&self) -> bool {
        self.played_cards() & hand!(DRAGON) != 0
            && self.trick_log[self.trick_log.len() - 1].get_hand().hand_type().unwrap().get_trick_type() < TRICK_BOMB4
    }

    pub fn get_starting_player(&self) -> PlayerIDInternal {
        self.trick_log[0].get_player()
    }

    pub fn get_trick_winner(&self) -> PlayerIDInternal {
        if self.trick_type == TRICK_DOG {
            TEAMMATE_PLAYERS[self.get_starting_player() as usize]
        } else {
            self.trick_log[self.trick_log.len() - 1].get_player()
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
    pub fn integrity_check(&self, player_hands: &mut [Hand; 4], game: usize, round_num: usize, trick_num: usize) {
        assert!(self.trick_log.len() > 0);
        assert!(self.trick_type != TRICK_DOG || self.trick_log.len() == 1);
        //Check that hand type of every played hand matches the trick type. In case of bombs, trick type can upgrade!
        //Also check that every card that is played can be played by player.
        //checks that no player plays twice in a row unless a bomb is involved.
        let mut prev_player = None;
        let mut trick_type = self.trick_type;
        let mut prev_hand: Option<HandType> = None;
        for (i, t_hand) in self.trick_log.iter().enumerate() {
            let (hand, player) = (t_hand.get_hand(), t_hand.get_player() as usize);
            assert_eq!(hand & player_hands[player], hand);
            player_hands[player] ^= hand;
            if hand.hand_type().is_none() {
                println!("{} {} {}", game, round_num, trick_num);
                println!("Current hand {} for player {}: {}", i, t_hand.get_player(), t_hand.get_hand().pretty_print());
                println!("No HandType: ");
                println!("Previous HandType: {:?} {}", prev_hand, trick_type);
            }
            let hand_type = hand.hand_type().unwrap();
            if !hand_type.matches_trick_type(trick_type) {
                println!("{} {} {}", game, round_num, trick_num);
                println!("Current hand {} for player {}: {}", i, t_hand.get_player(), t_hand.get_hand().pretty_print());
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
                    if trick_type >= TRICK_STREET5 && trick_type <= TRICK_STREET14 && phoenix_used_as_street_extension(self.trick_log[i - 1].get_hand()) {
                        //Lower prev_hand_type if possible
                        if let HandType::Street(lowest_card, length) = prev_hand_type {
                            if lowest_card > SPECIAL_CARD && self.trick_log[i-1].get_hand() & MASK_ACES == 0 { //Phoenix already used as low card if the street can't be extended further than ace.
                                prev_hand_type = HandType::Street(lowest_card - 1, length);
                            }
                        } else {
                            assert!(false);
                        }
                    }
                    //If the trick type is fullhouse, and the full house consists of two pairs + phoenix,
                    // we allow the phoenix to be used as the lower card instead of the default upper card (since we have no indication of how phoenix is played).
                    if trick_type == TRICK_FULLHOUSE && Trick::hand_is_two_pairs_plus_phoenix(self.trick_log[i - 1].get_hand()) {
                        if let HandType::FullHouse(lower_card, higher_card) = prev_hand_type {
                            prev_hand_type = HandType::FullHouse(higher_card, lower_card);
                        } else {
                            assert!(false);
                        }
                    }
                    if !hand_type.is_bigger_than_same_handtype(&prev_hand_type) {
                        println!("{} {} {}", game, round_num, trick_num);
                        println!("Current hand {} for player {}: {}", i, t_hand.get_player(), t_hand.get_hand().pretty_print());
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
pub enum Team {
    Team1,
    Team2,
}
impl Team {
    pub fn get_players(&self) -> (PlayerIDInternal, PlayerIDInternal) {
        match self {
            Team::Team1 => (PLAYER_0, PLAYER_2),
            Team::Team2 => (PLAYER_1, PLAYER_3),
        }
    }
}
#[derive(Encode, Decode)]
pub struct Game {
    pub rounds: Vec<Round>,
    pub round_logs: Vec<RoundLog>,
    pub player_ids: [PlayerIDGlobal; 4],
    pub unfinished: bool,
}
impl Game {
    pub fn get_winner(&self) -> Team {
        let mut score_team_0: Score = 0;
        let mut score_team_1: Score = 0;
        for round in &self.rounds {
            let round_scores = round.player_rounds[PLAYER_0 as usize].round_score();
            score_team_0 += round_scores.0;
            score_team_1 += round_scores.1;
        }
        if score_team_0 > score_team_1 {
            Team::Team1
        } else {
            Team::Team2
        }
    }
}

impl PlayerRoundHand {
    pub fn integrity_check(&self) {
        assert_eq!(self.first_8.count_ones(), 8);
        assert_eq!(self.first_8 & MASK_ALL, self.first_8);
        assert_eq!(self.first_14.count_ones(), 14);
        assert_eq!(self.first_14 & MASK_ALL, self.first_14);
        assert_eq!(self.final_14().count_ones(), 14);
        assert_eq!(self.final_14() & MASK_ALL, self.final_14());
        assert_eq!((hand!(self.left_out_exchange_card(),self.partner_out_exchange_card(),self.right_out_exchange_card()) & self.first_14).count_ones(), 3);
        assert_eq!((hand!(self.left_in_exchange_card(),self.partner_in_exchange_card(),self.right_in_exchange_card()) & self.first_14).count_ones(), 0);
    }
    pub fn left_out_exchange_card(&self) -> CardIndex {
        (self.extras & LEFT_OUT_EXCHANGE_MASK) as CardIndex
    }
    pub fn partner_out_exchange_card(&self) -> CardIndex {
        ((self.extras & PARTNER_OUT_EXCHANGE_MASK) >> 6) as CardIndex
    }
    pub fn right_out_exchange_card(&self) -> CardIndex {
        ((self.extras & RIGHT_OUT_EXCHANGE_MASK) >> 12) as CardIndex
    }

    pub fn left_in_exchange_card(&self) -> CardIndex {
        ((self.extras & LEFT_IN_EXCHANGE_MASK) >> 18) as CardIndex
    }
    pub fn partner_in_exchange_card(&self) -> CardIndex {
        ((self.extras & PARTNER_IN_EXCHANGE_MASK) >> 24) as CardIndex
    }
    pub fn right_in_exchange_card(&self) -> CardIndex {
        ((self.extras & RIGHT_IN_EXCHANGE_MASK) >> 30) as CardIndex
    }

    pub fn final_14(&self) -> Hand {
        //println!("LEX:{}", self.left_out_exchange_card());
        self.first_14
            ^ hand!(
                self.left_out_exchange_card(),
                self.partner_out_exchange_card(),
                self.right_out_exchange_card(),
                self.left_in_exchange_card(),
                self.partner_in_exchange_card(),
                self.right_in_exchange_card()
            )
    }

    pub fn player_0_call(&self) -> TichuCall {
        ((self.extras & CALL_PLAYER_0_MASK) >> 36) as TichuCall
    }
    pub fn player_1_call(&self) -> TichuCall {
        ((self.extras & CALL_PLAYER_1_MASK) >> 38) as TichuCall
    }
    pub fn player_2_call(&self) -> TichuCall {
        ((self.extras & CALL_PLAYER_2_MASK) >> 40) as TichuCall
    }
    pub fn player_3_call(&self) -> TichuCall {
        ((self.extras & CALL_PLAYER_3_MASK) >> 42) as TichuCall
    }
    pub fn player_id(&self) -> PlayerIDInternal {
        ((self.extras & PLAYER_ID_MASK) >> 44) as PlayerIDInternal
    }
    pub fn player_0_rank(&self) -> Rank {
        ((self.extras & RANK_PLAYER_0_MASK) >> 46) as Rank
    }
    pub fn player_1_rank(&self) -> Rank {
        ((self.extras & RANK_PLAYER_1_MASK) >> 48) as Rank
    }
    pub fn player_2_rank(&self) -> Rank {
        ((self.extras & RANK_PLAYER_2_MASK) >> 50) as Rank
    }
    pub fn player_3_rank(&self) -> Rank {
        ((self.extras & RANK_PLAYER_3_MASK) >> 52) as Rank
    }

    pub fn is_double_win_team_1(&self) -> bool {
        self.player_0_rank() + self.player_2_rank() <= RANK_1 + RANK_2
    }
    pub fn is_double_win_team_2(&self) -> bool {
        self.player_1_rank() + self.player_3_rank() <= RANK_1 + RANK_2
    }
    pub fn round_score(&self) -> (Score, Score) {
        //Reported absolute to Team1;
        let mut score_team_1: Score = 0;
        let mut score_team_2: Score = 0;
        score_team_1 += self.player_0_call() as Score * 100 * {
            if self.player_0_rank() == RANK_1 {
                1
            } else {
                -1
            }
        };
        score_team_1 += self.player_2_call() as Score * 100 * {
            if self.player_2_rank() == RANK_1 {
                1
            } else {
                -1
            }
        };
        score_team_2 += self.player_1_call() as Score * 100 * {
            if self.player_1_rank() == RANK_1 {
                1
            } else {
                -1
            }
        };
        score_team_2 += self.player_3_call() as Score * 100 * {
            if self.player_3_rank() == RANK_1 {
                1
            } else {
                -1
            }
        };
        //Double Win or Card Points
        if self.is_double_win_team_1() {
            //Double Win for Team 1
            score_team_1 += 200;
        } else if self.is_double_win_team_2() {
            //Double Win for Team 2
            score_team_2 += 200;
        } else {
            let card_score: Score = ((self.extras & CARD_SCORE_MASK) >> 54) as Score;
            score_team_1 += card_score - 25;
            score_team_2 += 125 - card_score;
        }
        (score_team_1, score_team_2)
    }
    pub fn round_score_relative_gain(&self) -> Score {
        let round_score = self.round_score();
        if self.player_id() == PLAYER_0 || self.player_id() == PLAYER_1 {
            round_score.0 - round_score.1
        } else {
            round_score.1 - round_score.0
        }
    }
}
