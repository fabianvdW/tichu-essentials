use crate::analysis::{format_slice_abs_relative, format_slice_abs_relative2};
use crate::bsw_database::DataBase;
use crate::hand;
use crate::street_detection_tricks::{prepare_hand, PACKING_BITS, PACKING_BITS_MASK, STREET_DATA_ARRAY};
use crate::tichu_hand::{Hand, HandType, TichuHand, ACE, EIGHT, FIVE, FOUR, JACK, KING, MASK_ACES, NINE, PHOENIX, QUEEN, SEVEN, SIX, TEN, TRICK_STREET5, TRICK_STREET6, TRICK_STREET7, TRICK_STREET8, TRICK_STREET9};

pub fn evaluate_streets_in_play(db: &DataBase) {
    let mut s5_played_by_other_3 = [0; 4];
    let mut s6_played_by_other_3 = [0; 4];
    let mut s7_played_by_other_3 = [0; 4];
    let mut s8_played_by_other_3 = [0; 4];
    let mut s9_played_by_other_3 = [0; 4];
    let mut rounds = 0;
    for game in db.games.iter() {
        for (_, round_log) in game.rounds.iter() {
            let mut s5_played = [false; 4];
            let mut s6_played = [false; 4];
            let mut s7_played = [false; 4];
            let mut s8_played = [false; 4];
            let mut s9_played = [false; 4];
            let mut iter = round_log.iter();
            while let Some(trick) = iter.next_trick() {
                if trick.trick_type == TRICK_STREET5 {
                    s5_played[trick.get_starting_player() as usize] = true;
                } else if trick.trick_type == TRICK_STREET6 {
                    s6_played[trick.get_starting_player() as usize] = true;
                } else if trick.trick_type == TRICK_STREET7 {
                    s7_played[trick.get_starting_player() as usize] = true;
                } else if trick.trick_type == TRICK_STREET8 {
                    s8_played[trick.get_starting_player() as usize] = true;
                } else if trick.trick_type == TRICK_STREET9 {
                    s9_played[trick.get_starting_player() as usize] = true;
                }
            }
            rounds += 1;
            for player in 0..4 {
                s5_played_by_other_3[player] += (s5_played[(player + 1) % 4] || s5_played[(player + 2) % 4] || s5_played[(player + 3) % 4]) as usize;
                s6_played_by_other_3[player] += (s6_played[(player + 1) % 4] || s6_played[(player + 2) % 4] || s6_played[(player + 3) % 4]) as usize;
                s7_played_by_other_3[player] += (s7_played[(player + 1) % 4] || s7_played[(player + 2) % 4] || s7_played[(player + 3) % 4]) as usize;
                s8_played_by_other_3[player] += (s8_played[(player + 1) % 4] || s8_played[(player + 2) % 4] || s8_played[(player + 3) % 4]) as usize;
                s9_played_by_other_3[player] += (s9_played[(player + 1) % 4] || s9_played[(player + 2) % 4] || s9_played[(player + 3) % 4]) as usize;
            }
        }
    }
    println!("Street 5 played by some other player: {}", format_slice_abs_relative(&s5_played_by_other_3, rounds));
    println!("Street 6 played by some other player: {}", format_slice_abs_relative(&s6_played_by_other_3, rounds));
    println!("Street 7 played by some other player: {}", format_slice_abs_relative(&s7_played_by_other_3, rounds));
    println!("Street 8 played by some other player: {}", format_slice_abs_relative(&s8_played_by_other_3, rounds));
    println!("Street 9 played by some other player: {}", format_slice_abs_relative(&s9_played_by_other_3, rounds));
}

//Odds of losing 4-Q if enemy has phoenix
pub fn evaluate_lose_four_to_queen(db: &DataBase) {
    let contains_bomb = |hand: Hand| (hand.contains_straight_bomb() || hand.contains_four_of_kind_bomb());

    let mut rounds_with_four_to_queen_and_ph_in_enemy = [0; 4];
    let mut rounds_beaten = [0; 4];
    let mut rounds_beaten_bomb_also = [0; 4];
    for game in db.games.iter() {
        for (round, _) in game.rounds.iter() {
            'A: for player in 0..4 {
                let player_hand = round.player_rounds[player].final_14();
                //Check if we have 4-Queen
                let street_hand = prepare_hand(player_hand);
                let four_to_queen = hand!(FOUR, FIVE, SIX, SEVEN, EIGHT, NINE, TEN, JACK, QUEEN);
                if street_hand & four_to_queen != four_to_queen {
                    continue;
                }
                let enemy_hand1 = round.player_rounds[(player + 1) % 4].final_14();
                let enemy_hand2 = round.player_rounds[(player + 3) % 4].final_14();

                if (enemy_hand1 | enemy_hand2) & hand!(PHOENIX) == 0 {
                    continue;
                }
                rounds_with_four_to_queen_and_ph_in_enemy[player] += 1;

                //Check if any enemy has 5-K or 6-A with/without phoenix
                let five_to_king = hand!(FIVE, SIX, SEVEN, EIGHT, NINE, TEN, JACK, QUEEN, KING);
                let six_to_ace = hand!(SIX, SEVEN, EIGHT, NINE, TEN, JACK, QUEEN, KING, ACE);
                let prep1 = prepare_hand(enemy_hand1);
                let prep2 = prepare_hand(enemy_hand2);
                let possibilities = [prep1 & (five_to_king | 1 << 14), prep1 & (six_to_ace | 1 << 14), prep2 & (five_to_king | 1 << 14), prep2 & (six_to_ace | 1 << 14)];
                for poss in possibilities {
                    if poss.count_ones() >= 9 && STREET_DATA_ARRAY[(poss >> PACKING_BITS) as usize] & (1 << (poss & PACKING_BITS_MASK)) != 0u64 {
                        rounds_beaten[player] += 1;
                        rounds_beaten_bomb_also[player] += 1;
                        continue 'A;
                    }
                }
                if contains_bomb(enemy_hand1) || contains_bomb(enemy_hand2) {
                    rounds_beaten_bomb_also[player] += 1;
                }
            }
        }
    }
    println!("Street 9 4-Q beaten by enemy with ph in enemy team: {}", format_slice_abs_relative2(&rounds_beaten, &rounds_with_four_to_queen_and_ph_in_enemy));
    println!("Street 9 4-Q beaten by enemy with ph in enemy team or bomb: {}", format_slice_abs_relative2(&rounds_beaten_bomb_also, &rounds_with_four_to_queen_and_ph_in_enemy));
}