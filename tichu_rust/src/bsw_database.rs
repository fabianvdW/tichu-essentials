use std::cmp::PartialEq;
use crate::bsw_binary_format::binary_format_constants::{PlayerIDGlobal, Score, PLAYER_0, PLAYER_1, PLAYER_2, PLAYER_3, CALL_TICHU, CALL_GRAND_TICHU, CALL_NONE, PlayerIDInternal, Rank, Team};
use crate::bsw_binary_format::game::{Game, FLAG_EXCLUDED_ROUND, FLAG_GAME_STOPPED_WITHIN_ROUND, FLAG_NO_WINNER_BSW};
use crate::bsw_binary_format::player_round_hand::PlayerRoundHand;
use crate::bsw_binary_format::round::Round;
use crate::bsw_binary_format::round_log::RoundLog;
use crate::tichu_hand::*;
use bitcode::{Decode, Encode};
use memmap2::MmapOptions;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use crate::bsw_binary_format::{game, round};
use crate::bsw_binary_format::trick::{Trick};
use crate::hand;
use datasize::{data_size, DataSize};


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

impl DataBase {
    pub fn collect_winrate_players(&self) -> HashMap<usize, (usize, usize)>{ //PlayerIDGlobal -> (Games, Wins)
        let mut res = HashMap::new();

        for game in self.games.iter(){
            let winner = game.get_winner();
            if winner.is_none(){continue;}
            let winner = winner.unwrap();
            for (id_internal, player_id) in game.player_ids.iter().enumerate(){
                let player_id = *player_id as usize;
                if !res.contains_key(&player_id){
                    res.insert(player_id, (0, 0));
                }
                let (ref mut games, ref mut wins) = res.get_mut(&player_id).unwrap();
                *games += 1;
                if winner == Team::Team1 && id_internal % 2 == 0 || winner == Team::Team2 && id_internal % 2 == 1{
                    *wins += 1;
                }
            }
        }

        res
    }
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
    fn add_skip_round(exclude_rounds: &mut HashMap<u32, Vec<usize>>, game: &mut Game, round: usize) {
        if exclude_rounds.contains_key(&game.original_bsw_id) {
            exclude_rounds.get_mut(&game.original_bsw_id).unwrap().push(round);
        } else {
            exclude_rounds.insert(game.original_bsw_id, vec![round]);
        }
        game.parsing_flags |= FLAG_EXCLUDED_ROUND;
    }
    pub fn from_bsw() -> std::io::Result<DataBase> {
        let mut database = DataBase {
            games: Vec::new(),
            players: Vec::new(),
        };
        let mut player_str_to_id: HashMap<String, PlayerIDGlobal> = HashMap::new();
        let mut bsw_id_to_game: HashMap<u32, Game> = HashMap::new();
        let mut round_results: HashMap<u32, Vec<(Score, Score)>> = HashMap::new();
        let mut exclude_rounds: HashMap<u32, Vec<usize>> = HashMap::new();

        for path in fs::read_dir("../tichulog_csv/")? {
            let name = path?.path().display().to_string();
            if name.contains("Spiel_") {
                DataBase::parse_spiel_file(
                    &mut database,
                    &mut player_str_to_id,
                    &mut bsw_id_to_game,
                    &mut round_results,
                    &name,
                );
            }
        }
        for path in fs::read_dir("../tichulog_csv/")? {
            let name = path?.path().display().to_string();
            if name.contains("Runde_") {
                DataBase::parse_runde_file(&mut bsw_id_to_game, &mut round_results, &mut exclude_rounds, &name);
            }
        }
        for path in fs::read_dir("../tichulog_csv/")? {
            let name = path?.path().display().to_string();
            if name.contains("Zugfolge_") {
                DataBase::parse_zugfolge_file(&mut bsw_id_to_game, &mut exclude_rounds, &name);
            }
        }
        println!("Estimated heap size: {}", data_size(&bsw_id_to_game)); // 9574560 for small dataset. (name.contains Spiel_570)
        println!("Finished parsing! Starting correction!");
        let mut round_count: usize = 0;
        //Fix extra fields for every PlayerRoundHand and every game
        for (game_idx, game) in bsw_id_to_game.iter_mut() {
            for round_num in 0..game.rounds.len() {
                round_count += 1;
                let (round, round_log) = game.rounds.get_mut(round_num).unwrap();
                if exclude_rounds.contains_key(game_idx) && exclude_rounds[game_idx].contains(&round_num) {
                    continue;
                }
                if let Some(err) = round_log.integrity_check(round).err() {
                    //Any of these errors can not be recovered from. The round is trash.
                    println!("Skipping Game {} round {}: {:?} in round_log integrity check. Printing Trick Log:", game_idx, round_num, err);
                    println!("{}", round_log.to_debug_str(round));
                    DataBase::add_skip_round(&mut exclude_rounds, game, round_num);
                    continue;
                }
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
                assert_eq!(pr1 & !0xFFC0000000000000, pr1);
                assert_eq!(pr2 & !0xFFC0000000000000, pr2);
                assert_eq!(pr3 & !0xFFC0000000000000, pr3);
                round.player_rounds[0].extras = pr0;
                round.player_rounds[1].extras = pr1;
                round.player_rounds[2].extras = pr2;
                round.player_rounds[3].extras = pr3;
                //Skip the round if both players of a team call GT.
                if round.player_rounds[0].player_0_call() == CALL_GRAND_TICHU && round.player_rounds[0].player_2_call() == CALL_GRAND_TICHU ||
                    round.player_rounds[0].player_1_call() == CALL_GRAND_TICHU && round.player_rounds[0].player_3_call() == CALL_GRAND_TICHU {
                    println!("Skipping Game {} round {}: Two players of a team both called Grand Tichu.", game_idx, round_num);
                    DataBase::add_skip_round(&mut exclude_rounds, game, round_num);
                    continue;
                }
                let res = round_log.play_round(round);
                if res.is_err() {
                    println!("Skipping Game {} round {}: {:?} in play_round.", game_idx, round_num, res.err().unwrap());
                    DataBase::add_skip_round(&mut exclude_rounds, game, round_num);
                    continue;
                }
                let (log_ranks, score, is_double_win) = res.unwrap();
                //Check that the ranks agree with the calculated ranks.
                let mut round_log_ranks = 0u64;
                round_log_ranks |= (log_ranks[PLAYER_0 as usize] as u64) << 0;
                round_log_ranks |= (log_ranks[PLAYER_1 as usize] as u64) << 2;
                round_log_ranks |= (log_ranks[PLAYER_2 as usize] as u64) << 4;
                round_log_ranks |= (log_ranks[PLAYER_3 as usize] as u64) << 6;
                if ranks != round_log_ranks {
                    println!("Skipping Game {} round {}: Calculated ranks do not match parsed ranks. Printing Trick Log:", game_idx, round_num);
                    println!("{}", round_log.to_debug_str(round));
                    DataBase::add_skip_round(&mut exclude_rounds, game, round_num);
                    continue;
                }

                //We have two different sources of round result.
                //First is round_results vector
                let parsed_round_result = round_results[game_idx][round_num];
                //Second is from the round log + calls points. They must match!
                let mut card_score_team_1 = score[PLAYER_0 as usize] + score[PLAYER_2 as usize];

                //In case of double wins, no card points must be set.
                if round.player_rounds[0].is_double_win_team_1()
                    || round.player_rounds[0].is_double_win_team_2()
                {
                    assert!(is_double_win);
                    if parsed_round_result != round.player_rounds[0].round_score() {
                        println!("Game {} round {}: Parsed round result {:?} does not match calculated round result {:?}(double win). Continuing with our calculated result. Trick Log:", game_idx, round_num, parsed_round_result, round.player_rounds[0].round_score());
                        println!("{}", round_log.to_debug_str(round));
                        game.parsing_flags |= game::FLAG_CHANGED_ROUND_SCORE | game::FLAG_CHANGED_ROUND_SCORE_WITHOUT_DRAGON;
                        round.parsing_flags |= round::FLAG_CHANGED_ROUND_SCORE | round::FLAG_CHANGED_ROUND_SCORE_WITHOUT_DRAGON;
                    }
                } else {
                    assert!(!is_double_win);
                    if score.iter().sum::<Score>() != 100 {
                        println!("Skipping Game {} round {}: Card points {:?} do not add up to 100. Printing Trick Log:", game_idx, round_num, score);
                        println!("{}", round_log.to_debug_str(round));
                        DataBase::add_skip_round(&mut exclude_rounds, game, round_num);
                        continue;
                    }
                    if let Some(fixed) = round_log.try_fix_dragon_gifting(&round) {
                        if fixed {
                            //Recalculate card_scores
                            let res = round_log.play_round(round);
                            if res.is_err() {
                                println!("Skipping Game {} round {}: {:?} in play_round.", game_idx, round_num, res.err().unwrap());
                                DataBase::add_skip_round(&mut exclude_rounds, game, round_num);
                                continue;
                            }
                            let new_score = res.unwrap().1;
                            if new_score.iter().sum::<Score>() != 100 {
                                println!("Skipping Game {} round {}: (Recalculated) Card points {:?} do not add up to 100. Printing Trick Log:", game_idx, round_num, new_score);
                                println!("{}", round_log.to_debug_str(round));
                                DataBase::add_skip_round(&mut exclude_rounds, game, round_num);
                                continue;
                            }
                            game.parsing_flags |= game::FLAG_CHANGED_DRAGON;
                            round.parsing_flags |= round::FLAG_CHANGED_DRAGON;
                            card_score_team_1 = new_score[PLAYER_0 as usize] + new_score[PLAYER_2 as usize];
                        } else {
                            println!("Skipping Game {} round {}: Dragon gifting can not be fixed. Printing Trick Log:", game_idx, round_num);
                            println!("{}", round_log.to_debug_str(round));
                            DataBase::add_skip_round(&mut exclude_rounds, game, round_num);
                            continue;
                        }
                    }
                    assert!(card_score_team_1 >= -25);
                    round.player_rounds[0].extras |= ((card_score_team_1 + 25) as u64) << 54;
                    round.player_rounds[1].extras |= ((card_score_team_1 + 25) as u64) << 54;
                    round.player_rounds[2].extras |= ((card_score_team_1 + 25) as u64) << 54;
                    round.player_rounds[3].extras |= ((card_score_team_1 + 25) as u64) << 54;
                    if parsed_round_result != round.player_rounds[0].round_score() {
                        let dragon_changed = round.parsing_flags & round::FLAG_CHANGED_DRAGON != 0;
                        game.parsing_flags |= game::FLAG_CHANGED_ROUND_SCORE;
                        round.parsing_flags |= round::FLAG_CHANGED_ROUND_SCORE;
                        if !dragon_changed {
                            game.parsing_flags |= game::FLAG_CHANGED_ROUND_SCORE_WITHOUT_DRAGON;
                            round.parsing_flags |= round::FLAG_CHANGED_ROUND_SCORE_WITHOUT_DRAGON;
                            println!("Game {} round {}: Parsed round result {:?} does not match calculated round result {:?} (no dragon gift change). Continuing with our calculated result. Trick Log:", game_idx, round_num, parsed_round_result, round.player_rounds[0].round_score());
                            println!("{}", round_log.to_debug_str(round));
                        }
                    }
                }

                if let Some(err) = round.integrity_check().err() {
                    //Any of these errors can not be recovered from. The round is trash.
                    println!("Skipping Game {} round {}: {:?} in Round integrity check.", game_idx, round_num, err);
                    DataBase::add_skip_round(&mut exclude_rounds, game, round_num);
                    continue;
                }
            }
        }
        println!("Correctly parsed {} rounds!", round_count);
        println!("Starting to delete excluded rounds from database! Excluded rounds: {}", exclude_rounds.iter().fold(0, |acc, inc| acc + inc.1.len()));
        for (game_idx, exclude_rounds) in exclude_rounds.iter() {
            let game: &mut Game = bsw_id_to_game.get_mut(game_idx).unwrap();
            if game.rounds.len() == exclude_rounds.len() {
                println!("Deleting game {} from database! No more rounds.", *game_idx);
                bsw_id_to_game.remove(game_idx);
                continue;
            }
            //Assert excluded rounds is sorted.
            assert!(exclude_rounds.is_sorted());
            for exclude_idx in exclude_rounds.iter().rev() {
                game.rounds.remove(*exclude_idx);
            }
        }
        //Collect bsw_id_to_game into database
        database.games = bsw_id_to_game.into_values().collect();
        Ok(database)
    }
    fn parse_spiel_file(
        database: &mut DataBase,
        player_str_to_id: &mut HashMap<String, PlayerIDGlobal>,
        game_id_to_idx: &mut HashMap<u32, Game>,
        round_results: &mut HashMap<u32, Vec<(Score, Score)>>,
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
                {
                    parts.next(); //PlayerIdInternal, not important
                    parts.next().unwrap().parse::<u8>().unwrap() //Win or not
                }
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
            assert!(!game_id_to_idx.contains_key(&chunk[0].0));
            let mut player_ids: [PlayerIDGlobal; 4] = [0; 4];
            for player in 0..4 {
                let player_id = {
                    let player_name = &chunk[player].1;
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
            let parsing_flags = if chunk.iter().all(|x| x.2 == 0) { FLAG_NO_WINNER_BSW } else { 0 };
            game_id_to_idx.insert(chunk[0].0, Game {
                rounds: Vec::new(),
                player_ids,
                original_bsw_id: chunk[0].0,
                parsing_flags,
            });
            round_results.insert(chunk[0].0, Vec::new());
        }
    }
    fn parse_runde_file(
        game_id_to_idx: &mut HashMap<u32, Game>,
        round_results: &mut HashMap<u32, Vec<(Score, Score)>>,
        exclude_rounds: &mut HashMap<u32, Vec<usize>>,
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

            assert!(game_id_to_idx.contains_key(&game_id));
            let game: &mut Game = game_id_to_idx.get_mut(&game_id).unwrap();
            let game_round_results: &mut Vec<(Score, Score)> = round_results.get_mut(&game_id).unwrap();

            assert!(game.rounds.len() >= round);
            if game.rounds.len() == round {
                game.rounds.push((Round::default(), RoundLog::default()));
                assert_eq!(game_round_results.len(), round);
                game_round_results.push((0, 0));
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
                .unwrap().0
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

            if let Some(err) = player_round_hand.integrity_check().err() {
                //Any of these errors can not be recovered from. The round is trash.
                println!("Skipping Game {} round {}: {:?} in PlayerRoundHand {:?}. File {}.", game_id, round, err, player_round_hand, path);
                DataBase::add_skip_round(exclude_rounds, game, round);
                continue;
            }
            if player_round_hand.final_14() != tichu_one_str_to_hand(final_14) {
                //This is also enough reason to discard the round and invite further investigation.
                println!("Skipping Game {} round {}: PlayerRoundHand {} does not match parsed final 14 {}. File {}.",
                         game_id, round, player_round_hand.final_14().pretty_print(),
                         tichu_one_str_to_hand(final_14).pretty_print(), path);
                DataBase::add_skip_round(exclude_rounds, game, round);
            }
        }
    }

    fn parse_zugfolge_file(
        game_id_to_idx: &mut HashMap<u32, Game>,
        exclude_rounds: &mut HashMap<u32, Vec<usize>>,
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

            assert!(game_id_to_idx.contains_key(&game_id));
            let game: &mut Game = game_id_to_idx.get_mut(&game_id).unwrap();
            let round = parts.next().unwrap().parse::<usize>().unwrap() - 1;
            if exclude_rounds.contains_key(&game_id) && exclude_rounds[&game_id].contains(&round) {
                continue;
            }
            if game.rounds.len() <= round {
                game.parsing_flags |= FLAG_GAME_STOPPED_WITHIN_ROUND;
                continue;
            }
            let _ = parts.next().unwrap().parse::<usize>().unwrap() - 1; //Trick num.

            let (_, round_log) = game.rounds.get_mut(round).unwrap();


            let mut trick = Trick::default();
            let trick_type_str = parts.next().unwrap();
            trick.trick_type = trick_type_str_to_trick_type(trick_type_str);

            let trick_length = parts.next().unwrap().parse::<usize>().unwrap();
            trick.trick_log = Vec::with_capacity(trick_length);
            let trick_players: Vec<PlayerIDInternal> = parts.next().unwrap().chars()
                .map(|c| c.to_digit(10).unwrap() as PlayerIDInternal)
                .collect();
            assert_eq!(trick_length, trick_players.len());
            let trick_cards = parts.next().unwrap();
            let trick_hands = trick_cards.split("|");

            for (i, trick_hand) in trick_hands.enumerate() {
                if trick_hand.len() == 0 {
                    continue;
                }
                let mut hand_bb = 0u64;
                let mut chars = trick_hand.chars().peekable();
                while let Some(c) = chars.next() {
                    let new_card_index = TICHU_ONE_ENCODING[&c];
                    let new_card = hand!(new_card_index);
                    assert_eq!(new_card & hand_bb, 0u64);
                    hand_bb ^= new_card;
                    if chars.peek() == Some(&'(') {
                        chars.next(); // consume '('
                        let next = chars.next().unwrap();
                        if hand_bb & hand!(MAHJONG) != 0u64 {
                            assert!(round_log.mahjong_wish.is_none());
                            round_log.mahjong_wish = Some(card_wish_to_cardtype(next));
                        } else if hand_bb & hand!(DRAGON) != 0u64 {
                            assert!(round_log.dragon_player_gift.is_none());
                            round_log.dragon_player_gift = Some(next.to_digit(10).unwrap() as PlayerIDInternal);
                        } else {
                            //Mahjong is served. Check that it holds
                            assert!(round_log.mahjong_wish.is_some());
                            let wish = round_log.mahjong_wish.unwrap();
                            assert_ne!(hand_bb & MASK_FOUR_OF_KIND[(wish - 1) as usize], 0u64);
                        }
                        chars.next(); // consume ')'
                        assert!(chars.next().is_none());
                    }
                }
                trick.trick_log.push((trick_players[i], hand_bb));
            }

            assert_eq!(trick.trick_log.len(), trick_length);
            trick.serialize_into(round_log);
        }
    }
}
