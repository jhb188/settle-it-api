#[macro_use]
extern crate rustler;

use rapier3d::prelude::*;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::io::Stdout;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};
mod body;
mod game;
mod init;
mod physics_world;
mod util;

mod atoms {
    rustler_atoms! {
        atom ok;
        atom error;
    }
}

type UserInput = HashMap<String, body::Body>;
type UserInputChannel = Receiver<UserInput>;

fn spawn_stdin_channel() -> UserInputChannel {
    let (tx, rx) = mpsc::channel::<HashMap<String, body::Body>>();
    let reader = std::io::stdin();
    thread::spawn(move || loop {
        let mut buf = String::new();
        reader.read_line(&mut buf);
        let result: serde_json::Result<HashMap<String, body::Body>> = serde_json::from_str(&buf);
        match result {
            Ok(new_bodies) => {
                tx.send(new_bodies);
            }
            Err(decode_err) => {
                eprintln!("{}", decode_err);
            }
        };
    });
    rx
}

fn handle_user_input(
    channel: &UserInputChannel,
    game_state: &mut game::Game,
) -> HashSet<RigidBodyHandle> {
    let mut user_updated_handles = HashSet::new();

    for updated_bodies in channel.try_iter() {
        for (body_id, body) in &updated_bodies {
            let is_new = game::upsert_body(game_state, &body);

            if is_new {
                user_updated_handles.insert(game::get_handle(body_id, &game_state));
            }
        }
    }

    user_updated_handles
}

fn write_body_updates(
    updated_handles: &HashSet<RigidBodyHandle>,
    game_state: &game::Game,
    writer: &mut Stdout,
) {
    let next_bodies: HashMap<String, body::Body> = updated_handles
        .iter()
        .filter_map(
            |handle| match game::get_body_from_handle(&game_state, handle) {
                Some(body) => Some((body.id.clone(), body)),
                _ => None,
            },
        )
        .collect();

    match serde_json::to_writer(writer, &next_bodies) {
        Ok(_) => {}
        Err(write_err) => {
            eprintln!("{}", write_err);
        }
    };
    println!("");
}

pub fn main() {
    let mut writer = std::io::stdout();
    let stdin_channel = spawn_stdin_channel();
    let initial_bodies: Vec<body::Body> = init::get_initial_bodies();
    let mut game_state = game::init();

    // add initial bodies to world and metadata store
    for body in &initial_bodies {
        game::upsert_body(&mut game_state, body);
    }

    let mut updated_handles: HashSet<RigidBodyHandle> = HashSet::new();
    let initial_world_handles = game::get_handles(&game_state);
    updated_handles.extend(initial_world_handles);

    let mut is_won: bool = false;
    let integration_dt_ms = game::get_tick_ms(&game_state);

    while !is_won {
        let loop_start = Instant::now();

        // 1. handle user input
        let user_updated_handles = handle_user_input(&stdin_channel, &mut game_state);
        updated_handles.extend(user_updated_handles);

        // 2. step physics world and handle updates
        let physics_updated_handles = game::step(&mut game_state);
        updated_handles.extend(physics_updated_handles);

        // 3. write updated bodies to channel
        write_body_updates(&updated_handles, &game_state, &mut writer);

        // 4. check win condition and write to channel if necessary
        is_won = game::is_won(&game_state);
        if is_won {
            serde_json::to_writer(&mut writer, "game_won");
            println!("");
        }
        updated_handles.clear();

        // 5. handle leftover time
        let remaining_ms = (integration_dt_ms - (loop_start.elapsed().as_millis() as f32)) as u64;
        if remaining_ms > 0 {
            thread::sleep(Duration::from_millis(remaining_ms));
        }
    }
}
