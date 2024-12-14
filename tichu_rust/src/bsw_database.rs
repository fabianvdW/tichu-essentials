use crate::bsw_binary_format::*;
use crate::tichu_hand::*;
use bitcode::{Decode, Encode};
use memmap2::MmapOptions;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use crate::hand;

pub fn trick_type_str_to_trick_type(trick_type: &str) -> TrickType {
    match trick_type {
        "1" => TRICK_SINGLETON,
        "2" => TRICK_PAIRS,
        "3" => TRICK_TRIPLETS,
        "T2" => TRICK_PAIRSTREET4,
        "T3" => TRICK_PAIRSTREET6,
        "T4" => TRICK_PAIRSTREET8,
        "T5" => TRICK_PAIRSTREET10,
        "T6" => TRICK_PAIRSTREET12,
        "T7" => TRICK_PAIRSTREET14,
        "S5" => TRICK_STREET5,
        "S6" => TRICK_STREET6,
        "S7" => TRICK_STREET7,
        "S8" => TRICK_STREET8,
        "S9" => TRICK_STREET9,
        "S10" => TRICK_STREET10,
        "S11" => TRICK_STREET11,
        "S12" => TRICK_STREET12,
        "S13" => TRICK_STREET13,
        "S14" => TRICK_STREET14,
        "F" => TRICK_FULLHOUSE,
        "D" => TRICK_DOG,
        "B4" => TRICK_BOMB4,
        "B5" => TRICK_BOMB5,
        "B6" => TRICK_BOMB6,
        "B7" => TRICK_BOMB7,
        "B8" => TRICK_BOMB8,
        "B9" => TRICK_BOMB9,
        "B10" => TRICK_BOMB10,
        "B11" => TRICK_BOMB11,
        "B12" => TRICK_BOMB12,
        "B13" => TRICK_BOMB13,
        _ => unreachable!(),
    }
}

pub fn card_wish_to_cardtype(card_wish: char) -> CardType {
    ".23456789TJQKA".find(card_wish).unwrap() as CardType
}

#[derive(Encode, Decode)]
pub struct DataBase {
    pub games: Vec<Game>,
    pub players: Vec<String>, //Indexed by PlayerIDGlobal
}
const BAD_GAMES: [u32; 9] = [
    1107083, 155317, 1837970, 288123, 39186, 500114, 50442, 927659, 968807,
]; //Game ID's of BSW dataset that are bugged. Hardcoded to exclude them.
impl DataBase {
    pub fn write(&self, path: &str) -> std::io::Result<()> {
        let encoded = bitcode::encode(self);
        let mut file = File::create(path)?;
        file.write_all(&encoded)?;
        Ok(())
    }
    pub fn read(path: &str) -> std::io::Result<DataBase> {
        let file = File::open(path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        Ok(bitcode::decode(&mmap[..]).unwrap())
    }

    pub fn from_bsw() -> std::io::Result<DataBase> {
        let mut database = DataBase {
            games: Vec::new(),
            players: Vec::new(),
        };
        let mut player_str_to_id: HashMap<String, PlayerIDGlobal> = HashMap::new();
        let mut game_id_to_idx: HashMap<u32, u32> = HashMap::new();
        let mut round_results: Vec<Vec<(Score, Score)>> = Vec::new();
        for path in fs::read_dir("../tichulog_csv/")? {
            let name = path?.path().display().to_string();
            if name.contains("Spiel_") {
                DataBase::parse_spiel_file(
                    &mut database,
                    &mut player_str_to_id,
                    &mut game_id_to_idx,
                    &mut round_results,
                    &name,
                );
            }
        }
        for path in fs::read_dir("../tichulog_csv/")? {
            let name = path?.path().display().to_string();
            if name.contains("Runde_") {
                DataBase::parse_runde_file(&mut database, &mut game_id_to_idx, &mut round_results, &name);
            }
        }
        for path in fs::read_dir("../tichulog_csv/")? {
            let name = path?.path().display().to_string();
            if name.contains("Zugfolge_") {
                DataBase::parse_zugfolge_file(&mut database, &mut game_id_to_idx, &name);
            }
        }
        let mut round_count: usize = 0;
        let mut round_count_dw: usize = 0;
        //Fix extra fields for every PlayerRoundHand and every game
        for (i, game) in database.games.iter_mut().enumerate() {
            assert_eq!(game.rounds.len(), game.round_logs.len());
            for (j, (round, round_log)) in game.rounds.iter_mut().zip(game.round_logs.iter()).enumerate() {
                round_log.integrity_check(round);
                let (mut pr0, mut pr1, mut pr2, mut pr3) = (
                    round.player_rounds[0].extras,
                    round.player_rounds[1].extras,
                    round.player_rounds[2].extras,
                    round.player_rounds[3].extras,
                );
                let calls = ((pr0 >> 36) | (pr1 >> 36) | (pr2 >> 36) | (pr3 >> 36)) & 0xFF;
                pr0 |= calls << 36;
                pr1 |= calls << 36;
                pr2 |= calls << 36;
                pr3 |= calls << 36;
                let ranks = ((pr0 >> 46) | (pr1 >> 46) | (pr2 >> 46) | (pr3 >> 46)) & 0xFF;
                pr0 |= ranks << 46;
                pr1 |= ranks << 46;
                pr2 |= ranks << 46;
                pr3 |= ranks << 46;
                assert_eq!(pr0 & !0xFFC0000000000000, pr0);
                assert_eq!(pr1 & !0xFFC0000000000000, pr0);
                assert_eq!(pr2 & !0xFFC0000000000000, pr0);
                assert_eq!(pr3 & !0xFFC0000000000000, pr0);
                round.player_rounds[0].extras = pr0;
                round.player_rounds[1].extras = pr1;
                round.player_rounds[2].extras = pr2;
                round.player_rounds[3].extras = pr3;
                round_count += 1;

                let (log_ranks, score, is_double_win) = round_log.play_round(round);
                //Check that the ranks agree with the calculated ranks.
                let mut round_log_ranks = 0u64;
                round_log_ranks |= (log_ranks[PLAYER_0 as usize] as u64) << 0;
                round_log_ranks |= (log_ranks[PLAYER_1 as usize] as u64) << 2;
                round_log_ranks |= (log_ranks[PLAYER_2 as usize] as u64) << 4;
                round_log_ranks |= (log_ranks[PLAYER_3 as usize] as u64) << 6;
                assert_eq!(ranks, round_log_ranks);

                //We have two different sources of round result.
                //First is round_results vector
                let parsed_round_result = round_results[i][j];
                //Second is from the round log + calls points. They must match!
                let card_score_team_1 = score[PLAYER_0 as usize] + score[PLAYER_2 as usize];

                //In case of double wins, no card points must be set.
                if round.player_rounds[0].is_double_win_team_1()
                    || round.player_rounds[0].is_double_win_team_2()
                {
                    assert!(is_double_win);
                    assert_eq!(parsed_round_result, round.player_rounds[0].round_score());
                    round_count_dw += 1;
                } else {
                    assert!(!is_double_win);
                    assert!(card_score_team_1 >= -25);
                    round.player_rounds[0].extras |= ((card_score_team_1 + 25) as u64) << 54;
                    round.player_rounds[1].extras |= ((card_score_team_1 + 25) as u64) << 54;
                    round.player_rounds[2].extras |= ((card_score_team_1 + 25) as u64) << 54;
                    round.player_rounds[3].extras |= ((card_score_team_1 + 25) as u64) << 54;
                    assert_eq!(parsed_round_result, round.player_rounds[0].round_score());
                }

                round.integrity_check();
            }
        }
        println!("{}", round_count);
        println!("{}", round_count_dw);
        Ok(database)
    }
    fn parse_spiel_file(
        database: &mut DataBase,
        player_str_to_id: &mut HashMap<String, PlayerIDGlobal>,
        game_id_to_idx: &mut HashMap<u32, u32>,
        round_results: &mut Vec<Vec<(Score, Score)>>,
        path: &str, )
    {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let parse_line = |line: &str| {
            let mut parts = line.split(";");
            (
                parts.next().unwrap().parse::<u32>().unwrap(),
                parts.next().unwrap().parse::<String>().unwrap(),
            )
        };
        while let (Some(line1), Some(line2), Some(line3), Some(line4)) =
            (lines.next(), lines.next(), lines.next(), lines.next())
        {
            // Process the chunk of 4 lines
            let chunk = [
                parse_line(&line1.unwrap()),
                parse_line(&line2.unwrap()),
                parse_line(&line3.unwrap()),
                parse_line(&line4.unwrap()),
            ];
            assert_eq!(chunk[0].0, chunk[1].0);
            assert_eq!(chunk[1].0, chunk[2].0);
            assert_eq!(chunk[2].0, chunk[3].0);
            if BAD_GAMES.contains(&chunk[0].0) {
                continue;
            }
            assert!(!game_id_to_idx.contains_key(&chunk[0].0));
            let game_idx = database.games.len() as u32;
            game_id_to_idx.insert(chunk[0].0, game_idx);
            let mut player_ids: [PlayerIDGlobal; 4] = [0; 4];
            for player in 0..4 {
                let player_id = {
                    let player_name = &chunk[0].1;
                    if let Some(x) = player_str_to_id.get(player_name) {
                        *x
                    } else {
                        let player_id = database.players.len() as PlayerIDGlobal;
                        player_str_to_id.insert(player_name.to_string(), player_id);
                        database.players.push(player_name.to_string());
                        player_id
                    }
                };
                player_ids[player] = player_id;
            }

            database.games.push(Game {
                rounds: Vec::new(),
                round_logs: Vec::new(),
                player_ids,
            });
            round_results.push(Vec::new());
        }
    }
    fn parse_runde_file(
        database: &mut DataBase,
        game_id_to_idx: &mut HashMap<u32, u32>,
        round_results: &mut Vec<Vec<(Score, Score)>>,
        path: &str,
    )
    {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        while let Some(line) = lines.next() {
            let line = line.unwrap();
            let mut parts = line.split(";");
            let game_id = parts.next().unwrap().parse::<u32>().unwrap();
            let round = parts.next().unwrap().parse::<usize>().unwrap() - 1;
            if BAD_GAMES.contains(&game_id) {
                continue;
            }
            assert!(game_id_to_idx.contains_key(&game_id));
            let game: &mut Game = database
                .games
                .get_mut(*game_id_to_idx.get(&game_id).unwrap() as usize)
                .unwrap();
            let game_round_results: &mut Vec<(Score, Score)> = round_results.get_mut(*game_id_to_idx.get(&game_id).unwrap() as usize).unwrap();

            assert!(game.rounds.len() >= round);
            if game.rounds.len() == round {
                game.rounds.push(Round::default());
                assert_eq!(game_round_results.len(), round);
                game_round_results.push((0, 0));
                assert_eq!(game.round_logs.len(), round);
                game.round_logs.push(RoundLog::default());
            }
            assert_eq!(game.rounds.len(), round + 1);


            let player: PlayerIDInternal = parts.next().unwrap().parse().unwrap();
            let call = match parts.next().unwrap() {
                "T" => CALL_TICHU,
                "GT" => CALL_GRAND_TICHU,
                _ => CALL_NONE,
            };
            let rank = parts.next().unwrap().parse::<Rank>().unwrap() - 1;
            parts.next(); // Punkte (sadly, this field is wrong!) We have to fix it ourselves..
            if player == PLAYER_0 || player == PLAYER_1 {
                let ergebnis = parts.next().unwrap().parse::<Score>().unwrap();
                let old_score = game_round_results[round];
                let new_score = if player == PLAYER_0 { (ergebnis, old_score.1) } else { (old_score.0, ergebnis) };
                game_round_results[round] = new_score;
            } else {
                parts.next();
            }
            parts.next(); //vorsprung
            let first_8 = parts.next().unwrap();
            let first_14 = parts.next().unwrap();
            let mut exch_out = parts.next().unwrap().chars();
            let mut exch_in = parts.next().unwrap().chars();
            let final_14 = parts.next().unwrap();
            let player_round_hand: &mut PlayerRoundHand = game
                .rounds
                .get_mut(round)
                .unwrap()
                .player_rounds
                .get_mut(player as usize)
                .unwrap();
            player_round_hand.first_8 = tichu_one_str_to_hand(first_8);
            player_round_hand.first_14 = tichu_one_str_to_hand(first_14);
            for i in 0..3 {
                player_round_hand.extras ^=
                    (TICHU_ONE_ENCODING[&exch_out.next().unwrap()] as u64) << (i * 6);
            }
            for i in 0..3 {
                player_round_hand.extras ^=
                    (TICHU_ONE_ENCODING[&exch_in.next().unwrap()] as u64) << ((i + 3) * 6);
            }

            player_round_hand.extras ^= (call as u64) << (36 + 2 * player);
            player_round_hand.extras ^= (player as u64) << 44;
            player_round_hand.extras ^= (rank as u64) << (46 + 2 * player);
            //This is how BAD_GAMES is found.
            /*if player_round_hand.final_14() != tichu_one_str_to_hand(final_14) || player_round_hand.final_14().count_ones() != 14 {
                println!("Bad game {} in {}. Round {}", game_id, path, round+1);
                println!("{}", first_8);
                println!("{}", first_14);
                println!("{}", final_14);
                println!("{}", player_round_hand.first_14.pretty_print());
                println!("{}", tichu_one_str_to_hand(final_14).pretty_print());
            }*/
            assert_eq!(
                player_round_hand.final_14(),
                tichu_one_str_to_hand(final_14)
            );
            player_round_hand.integrity_check();
        }
    }

    fn parse_zugfolge_file(
        database: &mut DataBase,
        game_id_to_idx: &mut HashMap<u32, u32>,
        path: &str,
    )
    {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        while let Some(line) = lines.next() {
            let line = line.unwrap();
            let mut parts = line.split(";");
            let game_id = parts.next().unwrap().parse::<u32>().unwrap();
            if BAD_GAMES.contains(&game_id) {
                continue;
            }
            assert!(game_id_to_idx.contains_key(&game_id));
            let game: &mut Game = database
                .games
                .get_mut(*game_id_to_idx.get(&game_id).unwrap() as usize)
                .unwrap();
            let round = parts.next().unwrap().parse::<usize>().unwrap() - 1;
            assert!(game.round_logs.len() > round);
            let round_log = game.round_logs.get_mut(round).unwrap();


            let trick = parts.next().unwrap().parse::<usize>().unwrap() - 1;
            assert_eq!(round_log.log.len(), trick);
            let mut trick = Trick::default();
            let trick_type_str = parts.next().unwrap();
            trick.trick_type = trick_type_str_to_trick_type(trick_type_str);

            let trick_length = parts.next().unwrap().parse::<usize>().unwrap(); //TODO: Check that it matches at the end
            let trick_players: Vec<PlayerIDInternal> = parts.next().unwrap().chars()
                .map(|c| c.to_digit(10).unwrap() as PlayerIDInternal)
                .collect();
            assert_eq!(trick_length, trick_players.len());
            let trick_cards = parts.next().unwrap();
            let trick_hands = trick_cards.split("|");

            for (i, trick_hand) in trick_hands.enumerate() {
                let mut hand = 0u64;
                let mut chars = trick_hand.chars().peekable();
                while let Some(c) = chars.next() {
                    let new_card = hand!(TICHU_ONE_ENCODING[&c]);
                    assert!(new_card & hand == 0u64);
                    hand ^= new_card;
                    if chars.peek() == Some(&'(') {
                        chars.next(); // consume '('
                        let next = chars.next().unwrap();
                        if hand & hand!(MAHJONG) != 0u64 {
                            assert!(round_log.mahjong_wish.is_none());
                            round_log.mahjong_wish = Some(card_wish_to_cardtype(next));
                        } else if hand & hand!(DRAGON) != 0u64 {
                            assert!(round_log.dragon_player_gift.is_none());
                            round_log.dragon_player_gift = Some(next.to_digit(10).unwrap() as PlayerIDInternal);
                        } else {
                            //Mahjong is served. Check that it holds
                            assert!(round_log.mahjong_wish.is_some());
                            let wish = round_log.mahjong_wish.unwrap();
                            assert!(hand & MASK_FOUR_OF_KIND[(wish -1) as usize] != 0u64);
                        }
                        chars.next(); // consume ')'
                        assert!(chars.next().is_none());
                    }
                }
                let tagged_hand = u64::construct(trick_players[i], hand);
                trick.trick_log.push(tagged_hand);
            }

            assert_eq!(trick.trick_log.len(), trick_length);
            round_log.log.push(trick);
        }
    }
}
