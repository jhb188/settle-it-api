use rapier3d::prelude::RigidBodyHandle;

use crate::body;
use crate::game;
use std::collections::HashSet;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
enum UserInput {
    Move {
        id: String,
        x: f32,
        y: f32,
    },
    Rotate {
        id: String,
        rotation_angle: f32,
    },
    Jump {
        id: String,
        linvel_z: f32,
    },
    Shoot {
        #[serde(flatten)]
        body: body::Body,
    },
    AddPlayer {
        #[serde(flatten)]
        body: body::Body,
    },
}
type UserInputChannel = Receiver<String>;

pub fn spawn_input_channel() -> Result<UserInputChannel, std::io::Error> {
    let (tx, rx) = mpsc::channel::<String>();
    let reader = std::io::stdin();
    thread::spawn(move || {
        let mut buf = String::new();
        while reader.read_line(&mut buf).is_ok() {
            let line = buf.trim().to_string();
            if !line.is_empty() {
                if tx.send(line).is_err() {
                    eprintln!("Failed to send user input to channel.");
                    break;
                }
            }
            buf.clear();
        }
    });
    Ok(rx)
}

pub fn handle_user_input(
    channel: &UserInputChannel,
    game_state: &mut game::Game,
) -> HashSet<RigidBodyHandle> {
    let mut user_updated_handles = HashSet::new();

    for input_line in channel.try_iter() {
        match serde_json::from_str::<UserInput>(&input_line) {
            Ok(action) => match action {
                UserInput::Move { id, x, y } => {
                    game::move_body(game_state, &id, x, y);
                    user_updated_handles.insert(game::get_handle(&id, game_state));
                }
                UserInput::Rotate { id, rotation_angle } => {
                    game::rotate_body(game_state, &id, rotation_angle);
                    user_updated_handles.insert(game::get_handle(&id, game_state));
                }
                UserInput::Jump { id, linvel_z } => {
                    game::jump_body(game_state, &id, linvel_z);
                    user_updated_handles.insert(game::get_handle(&id, game_state));
                }
                UserInput::Shoot { body } => {
                    game::upsert_body(game_state, &body);
                    user_updated_handles.insert(game::get_handle(&body.id, game_state));
                }
                UserInput::AddPlayer { body } => {
                    game::upsert_body(game_state, &body);
                    user_updated_handles.insert(game::get_handle(&body.id, game_state));
                }
            },
            Err(err) => {
                eprintln!("Failed to parse user input: {}", err);
            }
        }
    }

    user_updated_handles
}
