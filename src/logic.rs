// Welcome to
// __________         __    __  .__                               __
// \______   \_____ _/  |__/  |_|  |   ____   ______ ____ _____  |  | __ ____
//  |    |  _/\__  \\   __\   __\  | _/ __ \ /  ___//    \\__  \ |  |/ // __ \
//  |    |   \ / __ \|  |  |  | |  |_\  ___/ \___ \|   |  \/ __ \|    <\  ___/
//  |________/(______/__|  |__| |____/\_____>______>___|__(______/__|__\\_____>
//
// This file can be a nice home for your Battlesnake logic and helper functions.
//
// To get you started we've included code to prevent your Battlesnake from moving backwards.
// For more info see docs.battlesnake.com

mod mm_search;
pub mod simple;

use log::info;
//use rand::seq::SliceRandom;
use serde_json::{json, Value};
use std::hash::{Hash, Hasher};

use crate::{Battlesnake, Board, Coord, Game, GameInfo};

use mm_search::search;
//use simple::{SimpleBoard, SimpleSnake};

// info is called when you create your Battlesnake on play.battlesnake.com
// and controls your Battlesnake's appearance
// TIP: If you open your Battlesnake URL in a browser you should see this data
pub fn info() -> Value {
    info!("INFO");

    return json!({
        "apiversion": "1",
        "author": "Group 18", // TODO: Your Battlesnake Username
        "color": "#e83d84", // TODO: Choose color
        "head": "tiger-king", // TODO: Choose head
        "tail": "coffee", // TODO: Choose tail
    });
}

// start is called when your Battlesnake begins a game
pub fn start(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    // create team mate pairs
    // store timeout
    info!("GAME START");
}

// end is called when your Battlesnake finishes a game
pub fn end(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME OVER");
}

// move is called on every turn and returns your next move
// Valid moves are "up", "down", "left", or "right"
// See https://docs.battlesnake.com/api/example-move for available data
pub fn get_move(
    _game: &Game,
    turn: &i32,
    _board: &Board,
    you: &Battlesnake,
    game_info: &mut GameInfo,
) -> Value {
    let my_id = you.id.clone();
    let team_idx = game_info
        .agent_ids
        .iter()
        .position(|x| x == &my_id)
        .expect("Agent ID not found");

    if game_info.agent_moves[team_idx].len() == *turn as usize + 1 {
        return json!({ "move": game_info.agent_moves[team_idx][*turn as usize] });
    }

    let moves = search(_board, &game_info);

    let teammate_id = game_info.agent_ids[1 - team_idx].clone();
    let board_idx = _board.snakes.iter().position(|s| s.id == my_id).unwrap();
    let chosen = moves.iter().find(|mv| mv.id == board_idx).unwrap().mv;
    game_info.agent_moves[team_idx].push(chosen);
    let teammate_idx =  _board.snakes.iter().position(|s| s.id == teammate_id);
    if !teammate_idx.is_none() {
        game_info.agent_moves[1 - team_idx]
            .push(moves.iter().find(|mv| mv.id == teammate_idx.unwrap()).unwrap().mv);
    }

    info!("MOVE {}: {}", turn, chosen);
    // store down for team mate
    return json!({ "move": chosen });
}

impl Hash for Board {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for food in self.food.iter() {
            food.hash(state);
        }
        for snake in self.snakes.iter() {
            snake.hash(state);
        }
    }
}

impl Hash for Coord {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl Hash for Battlesnake {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        for part in self.body.iter() {
            part.hash(state);
        }
    }
}

impl PartialEq for Battlesnake {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Battlesnake {}
