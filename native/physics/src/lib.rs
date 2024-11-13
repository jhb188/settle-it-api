extern crate rustler;

use rapier3d::prelude::*;
use serde::Serialize;
use serde_json;
use std::collections::HashSet;
use std::io::{BufWriter, Stdout, Write};
use std::thread;
use std::time::{Duration, Instant};

mod body;
mod game;
mod init;
mod physics_world;
mod user_input;
mod util;

fn write_update_to_stdout<A: Serialize>(writer: &mut BufWriter<Stdout>, msg: A) {
    match serde_json::to_writer(&mut *writer, &msg) {
        Ok(_) => {
            writer.write_all(b"\n").expect("Failed to write newline.");
            writer.flush().expect("Failed to flush writer.");
        }
        Err(write_err) => {
            eprintln!("{}", write_err);
        }
    };
}

fn write_body_updates(
    updated_handles: &HashSet<RigidBodyHandle>,
    game_state: &game::Game,
    writer: &mut BufWriter<Stdout>,
) {
    let next_bodies: Vec<body::Body> = updated_handles
        .iter()
        .filter_map(|handle| game::get_body_from_handle(&game_state, handle))
        .collect();

    write_update_to_stdout(writer, &next_bodies);
}

fn sleep_for_remaining_time(loop_start: Instant, integration_dt_ms: f32) {
    let elapsed_ms = loop_start.elapsed().as_millis() as f32;
    if elapsed_ms < integration_dt_ms {
        let sleep_duration = Duration::from_millis((integration_dt_ms - elapsed_ms) as u64);
        thread::sleep(sleep_duration);
    }
}

pub fn main() {
    let mut writer = std::io::BufWriter::new(std::io::stdout());
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
        updated_handles.clear();

        // 5. handle leftover time
        sleep_for_remaining_time(loop_start, integration_dt_ms);
    }

    write_update_to_stdout(&mut writer, "game_won");
}
