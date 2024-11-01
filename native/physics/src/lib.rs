#[macro_use]
extern crate rustler;

use rapier3d::na::Vector3;
use rapier3d::prelude::*;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};
mod body;
mod init;
mod physics_world;

mod atoms {
    rustler_atoms! {
        atom ok;
        atom error;
    }
}

#[derive(Clone, Debug)]
struct BodyMetadata {
    id: String,
    team_id: Option<String>,
    owner_id: Option<String>,
    class: body::BodyClass,
    rotation: (f32, f32, f32),
    dimensions: (f32, f32, f32),
    hp: i32,
}

fn get_metadata_from_collider_set(
    collider_handle: ColliderHandle,
    collider_set: &ColliderSet,
    metadata_by_handle: &HashMap<RigidBodyHandle, BodyMetadata>,
) -> Option<(RigidBodyHandle, BodyMetadata)> {
    if let Some(body_handle) = collider_set
        .get(collider_handle)
        .map(|c| c.parent().unwrap())
    {
        if let Some(metadata) = metadata_by_handle.get(&body_handle) {
            return Some((body_handle, metadata.clone()));
        }
    }

    return None;
}

fn handle_player_bullet_collision(
    player_handle: RigidBodyHandle,
    bullet_handle: RigidBodyHandle,
    world: &mut physics_world::PhysicsWorld,
    metadata: &mut HashMap<RigidBodyHandle, BodyMetadata>,
) {
    if let Some(body_data) = metadata.get_mut(&player_handle) {
        body_data.hp = (body_data.hp - 1).max(0);
        physics_world::remove_body(world, bullet_handle);
        metadata.remove(&bullet_handle);
    }
}

fn handle_contact(
    contact_event: ContactEvent,
    world: &mut physics_world::PhysicsWorld,
    metadata_by_handle: &mut HashMap<RigidBodyHandle, BodyMetadata>,
) {
    let collider_set = physics_world::get_colliders(world);
    match contact_event {
        ContactEvent::Started(collider_handle_a, collider_handle_b) => {
            let ((body_handle_a, metadata_a), (body_handle_b, metadata_b)) = match (
                get_metadata_from_collider_set(collider_handle_a, collider_set, metadata_by_handle),
                get_metadata_from_collider_set(collider_handle_b, collider_set, metadata_by_handle),
            ) {
                (Some(a), Some(b)) => (a, b),
                _ => return,
            };

            let are_objects_on_same_team = metadata_a.team_id == metadata_b.team_id;

            match (metadata_a.class, metadata_b.class) {
                (body::BodyClass::Player, body::BodyClass::Bullet) => {
                    if !are_objects_on_same_team {
                        handle_player_bullet_collision(
                            body_handle_a,
                            body_handle_b,
                            world,
                            metadata_by_handle,
                        );
                    }
                }
                (body::BodyClass::Bullet, body::BodyClass::Player) => {
                    if !are_objects_on_same_team {
                        handle_player_bullet_collision(
                            body_handle_b,
                            body_handle_a,
                            world,
                            metadata_by_handle,
                        );
                    }
                }
                _ => {}
            };
        }
        _ => {}
    };
}

fn upsert_body(
    world: &mut physics_world::PhysicsWorld,
    metadata_by_handle: &mut HashMap<RigidBodyHandle, BodyMetadata>,
    handle_by_body_id: &mut HashMap<String, RigidBodyHandle>,
    body: &body::Body,
) -> bool {
    match handle_by_body_id.get_mut(&body.id) {
        Some(existing_body_handle) => {
            let (transx, transy, transz) = body.translation;
            let (linvelx, linvely, linvelz) = body.linvel;
            let (angvelx, angvely, angvelz) = body.angvel;
            let (_rotx, _roty, rotz) = body.rotation;
            let existing_body = physics_world::get_body_mut(world, existing_body_handle);
            existing_body.set_translation(Vector3::new(transx, transy, transz), true);
            existing_body.set_linvel(Vector3::new(linvelx, linvely, linvelz), true);
            existing_body.set_angvel(Vector3::new(angvelx, angvely, angvelz), true);
            existing_body.set_rotation(Vector3::z() * rotz, true);
            // TODO: get rotations working natively so that we don't have to keep them
            // in metadata
            metadata_by_handle.insert(
                *existing_body_handle,
                BodyMetadata {
                    id: body.id.clone(),
                    team_id: body.team_id.clone(),
                    owner_id: body.owner_id.clone(),
                    class: body.class,
                    rotation: body.rotation,
                    dimensions: body.dimensions,
                    hp: body.hp,
                },
            );
            false
        }
        None => {
            let body_handle = physics_world::add_body(world, body);
            metadata_by_handle.insert(
                body_handle,
                BodyMetadata {
                    id: body.id.clone(),
                    team_id: body.team_id.clone(),
                    owner_id: body.owner_id.clone(),
                    class: body.class,
                    rotation: body.rotation,
                    dimensions: body.dimensions,
                    hp: body.hp,
                },
            );
            handle_by_body_id.insert(body.id.clone(), body_handle);
            true
        }
    }
}

fn delete_body(
    body: body::Body,
    world: &mut physics_world::PhysicsWorld,
    metadata_by_handle: &mut HashMap<RigidBodyHandle, BodyMetadata>,
    handle_by_body_id: &mut HashMap<String, RigidBodyHandle>,
) {
    match handle_by_body_id.remove(&body.id) {
        Some(handle) => {
            physics_world::remove_body(world, handle);
            metadata_by_handle.remove(&handle);
        }
        None => {}
    };
}

fn add_body_sets(
    input_bodies: HashMap<String, body::Body>,
    world: &mut physics_world::PhysicsWorld,
    metadata_by_handle: &mut HashMap<RigidBodyHandle, BodyMetadata>,
    handle_by_body_id: &mut HashMap<String, RigidBodyHandle>,
) {
    for (_body_id, body) in &input_bodies {
        upsert_body(world, metadata_by_handle, handle_by_body_id, body);
    }
}

fn is_stale(body: &rapier3d::dynamics::RigidBody, metadata: &BodyMetadata) -> bool {
    match metadata.class {
        body::BodyClass::Bullet => is_on_floor(body, metadata) || is_at_rest(body),
        _ => false,
    }
}

fn is_at_rest(body: &rapier3d::dynamics::RigidBody) -> bool {
    let linvel = body.linvel();

    linvel.x.round() == 0.0 && linvel.y.round() == 0.0 && linvel.z.round() == 0.0
}

fn rigid_body_to_body(body: &rapier3d::dynamics::RigidBody, metadata: &BodyMetadata) -> body::Body {
    let translation = body.translation();
    let linvel = body.linvel();
    let angvel = body.angvel();

    body::Body {
        id: metadata.id.clone(),
        team_id: metadata.team_id.clone(),
        owner_id: metadata.owner_id.clone(),
        translation: (translation.x, translation.y, translation.z),
        // TODO: get rotations working natively so that we don't have to keep them
        // in metadata
        rotation: metadata.rotation,
        linvel: (linvel.x, linvel.y, linvel.z),
        angvel: (angvel.x, angvel.y, angvel.z),
        dimensions: metadata.dimensions,
        mass: body.mass(),
        class: metadata.class,
        hp: metadata.hp,
    }
}

fn is_on_floor(body: &rapier3d::dynamics::RigidBody, metadata: &BodyMetadata) -> bool {
    let position = body.position();
    let translation = position.translation;
    let origin_height = translation.z;
    let object_height = metadata.dimensions.2;

    origin_height <= (object_height / 2.0)
}

fn spawn_stdin_channel() -> Receiver<HashMap<String, body::Body>> {
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

pub fn main() {
    let mut writer = std::io::stdout();
    let initial_bodies: HashMap<String, body::Body> = init::get_initial_world();
    let mut metadata_by_handle: HashMap<RigidBodyHandle, BodyMetadata> = HashMap::new();
    let mut handle_by_body_id: HashMap<String, RigidBodyHandle> = HashMap::new();
    let mut world: physics_world::PhysicsWorld = physics_world::init();

    add_body_sets(
        initial_bodies,
        &mut world,
        &mut metadata_by_handle,
        &mut handle_by_body_id,
    );

    let mut updated_handles: HashSet<RigidBodyHandle> = HashSet::new();
    let initial_world_handles: HashSet<RigidBodyHandle> = physics_world::get_bodies(&world)
        .iter()
        .map(|(handle, _body)| handle)
        .collect();

    updated_handles.extend(initial_world_handles);

    let stdin_channel = spawn_stdin_channel();
    let mut is_won: bool = false;

    let integration_dt_ms = physics_world::get_tick_ms(&world);

    while !is_won {
        let mut user_updated_handles = HashSet::new();

        for updated_bodies in stdin_channel.try_iter() {
            for (body_id, body) in &updated_bodies {
                let is_new = upsert_body(
                    &mut world,
                    &mut metadata_by_handle,
                    &mut handle_by_body_id,
                    &body,
                );

                if is_new {
                    user_updated_handles.insert(handle_by_body_id[body_id]);
                }
            }
        }
        updated_handles.extend(user_updated_handles);

        let physics_step_start = Instant::now();
        physics_world::step(&mut world);
        while let Ok(contact_event) = physics_world::get_contact_receiver(&world).try_recv() {
            handle_contact(contact_event, &mut world, &mut metadata_by_handle);
        }
        let physics_updated_handles = physics_world::get_active_handles(&world);

        updated_handles.extend(physics_updated_handles);

        let next_bodies: HashMap<String, body::Body> = updated_handles
            .iter()
            .filter_map(|handle| {
                match (
                    physics_world::get_body(&world, handle),
                    metadata_by_handle.get(handle),
                ) {
                    (Some(rigid_body), Some(metadata)) => {
                        let body_id = metadata.id.clone();
                        let body = rigid_body_to_body(rigid_body, &metadata);

                        if is_stale(rigid_body, &metadata) {
                            delete_body(
                                body,
                                &mut world,
                                &mut metadata_by_handle,
                                &mut handle_by_body_id,
                            );
                            None
                        } else {
                            Some((body_id, body))
                        }
                    }
                    _ => None,
                }
            })
            .collect();

        let remaining_ms =
            (integration_dt_ms - (physics_step_start.elapsed().as_millis() as f32)) as u64;

        if remaining_ms > 0 {
            thread::sleep(Duration::from_millis(remaining_ms));
        }

        match serde_json::to_writer(&mut writer, &next_bodies) {
            Ok(_) => {}
            Err(write_err) => {
                eprintln!("{}", write_err);
            }
        };
        println!("");

        let mut teams_alive = HashSet::new();

        for (_id, body) in &metadata_by_handle {
            match (body.hp, &body.team_id) {
                (_, None) => {}
                (0, _) => {}
                (_nonzero_hp, Some(team_id)) => {
                    teams_alive.insert(team_id);
                }
            };
        }

        is_won = teams_alive.len() == 1;

        if is_won {
            serde_json::to_writer(&mut writer, "game_won");
            println!("");
        }

        updated_handles.clear();
    }
}
