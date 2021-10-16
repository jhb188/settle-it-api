use rapier3d::dynamics::{IntegrationParameters, JointSet, RigidBodySet};
#[macro_use]
extern crate rustler;

use crossbeam;
use rand::Rng;
use rapier3d::na::Vector3;
use rapier3d::prelude::*;
use serde;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

mod atoms {
    rustler_atoms! {
        atom ok;
        atom error;
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
enum BodyClass {
    #[serde(rename = "player")]
    Player,
    #[serde(rename = "bullet")]
    Bullet,
    #[serde(rename = "obstacle")]
    Obstacle,
    #[serde(rename = "test")]
    Test,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "Elixir.SettleIt.GameServer.State.Body")]
struct Body {
    id: String,
    team_id: Option<String>,
    owner_id: Option<String>,
    translation: (f32, f32, f32),
    rotation: (f32, f32, f32),
    linvel: (f32, f32, f32),
    angvel: (f32, f32, f32),
    dimensions: (f32, f32, f32),
    mass: f32,
    #[serde(rename = "class")]
    class: BodyClass,
    hp: i32,
}

#[derive(Clone, Debug)]
struct BodyMetadata {
    id: String,
    team_id: Option<String>,
    owner_id: Option<String>,
    class: BodyClass,
    rotation: (f32, f32, f32),
    dimensions: (f32, f32, f32),
    hp: i32,
}

const ARENA_WIDTH: f32 = 200.0;

fn get_init_world() -> HashMap<String, Body> {
    let mut initial_bodies: HashMap<String, Body> = HashMap::new();

    let floor = Body {
        id: String::from("floor"),
        team_id: None,
        owner_id: None,
        translation: (0.0, 0.0, -0.5),
        rotation: (0.0, 0.0, 0.0),
        linvel: (0.0, 0.0, 0.0),
        angvel: (0.0, 0.0, 0.0),
        dimensions: (ARENA_WIDTH + 0.1, ARENA_WIDTH + 0.1, 1.0),
        mass: 0.0,
        class: BodyClass::Obstacle,
        hp: 0,
    };

    initial_bodies.insert(String::from("floor"), floor);

    seed_obstacles(&mut initial_bodies);

    initial_bodies
}

fn handle_contact(
    contact_event: ContactEvent,
    islands: &mut IslandManager,
    body_set: &mut RigidBodySet,
    collider_set: &mut ColliderSet,
    joint_set: &mut JointSet,
    metadata_by_handle: &mut HashMap<RigidBodyHandle, BodyMetadata>,
) {
    match contact_event {
        ContactEvent::Started(collider_handle_a, collider_handle_b) => {
            let body_handle_a = collider_set
                .get(collider_handle_a)
                .map(|c| c.parent().unwrap())
                .expect("missing body handle for collider");
            let body_handle_b = collider_set
                .get(collider_handle_b)
                .map(|c| c.parent().unwrap())
                .expect("missing body handle for collider");

            let metadata_a = metadata_by_handle
                .get(&body_handle_a)
                .expect("missing metadata for body handle")
                .clone();
            let metadata_b = metadata_by_handle
                .get(&body_handle_b)
                .expect("missing metadata for body handle")
                .clone();

            let metadata_a_team_id = metadata_a.team_id.clone();
            let metadata_b_team_id = metadata_b.team_id.clone();

            match (metadata_a.class, metadata_b.class) {
                (BodyClass::Player, BodyClass::Bullet) => {
                    if metadata_a_team_id != metadata_b.team_id {
                        metadata_by_handle.insert(
                            body_handle_a,
                            BodyMetadata {
                                id: metadata_a.id.clone(),
                                team_id: metadata_a.team_id.clone(),
                                owner_id: metadata_a.owner_id.clone(),
                                class: metadata_a.class,
                                rotation: metadata_a.rotation,
                                dimensions: metadata_a.dimensions,
                                hp: std::cmp::max(metadata_a.hp - 1, 0),
                            },
                        );
                        body_set.remove(body_handle_b, islands, collider_set, joint_set);
                        metadata_by_handle.remove(&body_handle_b);
                    }
                }
                (BodyClass::Bullet, BodyClass::Player) => {
                    if metadata_a.team_id != metadata_b_team_id {
                        metadata_by_handle.insert(
                            body_handle_b,
                            BodyMetadata {
                                id: metadata_b.id.clone(),
                                team_id: metadata_b.team_id.clone(),
                                owner_id: metadata_b.owner_id.clone(),
                                class: metadata_b.class,
                                rotation: metadata_b.rotation,
                                dimensions: metadata_b.dimensions,
                                hp: std::cmp::max(metadata_b.hp - 1, 0),
                            },
                        );
                        body_set.remove(body_handle_a, islands, collider_set, joint_set);
                        metadata_by_handle.remove(&body_handle_a);
                    }
                }
                _ => {}
            };
        }
        _ => {}
    };
}

fn get_body_metadata(
    handle: &RigidBodyHandle,
    metadata_by_handle: &mut HashMap<RigidBodyHandle, BodyMetadata>,
) -> BodyMetadata {
    let value = metadata_by_handle
        .remove(handle)
        .expect("no metadata for body handle");

    metadata_by_handle.insert(*handle, value.clone());

    value
}

fn add_body(
    body_set: &mut RigidBodySet,
    collider_set: &mut ColliderSet,
    metadata_by_handle: &mut HashMap<RigidBodyHandle, BodyMetadata>,
    handle_by_body_id: &mut HashMap<String, RigidBodyHandle>,
    body: &Body,
) -> bool {
    match handle_by_body_id.get_mut(&body.id) {
        Some(existing_body_handle) => {
            let (transx, transy, transz) = body.translation;
            let (linvelx, linvely, linvelz) = body.linvel;
            let (angvelx, angvely, angvelz) = body.angvel;
            let (_rotx, _roty, rotz) = body.rotation;
            let existing_body = body_set.get_mut(*existing_body_handle).unwrap();
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
            let rigid_body = body_to_rigid_body(body);
            let collider = get_collider_for_body(body);
            let body_handle = body_set.insert(rigid_body);
            collider_set.insert_with_parent(collider, body_handle, body_set);
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
    body_set: &mut RigidBodySet,
    islands: &mut IslandManager,
    collider_set: &mut ColliderSet,
    joint_set: &mut JointSet,
    metadata_by_handle: &mut HashMap<RigidBodyHandle, BodyMetadata>,
    handle_by_body_id: &mut HashMap<String, RigidBodyHandle>,
    body: Body,
) {
    match handle_by_body_id.remove(&body.id) {
        Some(handle) => {
            body_set.remove(handle, islands, collider_set, joint_set);
            metadata_by_handle.remove(&handle);
        }
        None => {}
    };
}

fn get_body_sets(
    input_bodies: HashMap<String, Body>,
    metadata_by_handle: &mut HashMap<RigidBodyHandle, BodyMetadata>,
    handle_by_body_id: &mut HashMap<String, RigidBodyHandle>,
) -> (RigidBodySet, ColliderSet) {
    let mut body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();

    for (_body_id, body) in &input_bodies {
        add_body(
            &mut body_set,
            &mut collider_set,
            metadata_by_handle,
            handle_by_body_id,
            body,
        );
    }

    (body_set, collider_set)
}

fn body_to_rigid_body(body: &Body) -> RigidBody {
    match body.class {
        BodyClass::Player => body_to_dynamic_rigid_body(body),
        BodyClass::Bullet => body_to_dynamic_rigid_body(body),
        BodyClass::Test => body_to_dynamic_rigid_body(body),
        BodyClass::Obstacle => body_to_static_rigid_body(body),
    }
}

fn body_to_dynamic_rigid_body(body: &Body) -> RigidBody {
    let (transx, transy, transz) = body.translation;
    let (rotx, roty, rotz) = body.rotation;
    let (linvelx, linvely, linvelz) = body.linvel;
    let (angvelx, angvely, angvelz) = body.angvel;

    let base_rigid_body_builder = RigidBodyBuilder::new(RigidBodyType::Dynamic)
        .translation(Vector3::new(transx, transy, transz))
        .rotation(Vector3::new(rotx, roty, rotz))
        .lock_rotations()
        .linvel(Vector3::new(linvelx, linvely, linvelz))
        .angvel(Vector3::new(angvelx, angvely, angvelz))
        .additional_mass(body.mass);

    let rigid_body_builder = match body.class {
        BodyClass::Player => base_rigid_body_builder.sleeping(body.hp == 0),
        _ => base_rigid_body_builder,
    };

    rigid_body_builder.build()
}

fn body_to_static_rigid_body(body: &Body) -> RigidBody {
    let (transx, transy, transz) = body.translation;
    let (_rotx, _roty, rotz) = body.rotation;

    RigidBodyBuilder::new(RigidBodyType::Static)
        .translation(Vector3::new(transx, transy, transz))
        .rotation(Vector3::z() * rotz)
        .lock_rotations()
        .additional_mass(body.mass)
        .build()
}

fn is_stale(body: &rapier3d::dynamics::RigidBody, metadata: &BodyMetadata) -> bool {
    match metadata.class {
        BodyClass::Bullet => is_on_floor(body, metadata) || is_at_rest(body),
        _ => false,
    }
}

fn is_at_rest(body: &rapier3d::dynamics::RigidBody) -> bool {
    let linvel = body.linvel();

    linvel.x.round() == 0.0 && linvel.y.round() == 0.0 && linvel.z.round() == 0.0
}

fn rigid_body_to_body(body: &rapier3d::dynamics::RigidBody, metadata: &BodyMetadata) -> Body {
    let translation = body.translation();
    let linvel = body.linvel();
    let angvel = body.angvel();

    Body {
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

fn get_collider_for_body(body: &Body) -> Collider {
    let half_height = body.dimensions.2 / 2.0;
    match body.class {
        BodyClass::Player => ColliderBuilder::capsule_z(half_height - 0.525, 0.525)
            .active_events(ActiveEvents::CONTACT_EVENTS)
            .build(),
        BodyClass::Bullet => ColliderBuilder::new(SharedShape::ball(half_height))
            .active_events(ActiveEvents::CONTACT_EVENTS)
            .build(),
        BodyClass::Test => ColliderBuilder::new(SharedShape::ball(half_height)).build(),
        BodyClass::Obstacle => ColliderBuilder::new(SharedShape::cuboid(
            body.dimensions.0 / 2.0,
            body.dimensions.1 / 2.0,
            body.dimensions.2 / 2.0,
        ))
        .build(),
    }
}

fn body_overlaps_existing_bodies(body: &Body, bodies: &HashMap<String, Body>) -> bool {
    let mut overlaps = false;
    for (_k, current_body) in bodies {
        if bounding_cubes_collide(body, current_body) {
            overlaps = true;
            break;
        }
    }

    overlaps
}

fn bounding_cubes_collide(body_a: &Body, body_b: &Body) -> bool {
    let body_a_x_size = body_a.dimensions.0;
    let body_a_x_min = body_a.translation.0;
    let body_a_x_max = body_a_x_min + body_a_x_size;
    let body_b_x_size = body_b.dimensions.0;
    let body_b_x_min = body_b.translation.0;
    let body_b_x_max = body_b_x_min + body_b_x_size;

    let body_a_y_size = body_a.dimensions.1;
    let body_a_y_min = body_a.translation.1;
    let body_a_y_max = body_a_y_min + body_a_y_size;
    let body_b_y_size = body_b.dimensions.1;
    let body_b_y_min = body_b.translation.1;
    let body_b_y_max = body_b_y_min + body_b_y_size;

    let body_a_z_size = body_a.dimensions.2;
    let body_a_z_min = body_a.translation.2;
    let body_a_z_max = body_a_z_min + body_a_z_size;
    let body_b_z_size = body_b.dimensions.2;
    let body_b_z_min = body_b.translation.2;
    let body_b_z_max = body_b_z_min + body_b_z_size;

    (body_a_x_min <= body_b_x_max && body_a_x_max >= body_b_x_min)
        && (body_a_y_min <= body_b_y_max && body_a_y_max >= body_b_y_min)
        && (body_a_z_min <= body_b_z_max && body_a_z_max >= body_b_z_min)
}

fn get_random_obstacle() -> Body {
    let mut rng = rand::thread_rng();
    let margin = 25.0;
    let position_max = (ARENA_WIDTH / 2.0) - margin;
    let position_min = -position_max;
    let position_x = rng.gen_range(position_min..position_max);
    let position_y = rng.gen_range(position_min..position_max);
    let length = rng.gen_range(1.0..5.0);
    let width = rng.gen_range(1.0..5.0);
    let height = rng.gen_range(0.2..5.0);
    Body {
        id: Uuid::new_v4().to_string(),
        team_id: None,
        owner_id: None,
        translation: (position_x, position_y, height / 2.0),
        rotation: (0.0, 0.0, 0.0),
        linvel: (0.0, 0.0, 0.0),
        angvel: (0.0, 0.0, 0.0),
        dimensions: (length, width, height),
        mass: 100.0,
        class: BodyClass::Obstacle,
        hp: 0,
    }
}

fn seed_obstacle_in_open_space(bodies: &mut HashMap<String, Body>) {
    let obstacle = get_random_obstacle();

    if !body_overlaps_existing_bodies(&obstacle, bodies) {
        bodies.insert(obstacle.id.clone(), obstacle);
    } else {
        seed_obstacle_in_open_space(bodies);
    }
}

fn seed_obstacles(bodies: &mut HashMap<String, Body>) {
    for _ in 0..50 {
        seed_obstacle_in_open_space(bodies);
    }
}

fn spawn_stdin_channel() -> Receiver<HashMap<String, Body>> {
    let (tx, rx) = mpsc::channel::<HashMap<String, Body>>();
    let reader = std::io::stdin();
    thread::spawn(move || loop {
        let mut buf = String::new();
        reader.read_line(&mut buf);
        let result: serde_json::Result<HashMap<String, Body>> = serde_json::from_str(&buf);
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
    let initial_world: HashMap<String, Body> = get_init_world();

    let mut pipeline = PhysicsPipeline::new();
    let mut island_manager = IslandManager::new();
    let gravity = Vector3::new(0.0, 0.0, -9.80665);
    let integration_parameters = IntegrationParameters::default();
    let mut broad_phase = BroadPhase::new();
    let mut narrow_phase = NarrowPhase::new();
    let mut metadata_by_handle: HashMap<RigidBodyHandle, BodyMetadata> = HashMap::new();
    let mut handle_by_body_id: HashMap<String, RigidBodyHandle> = HashMap::new();
    let mut joints = JointSet::new();
    let mut ccd_solver = CCDSolver::new();
    let physics_hooks = ();
    let (contact_send, contact_recv) = crossbeam::channel::unbounded();
    let (intersection_send, _intersection_recv) = crossbeam::channel::unbounded();
    let event_handler = ChannelEventCollector::new(intersection_send, contact_send);

    let (mut body_set, mut collider_set) = get_body_sets(
        initial_world,
        &mut metadata_by_handle,
        &mut handle_by_body_id,
    );

    let mut updated_handles: HashSet<RigidBodyHandle> = HashSet::new();

    let initial_world_handles: HashSet<RigidBodyHandle> =
        body_set.iter().map(|(handle, _body)| handle).collect();

    updated_handles.extend(initial_world_handles);

    let stdin_channel = spawn_stdin_channel();

    let mut is_won: bool = false;

    let integration_dt_ms = integration_parameters.dt * 1000.0;

    while !is_won {
        let mut user_updated_handles = HashSet::new();

        for updated_bodies in stdin_channel.try_iter() {
            for (body_id, body) in &updated_bodies {
                let is_new = add_body(
                    &mut body_set,
                    &mut collider_set,
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

        pipeline.step(
            &gravity,
            &integration_parameters,
            &mut island_manager,
            &mut broad_phase,
            &mut narrow_phase,
            &mut body_set,
            &mut collider_set,
            &mut joints,
            &mut ccd_solver,
            &physics_hooks,
            &event_handler,
        );

        while let Ok(contact_event) = contact_recv.try_recv() {
            handle_contact(
                contact_event,
                &mut island_manager,
                &mut body_set,
                &mut collider_set,
                &mut joints,
                &mut metadata_by_handle,
            );
        }

        let physics_updated_handles: HashSet<RigidBodyHandle> = island_manager
            .active_dynamic_bodies()
            .into_iter()
            .map(|handle_ref| handle_ref.clone())
            .collect();

        updated_handles.extend(physics_updated_handles);

        let next_bodies: HashMap<String, Body> = updated_handles
            .iter()
            .filter_map(|handle| match body_set.get(*handle) {
                Some(rigid_body) => {
                    let metadata = get_body_metadata(handle, &mut metadata_by_handle);
                    let body_id = metadata.id.clone();
                    let body = rigid_body_to_body(rigid_body, &metadata);

                    if is_stale(rigid_body, &metadata) {
                        delete_body(
                            &mut body_set,
                            &mut island_manager,
                            &mut collider_set,
                            &mut joints,
                            &mut metadata_by_handle,
                            &mut handle_by_body_id,
                            body,
                        );
                        None
                    } else {
                        Some((body_id, body))
                    }
                }
                None => None,
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
