use rapier3d::dynamics::{IntegrationParameters, JointSet, RigidBodySet};
#[macro_use]
extern crate rustler;

use crossbeam;
use rand::Rng;
use rapier3d::dynamics::{BodyStatus, RigidBody, RigidBodyBuilder, RigidBodyHandle};
use rapier3d::geometry::{
    BroadPhase, Collider, ColliderBuilder, ColliderSet, ContactEvent, NarrowPhase, SharedShape,
};
use rapier3d::na::Vector3;
use rapier3d::pipeline::{ChannelEventCollector, PhysicsPipeline};
use rustler::{Env, NifResult, Term};
use serde;
use serde::{Deserialize, Serialize};
use serde_rustler::{from_term, to_term};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

mod atoms {
    rustler_atoms! {
        atom ok;
        atom error;
    }
}

rustler_export_nifs! {
    "Elixir.SettleIt.GameServer.Physics",
    [
    ("step", 2, step),
    ("init_world", 0, init_world)
    ],
    None
}

#[derive(Serialize, Deserialize, Copy, Clone)]
#[serde(rename = "class")]
enum BodyClass {
    #[serde(rename = "player")]
    Player,
    #[serde(rename = "bullet")]
    Bullet,
    #[serde(rename = "test")]
    Test,
    #[serde(rename = "obstacle")]
    Obstacle,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Clone)]
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

fn init_world<'a>(env: Env<'a>, _args: &[Term<'a>]) -> NifResult<Term<'a>> {
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

    to_term(env, initial_bodies).map_err(|err| err.into())
}

fn step<'a>(env: Env<'a>, args: &[Term<'a>]) -> NifResult<Term<'a>> {
    let bodies: HashMap<String, Body> = from_term(args[0])?;
    let dt: f32 = from_term(args[1])?;
    let bodies: HashMap<String, Body> = step_bodies(bodies, dt);

    let mut teams_alive = HashSet::new();

    for (_id, body) in &bodies {
        match (body.hp, &body.team_id) {
            (0, _) => {}
            (_nonzero_hp, team_id) => {
                teams_alive.insert(team_id);
            }
        };
    }

    let is_won = teams_alive.len() < 2;

    to_term(env, (bodies, is_won)).map_err(|err| err.into())
}

fn step_bodies(input_bodies: HashMap<String, Body>, dt: f32) -> HashMap<String, Body> {
    let mut pipeline = PhysicsPipeline::new();
    let gravity = Vector3::new(0.0, 0.0, -9.80665);
    let mut integration_parameters = IntegrationParameters::default();
    integration_parameters.dt = dt;
    let mut broad_phase = BroadPhase::new();
    let mut narrow_phase = NarrowPhase::new();
    let mut metadata_by_handle: HashMap<RigidBodyHandle, BodyMetadata> = HashMap::new();
    let mut joints = JointSet::new();
    let physics_hooks = ();
    let (contact_send, contact_recv) = crossbeam::channel::unbounded();
    let (intersection_send, _intersection_recv) = crossbeam::channel::unbounded();
    let event_handler = ChannelEventCollector::new(intersection_send, contact_send);

    let (mut body_set, mut collider_set) = get_body_sets(input_bodies, &mut metadata_by_handle);

    pipeline.step(
        &gravity,
        &integration_parameters,
        &mut broad_phase,
        &mut narrow_phase,
        &mut body_set,
        &mut collider_set,
        &mut joints,
        &physics_hooks,
        &event_handler,
    );

    while let Ok(contact_event) = contact_recv.try_recv() {
        handle_contact(
            contact_event,
            &mut body_set,
            &mut collider_set,
            &mut joints,
            &mut metadata_by_handle,
        );
    }

    body_set
        .iter()
        .filter_map(|(handle, body)| {
            let metadata = pop_body_metadata(&handle, &mut metadata_by_handle);
            let body_id = metadata.id.clone();
            if is_stale(body, &metadata) {
                None
            } else {
                Some((body_id, rigid_body_to_body(body, &metadata)))
            }
        })
        .collect()
}

fn handle_contact(
    contact_event: ContactEvent,
    body_set: &mut RigidBodySet,
    collider_set: &mut ColliderSet,
    joint_set: &mut JointSet,
    metadata_by_handle: &mut HashMap<RigidBodyHandle, BodyMetadata>,
) {
    match contact_event {
        ContactEvent::Started(collider_handle_a, collider_handle_b) => {
            let body_handle_a = collider_set
                .get(collider_handle_a)
                .map(|c| c.parent())
                .expect("missing body handle for collider");
            let body_handle_b = collider_set
                .get(collider_handle_b)
                .map(|c| c.parent())
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
                        body_set.remove(body_handle_b, collider_set, joint_set);
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
                        body_set.remove(body_handle_a, collider_set, joint_set);
                        metadata_by_handle.remove(&body_handle_a);
                    }
                }
                _ => {}
            };
        }
        _ => {}
    };
}

fn pop_body_metadata(
    handle: &RigidBodyHandle,
    metadata_by_handle: &mut HashMap<RigidBodyHandle, BodyMetadata>,
) -> BodyMetadata {
    metadata_by_handle
        .remove(handle)
        .expect("no metadata for body handle")
}

fn get_body_sets(
    input_bodies: HashMap<String, Body>,
    metadata_by_handle: &mut HashMap<RigidBodyHandle, BodyMetadata>,
) -> (RigidBodySet, ColliderSet) {
    let mut body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();

    for (_body_id, body) in &input_bodies {
        let rigid_body = body_to_rigid_body(body);
        let collider = get_collider_for_body(body);
        let body_handle = body_set.insert(rigid_body);
        collider_set.insert(collider, body_handle, &mut body_set);
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
    let (linvelx, linvely, linvelz) = body.linvel;
    let (angvelx, angvely, angvelz) = body.angvel;

    let base_rigid_body_builder = RigidBodyBuilder::new(BodyStatus::Dynamic)
        .translation(transx, transy, transz)
        .lock_rotations()
        .linvel(linvelx, linvely, linvelz)
        .angvel(Vector3::new(angvelx, angvely, angvelz))
        .mass(body.mass);

    let rigid_body_builder = match body.class {
        BodyClass::Player => base_rigid_body_builder.sleeping(body.hp == 0),
        _ => base_rigid_body_builder,
    };

    rigid_body_builder.build()
}

fn body_to_static_rigid_body(body: &Body) -> RigidBody {
    let (transx, transy, transz) = body.translation;
    let (_rotx, _roty, rotz) = body.rotation;

    RigidBodyBuilder::new(BodyStatus::Static)
        .translation(transx, transy, transz)
        .rotation(Vector3::z() * rotz)
        .lock_rotations()
        .mass(body.mass)
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
    let orientation = body.position();
    let translation = orientation.translation;
    let rotation = metadata.rotation;
    let linvel = body.linvel();
    let angvel = body.angvel();

    // clamp to floor
    let (z_translation, z_vel) =
        if body.is_dynamic() && is_on_floor(body, metadata) && is_falling(body) {
            (metadata.dimensions.2 / 2.0, 0.0)
        } else {
            (translation.z, linvel.z)
        };

    Body {
        id: metadata.id.clone(),
        team_id: metadata.team_id.clone(),
        owner_id: metadata.owner_id.clone(),
        translation: (translation.x, translation.y, z_translation),
        rotation: (rotation.0, rotation.1, rotation.2),
        linvel: (linvel.x, linvel.y, z_vel),
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

fn is_falling(body: &rapier3d::dynamics::RigidBody) -> bool {
    let linvel = body.linvel();

    linvel.z <= 0.0
}

fn get_collider_for_body(body: &Body) -> Collider {
    let height = body.dimensions.2;
    match body.class {
        BodyClass::Player => ColliderBuilder::new(SharedShape::cylinder(height, 0.525))
            .density(0.0)
            .build(),
        BodyClass::Bullet => ColliderBuilder::new(SharedShape::ball(height / 2.0))
            .density(0.0)
            .build(),
        BodyClass::Test => ColliderBuilder::new(SharedShape::ball(height / 2.0))
            .density(0.0)
            .build(),
        BodyClass::Obstacle => ColliderBuilder::new(SharedShape::cuboid(
            body.dimensions.0 / 2.0,
            body.dimensions.1 / 2.0,
            body.dimensions.2 / 2.0,
        ))
        .density(0.0)
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
