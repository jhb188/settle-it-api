extern crate rustler;

use rapier3d::prelude::*;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::io::Stdout;
use std::thread;
use std::time::{Duration, Instant};

mod body;
mod game;
mod init;
mod physics_world;
mod user_input;
mod util;

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

fn sleep_for_remaining_time(loop_start: Instant, integration_dt_ms: f32) {
    let elapsed_ms = loop_start.elapsed().as_millis() as f32;
    if elapsed_ms < integration_dt_ms {
        let sleep_duration = Duration::from_millis((integration_dt_ms - elapsed_ms) as u64);
        thread::sleep(sleep_duration);
    }
}

pub fn main() {
    let mut writer = std::io::stdout();
    let stdin_channel = user_input::spawn_input_channel().expect("Failed to spawn stdin channel");
    let mut game_state = game::init();

    let mut updated_handles: HashSet<RigidBodyHandle> = HashSet::new();
    let initial_world_handles = game::get_handles(&game_state);
    updated_handles.extend(initial_world_handles);

    let mut is_won: bool = false;
    let integration_dt_ms = game::get_tick_ms(&game_state);

    while !is_won {
        let loop_start = Instant::now();

        // 1. handle user input
        let user_updated_handles = user_input::handle_user_input(&stdin_channel, &mut game_state);
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
        sleep_for_remaining_time(loop_start, integration_dt_ms);
    }
}
