use crate::bsw_binary_format::{
    Game, PlayerIDGlobal, PlayerIDInternal, PlayerRoundHand, Rank, Round, Score, CALL_GRAND_TICHU,
    CALL_NONE, CALL_TICHU,
};
use crate::tichu_hand::{tichu_one_str_to_hand, TICHU_ONE_ENCODING};
use bitcode::{Decode, Encode};
use memmap2::MmapOptions;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

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
        for path in fs::read_dir("../tichulog_csv/")? {
            let name = path.unwrap().path().display().to_string();
            if name.contains("Spiel_") {
                DataBase::parse_spiel_file(
                    &mut database,
                    &mut player_str_to_id,
                    &mut game_id_to_idx,
                    &name,
                );
            }
        }
        for path in fs::read_dir("../tichulog_csv/")? {
            let name = path.unwrap().path().display().to_string();
            if name.contains("Runde_") {
                DataBase::parse_runde_file(&mut database, &mut game_id_to_idx, &name);
            }
        }
        //Fix extra fields for every PlayerRoundHand and every game
        for (i, game) in database.games.iter_mut().enumerate() {
            for (j, round) in game.rounds.iter_mut().enumerate() {
                for prh in round.player_rounds.iter(){
                    prh.integrity_check();
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
                let card_score_team_1: Score = ((pr0 >> 54) + (pr2 >> 54)) as Score - 50;
                let card_score_team_2: Score = ((pr1 >> 54) + (pr3 >> 54)) as Score - 50;
                if round.player_rounds[0].is_double_win_team_1()
                    || round.player_rounds[1].is_double_win_team_2()
                {
                    continue;
                }
                if card_score_team_1 + card_score_team_2 != 100 {
                    println!("{} {}", card_score_team_1, card_score_team_2);
                    println!("{} {}", i, j);
                    for x in game_id_to_idx.iter() {
                        if *x.1 as usize == i {
                            println!("{}", x.0)
                        }
                    }
                }
                assert!(card_score_team_1 + card_score_team_2 == 100);
                pr0 &= !0xFFC0000000000000;
                pr1 &= !0xFFC0000000000000;
                pr2 &= !0xFFC0000000000000;
                pr3 &= !0xFFC0000000000000;
                pr0 |= ((card_score_team_1 + 25) as u64) << 54;
                pr1 |= ((card_score_team_1 + 25) as u64) << 54;
                pr2 |= ((card_score_team_1 + 25) as u64) << 54;
                pr3 |= ((card_score_team_1 + 25) as u64) << 54;
                round.player_rounds[0].extras = pr0;
                round.player_rounds[1].extras = pr1;
                round.player_rounds[2].extras = pr2;
                round.player_rounds[3].extras = pr3;
                round.integrity_check();
            }
        }
        Ok(database)
    }
    fn parse_spiel_file(
        database: &mut DataBase,
        player_str_to_id: &mut HashMap<String, PlayerIDGlobal>,
        game_id_to_idx: &mut HashMap<u32, u32>,
        path: &str,
    ) {
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
        }
    }
    fn parse_runde_file(
        database: &mut DataBase,
        game_id_to_idx: &mut HashMap<u32, u32>,
        path: &str,
    ) {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        while let Some(line) = lines.next() {
            let line = line.unwrap();
            let mut parts = line.split(";");
            let game_id = parts.next().unwrap().parse::<u32>().unwrap();
            let round = parts.next().unwrap().parse::<usize>().unwrap() - 1;
            let player: PlayerIDInternal = parts.next().unwrap().parse().unwrap();
            let call = match parts.next().unwrap() {
                "T" => CALL_TICHU,
                "GT" => CALL_GRAND_TICHU,
                _ => CALL_NONE,
            };
            let rank = parts.next().unwrap().parse::<Rank>().unwrap() - 1;
            let points = parts.next().unwrap().parse::<i16>().unwrap(); //punkte
            parts.next(); //ergebnis
            parts.next(); //vorsprung
            let first_8 = parts.next().unwrap();
            let first_14 = parts.next().unwrap();
            let mut exch_out = parts.next().unwrap().chars();
            let mut exch_in = parts.next().unwrap().chars();
            let final_14 = parts.next().unwrap();
            if BAD_GAMES.contains(&game_id) {
                continue;
            }
            assert!(game_id_to_idx.contains_key(&game_id));
            let game: &mut Game = database
                .games
                .get_mut(*game_id_to_idx.get(&game_id).unwrap() as usize)
                .unwrap();
            assert!(game.rounds.len() >= round);
            if game.rounds.len() == round {
                game.rounds.push(Round::default())
            }
            assert_eq!(game.rounds.len(), round + 1);
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
            player_round_hand.extras ^= ((points + 25) as u64) << 54;
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
        }
    }
}
