use crate::hand;
use crate::tichu_hand::*;
use bitcode::{Decode, Encode};
use memmap2::MmapOptions;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

//TODO: Use rkyv or binrw or bitcode for binary serialization/deserialization
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
#[derive(Encode, Decode, Copy, Clone)]
pub struct PlayerRoundHand {
    pub first_8: Hand,
    pub first_14: Hand,
    pub extras: u64,
}
#[derive(Encode, Decode)]
pub struct Round {
    pub player_rounds: [PlayerRoundHand; 4],
}

pub type TaggedCardIndex = u8; //Lower 6 bits are CardIndex, upper 2 CardIndex are Tag
pub type Tag = u8; //Either PlayerId or
pub const TAG_NEW_TRICK: Tag = 4;
pub const TAG_BOMB: Tag = 5;

pub trait TaggedCardndexT {
    fn get_tag(&self) -> Tag;
    fn get_card_index(&self) -> CardIndex;
}
#[derive(Encode, Decode)]
pub struct RoundLog {
    pub mahjong_wish: Option<CardIndex>,
    pub dragon_player_gift: Option<PlayerIDInternal>,
    pub log: Vec<Vec<Trick>>,
    //TODO: This and all subclcasses
}
#[derive(Encode, Decode)]
pub struct Trick {
    pub trick_log: Vec<TaggedCardIndex>,
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
}
#[derive(Encode, Decode)]
pub struct DataBase {
    pub games: Vec<Game>,
    pub players: Vec<String>, //Indexed by PlayerIDGlobal
}
const BAD_GAMES: [u32; 9] = [1107083, 155317, 1837970, 288123, 39186, 500114, 50442, 927659, 968807]; //Game ID's of BSW dataset that are bugged. Hardcoded to exclude them.
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
            for (j,round) in game.rounds.iter_mut().enumerate() {
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
                if card_score_team_1 + card_score_team_2 != 100{
                    println!("{} {}", card_score_team_1, card_score_team_2);
                    println!("{} {}", i, j);
                    for x in game_id_to_idx.iter() {
                        if *x.1 as usize == i{
                            println!("{}", x.0)
                        }
                    }
                }
                assert!(card_score_team_1 + card_score_team_2 == 100);
                pr0 &= 0xFF00000000000000;
                pr1 &= 0xFF00000000000000;
                pr2 &= 0xFF00000000000000;
                pr3 &= 0xFF00000000000000;
                pr0 |= ((card_score_team_1 + 25) as u64) << 54;
                pr1 |= ((card_score_team_1 + 25) as u64) << 54;
                pr2 |= ((card_score_team_1 + 25) as u64) << 54;
                pr3 |= ((card_score_team_1 + 25) as u64) << 54;
                round.player_rounds[0].extras = pr0;
                round.player_rounds[1].extras = pr1;
                round.player_rounds[2].extras = pr2;
                round.player_rounds[3].extras = pr3;
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
            if BAD_GAMES.contains(&chunk[0].0){
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
            if BAD_GAMES.contains(&game_id){
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
                player_round_hand.extras ^= (TICHU_ONE_ENCODING[&exch_out.next().unwrap()] as u64) << (i * 6);
            }
            for i in 0..3 {
                player_round_hand.extras ^= (TICHU_ONE_ENCODING[&exch_in.next().unwrap()] as u64) << ((i + 3) * 6);
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
            assert_eq!(player_round_hand.final_14(), tichu_one_str_to_hand(final_14));
        }
    }
}
impl PlayerRoundHand {
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
    pub fn round_score_relative(&self) -> (Score, Score) {
        //Reported relative to own team;
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
        let relative_score = self.round_score_relative();
        relative_score.0 - relative_score.1
    }
}
impl Game {
    pub fn get_winner(&self) -> Team {
        let mut score_team_0: Score = 0;
        let mut score_team_1: Score = 0;
        for round in &self.rounds {
            let round_scores = round.player_rounds[PLAYER_0 as usize].round_score_relative();
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

impl Default for PlayerRoundHand {
    fn default() -> Self {
        PlayerRoundHand {
            first_8: 0,
            first_14: 0,
            extras: 0,
        }
    }
}
impl Default for Round {
    fn default() -> Self {
        Round {
            player_rounds: [
                PlayerRoundHand::default(),
                PlayerRoundHand::default(),
                PlayerRoundHand::default(),
                PlayerRoundHand::default(),
            ],
        }
    }
}
