use rapier3d::math::Rotation;
use rapier3d::prelude::ColliderHandle;
use rapier3d::prelude::CollisionEvent;
use rapier3d::prelude::RigidBodyHandle;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::body;
use crate::physics_world;
use crate::util::to_vec3;

pub struct Game {
    world: physics_world::PhysicsWorld,
    metadata_by_handle: HashMap<RigidBodyHandle, BodyMetadata>,
    handle_by_body_id: HashMap<String, RigidBodyHandle>,
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

pub fn init() -> Game {
    Game {
        world: physics_world::init(),
        metadata_by_handle: HashMap::new(),
        handle_by_body_id: HashMap::new(),
    }
}

pub fn step(game_state: &mut Game) -> HashSet<RigidBodyHandle> {
    let collisions = physics_world::step(&mut game_state.world);

    for collision_event in collisions {
        handle_contact(collision_event, game_state);
    }

    let active_handles = physics_world::get_active_handles(&game_state.world);
    let _any_removed = remove_stale_objects(game_state, active_handles);

    physics_world::get_active_handles(&game_state.world)
}

fn remove_stale_objects(game_state: &mut Game, handles: HashSet<RigidBodyHandle>) -> bool {
    let mut any_removed = false;
    for handle in handles {
        if let Some(body) = get_body_from_handle(game_state, &handle) {
            if is_stale(&body) {
                delete_body(body, game_state);
                any_removed = true;
            }
        }
    }

    any_removed
}

fn is_stale(body: &body::Body) -> bool {
    match body.class {
        body::BodyClass::Bullet => is_on_floor(body) || is_at_rest(body),
        _ => false,
    }
}

fn is_at_rest(body::Body { linvel, .. }: &body::Body) -> bool {
    linvel.0.round() == 0.0 && linvel.1.round() == 0.0 && linvel.2.round() == 0.0
}

fn is_on_floor(body: &body::Body) -> bool {
    let translation = body.translation;
    let origin_height = translation.2;
    let object_height = body.dimensions.2;

    origin_height <= (object_height / 2.0)
}

pub fn get_handles(game_state: &Game) -> HashSet<RigidBodyHandle> {
    physics_world::get_bodies(&game_state.world)
        .iter()
        .map(|(handle, _body)| handle)
        .collect()
}

pub fn get_tick_ms(game_state: &Game) -> f32 {
    physics_world::get_dt(&game_state.world) * 1000.0
}

pub fn upsert_body(game_state: &mut Game, body: &body::Body) -> bool {
    // TODO: get rotations working natively so that we don't have to keep them
    // in metadata
    let metadata = BodyMetadata {
        id: body.id.clone(),
        team_id: body.team_id.clone(),
        owner_id: body.owner_id.clone(),
        class: body.class,
        rotation: body.rotation,
        dimensions: body.dimensions,
        hp: body.hp,
    };

    match game_state.handle_by_body_id.get_mut(&body.id) {
        Some(existing_body_handle) => {
            if let Some(existing_body) =
                physics_world::get_body_mut(&mut game_state.world, existing_body_handle)
            {
                existing_body.set_translation(to_vec3(body.translation), true);
                existing_body.set_linvel(to_vec3(body.linvel), true);
                existing_body.set_angvel(to_vec3(body.angvel), true);
                existing_body
                    .set_rotation(Rotation::from_euler_angles(0.0, 0.0, body.rotation.2), true);
            }
            game_state
                .metadata_by_handle
                .insert(*existing_body_handle, metadata);
            false
        }
        None => {
            let body_handle = physics_world::add_body(&mut game_state.world, body);
            game_state.metadata_by_handle.insert(body_handle, metadata);
            game_state
                .handle_by_body_id
                .insert(body.id.clone(), body_handle);
            true
        }
    }
}

pub fn delete_body(body: body::Body, game_state: &mut Game) {
    match game_state.handle_by_body_id.remove(&body.id) {
        Some(handle) => {
            physics_world::remove_body(&mut game_state.world, handle);
            game_state.metadata_by_handle.remove(&handle);
        }
        None => {}
    };
}

pub fn get_handle(body_id: &String, game_state: &Game) -> RigidBodyHandle {
    game_state.handle_by_body_id[body_id]
}

pub fn is_won(game_state: &Game) -> bool {
    get_num_teams_alive(&game_state.metadata_by_handle) == 1
}

pub fn get_body_from_handle(game_state: &Game, handle: &RigidBodyHandle) -> Option<body::Body> {
    match (
        physics_world::get_body(&game_state.world, handle),
        game_state.metadata_by_handle.get(handle),
    ) {
        (Some(rigid_body), Some(metadata)) => Some(rigid_body_to_body(rigid_body, &metadata)),
        _ => None,
    }
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

fn get_num_teams_alive(metadata_by_handle: &HashMap<RigidBodyHandle, BodyMetadata>) -> usize {
    let mut teams_alive = HashSet::new();

    for (_id, body) in metadata_by_handle {
        match (body.hp, &body.team_id) {
            (_, None) => {}
            (0, _) => {}
            (_nonzero_hp, Some(team_id)) => {
                teams_alive.insert(team_id);
            }
        };
    }

    teams_alive.len()
}

fn get_metadata_for_collider_handle(
    collider_handle: ColliderHandle,
    game: &Game,
) -> Option<(RigidBodyHandle, BodyMetadata)> {
    let collider_set = physics_world::get_colliders(&game.world);
    if let Some(body_handle) = collider_set
        .get(collider_handle)
        .map(|c| c.parent().unwrap())
    {
        if let Some(metadata) = game.metadata_by_handle.get(&body_handle) {
            return Some((body_handle, metadata.clone()));
        }
    }

    return None;
}

pub fn handle_contact(collision_event: CollisionEvent, game_state: &mut Game) {
    match collision_event {
        CollisionEvent::Started(collider_handle_a, collider_handle_b, _flags) => {
            let ((body_handle_a, metadata_a), (body_handle_b, metadata_b)) = match (
                get_metadata_for_collider_handle(collider_handle_a, &game_state),
                get_metadata_for_collider_handle(collider_handle_b, &game_state),
            ) {
                (Some(a), Some(b)) => (a, b),
                _ => return,
            };

            let are_objects_on_same_team = metadata_a.team_id == metadata_b.team_id;

            match (metadata_a.class, metadata_b.class) {
                (body::BodyClass::Player, body::BodyClass::Bullet) => {
                    if !are_objects_on_same_team {
                        handle_player_bullet_collision(body_handle_a, body_handle_b, game_state);
                    }
                }
                (body::BodyClass::Bullet, body::BodyClass::Player) => {
                    if !are_objects_on_same_team {
                        handle_player_bullet_collision(body_handle_b, body_handle_a, game_state);
                    }
                }
                _ => {}
            };
        }
        _ => {}
    };
}

fn handle_player_bullet_collision(
    player_handle: RigidBodyHandle,
    bullet_handle: RigidBodyHandle,
    game_state: &mut Game,
) {
    if let Some(body_data) = game_state.metadata_by_handle.get_mut(&player_handle) {
        body_data.hp = (body_data.hp - 1).max(0);
        physics_world::remove_body(&mut game_state.world, bullet_handle);
        game_state.metadata_by_handle.remove(&bullet_handle);
    }
}
