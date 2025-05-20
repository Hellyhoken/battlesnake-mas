#[macro_use]
extern crate rocket;

use log::info;
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::serde::{json::Json, Deserialize};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use rocket::{get, launch, routes, State};
use std::sync::{Arc, Mutex};

mod logic;
use logic::simple::Movement;

type SharedData = Arc<Mutex<HashMap<String, GameInfo>>>;

// API and Response Objects
// See https://docs.battlesnake.com/api

#[derive(Deserialize, Serialize, Debug)]
pub struct Game {
    id: String,
    ruleset: HashMap<String, Value>,
    timeout: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Board {
    height: u32,
    width: i32,
    food: Vec<Coord>,
    snakes: Vec<Battlesnake>,
    hazards: Vec<Coord>,
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        /* build board representation string */
        let mut board: String = "\n|:---------:|".to_owned();
        for y in (0..self.height).rev() {
            board += "\n|";
            for x in 0..self.width {
                let coord = Coord { x: x as i32, y: y as i32 };
                let piece: String = if self.food.contains(&coord) {
                    "f".to_string()
                } else if self.hazards.contains(&coord) {
                    "b".to_string()
                } else if let Some(snake) = self.snakes.iter().find(|s| s.body.contains(&coord)) {
                    if snake.body[0] == coord {
                        "h".to_string()
                    } else {
                        "s".to_string()
                    }
                } else {
                    " ".to_string()
                };
                board += &piece;
            }
            board += "|";
        }
        board += "\n|:---------:|";

        write!(f, "{}", board)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Battlesnake {
    id: String,
    name: String,
    health: i32,
    body: Vec<Coord>,
    head: Coord,
    length: i32,
    latency: String,
    shout: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coord {
    x: i32,
    y: i32,
}

impl std::fmt::Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GameState {
    game: Game,
    turn: i32,
    board: Board,
    you: Battlesnake,
}

pub struct GameInfo {
    id: String,
    timeout: u32,
    agent_ids: [String; 2],
    agent_moves: [Vec<Movement>; 2],
}

#[get("/")]
fn handle_index() -> Json<Value> {
    Json(logic::info())
}

#[post("/start", format = "json", data = "<start_req>")]
fn handle_start(shared_data: &State<SharedData>, start_req: Json<GameState>) -> Status {
    // Store game information in shared data
    let mut data = shared_data.lock().unwrap();
    // Check if the game ID already exists
    let game_id = start_req.game.id.clone();
    let you_id = start_req.you.id.clone();
    if data.contains_key(&game_id) {
        // Add agent ID to the existing game info
        if let Some(game_info) = data.get_mut(&start_req.game.id) {
            game_info.agent_ids[1] = you_id.clone();
        }
    } else {
        // Create a new game info entry
        let game_info = GameInfo {
            id: game_id.clone(),
            timeout: start_req.game.timeout-25,
            agent_ids: [you_id.clone(), String::new()],
            agent_moves: [vec![], vec![]],
        };
        data.insert(game_id.clone(), game_info);
    }
    logic::start(
        &start_req.game,
        &start_req.turn,
        &start_req.board,
        &start_req.you,
    );

    Status::Ok
}

#[post("/move", format = "json", data = "<move_req>")]
fn handle_move(shared_data: &State<SharedData>, move_req: Json<GameState>) -> Json<Value> {
    // Retrieve game information from shared data
    let mut data = shared_data.lock().unwrap();
    let game_id = move_req.game.id.clone();
    let game_info = data.get_mut(&game_id).unwrap_or_else(|| {
        panic!("Game ID {} not found in shared data", game_id)
    });
    let response = logic::get_move(
        &move_req.game,
        &move_req.turn,
        &move_req.board,
        &move_req.you,
        game_info
    );

    Json(response)
}

#[post("/end", format = "json", data = "<end_req>")]
fn handle_end(shared_data: &State<SharedData>, end_req: Json<GameState>) -> Status {
    logic::end(&end_req.game, &end_req.turn, &end_req.board, &end_req.you);
    // Store game information in shared data
    let mut data = shared_data.lock().unwrap();
    // Check if the game ID already exists
    let game_id = end_req.game.id.clone();
    if let Some(_game_info) = data.get_mut(&game_id) {
        // Remove the game info from shared data
        data.remove(&game_id);
    }

    Status::Ok
}

#[launch]
fn rocket() -> _ {
    // Lots of web hosting services expect you to bind to the port specified by the `PORT`
    // environment variable. However, Rocket looks at the `ROCKET_PORT` environment variable.
    // If we find a value for `PORT`, we set `ROCKET_PORT` to that value.
    if let Ok(port) = env::var("PORT") {
        env::set_var("ROCKET_PORT", &port);
    }

    // We default to 'info' level logging. But if the `RUST_LOG` environment variable is set,
    // we keep that value instead.
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    env_logger::init();

    info!("Starting Battlesnake Server...");
    let initial_data: HashMap<String, GameInfo> = HashMap::new();
    let shared_data = Arc::new(Mutex::new(initial_data));

    rocket::build()
        .attach(AdHoc::on_response("Server ID Middleware", |_, res| {
            Box::pin(async move {
                res.set_raw_header("Server", "battlesnake/github/starter-snake-rust");
            })
        }))
        .manage(shared_data)
        .mount(
            "/",
            routes![handle_index, handle_start, handle_move, handle_end],
        )
}
