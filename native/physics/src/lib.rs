use rapier3d::dynamics::{IntegrationParameters, JointSet, RigidBodySet};
#[macro_use]
extern crate rustler;

use rapier3d::dynamics::{BodyStatus, RigidBody, RigidBodyBuilder, RigidBodyHandle};
use rapier3d::geometry::{
    BroadPhase, Collider, ColliderBuilder, ColliderSet, NarrowPhase, SharedShape,
};
use rapier3d::na::Vector3;
use rapier3d::pipeline::PhysicsPipeline;
use rustler::{Env, NifResult, Term};
use serde;
use serde::{Deserialize, Serialize};
use serde_rustler::{from_term, to_term};
use std::collections::HashMap;

mod atoms {
    rustler_atoms! {
        atom ok;
        atom error;
    }
}

rustler_export_nifs! {
    "Elixir.SettleIt.GameServer.Physics",
    [("apply_jump", 1, apply_jump),
    ("step", 2, step)],
    None
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "Elixir.SettleIt.GameServer.Physics.Body")]
struct Body {
    id: Option<String>,
    translation: (f32, f32, f32),
    rotation: (f32, f32, f32),
    linvel: (f32, f32, f32),
    angvel: (f32, f32, f32),
    mass: f32,
}

const PLAYER_HEIGHT: f32 = 1.0;
const PLAYER_RADIUS: f32 = 0.20;

fn apply_jump<'a>(env: Env<'a>, args: &[Term<'a>]) -> NifResult<Term<'a>> {
    let body: Body = from_term(args[0])?;
    let body_id = body.id.clone();

    let rigid_body: RigidBody = body_to_rigid_body(body);
    let mut body_set = RigidBodySet::new();

    let rigid_body = if can_player_jump(&rigid_body) {
        let mut collider_set = ColliderSet::new();
        let collider = get_player_collider();
        let body_handle = body_set.insert(rigid_body);
        collider_set.insert(collider, body_handle, &mut body_set);

        let rigid_body_to_jump = body_set.get_mut(body_handle).unwrap();
        let impulse = Vector3::z() * 1000.0;
        rigid_body_to_jump.apply_impulse(impulse, true);

        rigid_body_to_jump
    } else {
        &rigid_body
    };

    let body = rigid_body_to_body(rigid_body, body_id);
    to_term(env, body).map_err(|err| err.into())
}

fn step<'a>(env: Env<'a>, args: &[Term<'a>]) -> NifResult<Term<'a>> {
    let bodies: Vec<Body> = from_term(args[0])?;
    let dt: f32 = from_term(args[1])?;
    let bodies = step_bodies(bodies, dt);

    to_term(env, bodies).map_err(|err| err.into())
}

fn step_bodies(input_bodies: Vec<Body>, dt: f32) -> Vec<Body> {
    let mut pipeline = PhysicsPipeline::new();
    let gravity = Vector3::new(0.0, 0.0, -9.80665);
    let mut integration_parameters = IntegrationParameters::default();
    integration_parameters.dt = dt;
    let mut broad_phase = BroadPhase::new();
    let mut narrow_phase = NarrowPhase::new();
    let mut body_ids: HashMap<RigidBodyHandle, String> = HashMap::new();
    let mut joints = JointSet::new();

    // TODO: implement physics and contact hooks
    let physics_hooks = ();
    let event_handler = ();

    let (mut body_set, mut collider_set) = get_body_sets(input_bodies, &mut body_ids);

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

    body_set
        .iter()
        .map(|(handle, b)| rigid_body_to_body(b, get_body_id(&handle, &body_ids)))
        .collect()
}

fn get_body_id(
    handle: &RigidBodyHandle,
    body_ids: &HashMap<RigidBodyHandle, String>,
) -> Option<String> {
    match body_ids.get(handle) {
        Some(body_id) => Some(body_id.to_owned()),
        None => None,
    }
}

fn init_world(_body_set: &mut RigidBodySet, _collider_set: &mut ColliderSet) {
    /*
    using an actual floor body to avoid falling through the ground is way slower than just clamping z-linvel
    and z-translation at 0 if we fall through. it was 20x slower with a 250 x 250 x 1 floor.
    */
    // add_floor(body_set, collider_set);
}

// fn add_floor(body_set: &mut RigidBodySet, collider_set: &mut ColliderSet) {
//     let floor_size = 250.0;
//     let floor_height = 1.0;
//     let floor_body = RigidBodyBuilder::new_static()
//         .translation(0.0, 0.0, -floor_height)
//         .lock_rotations()
//         .lock_translations()
//         .build();
//     let floor_handle = body_set.insert(floor_body);
//     let floor_collider = ColliderBuilder::cuboid(floor_size, floor_size, floor_height).build();
//     collider_set.insert(floor_collider, floor_handle, body_set);
// }

fn get_body_sets(
    input_bodies: Vec<Body>,
    body_ids: &mut HashMap<RigidBodyHandle, String>,
) -> (RigidBodySet, ColliderSet) {
    let mut body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();

    // let world_is_uninitialized = input_bodies
    //     .iter()
    //     .filter(|input_body| input_body.id.is_none())
    //     .peekable()
    //     .peek()
    //     .is_none();

    // if world_is_uninitialized {
    init_world(&mut body_set, &mut collider_set);
    // }

    let dynamic_bodies: Vec<Body> = input_bodies
        .into_iter()
        .filter(|input_body| input_body.id.is_some())
        .collect();

    for body in dynamic_bodies {
        let maybe_body_id = body.id.clone();
        let rigid_body = body_to_rigid_body(body);

        let collider = get_player_collider();
        let body_handle = body_set.insert(rigid_body);
        collider_set.insert(collider, body_handle, &mut body_set);
        match maybe_body_id {
            Some(body_id) => {
                body_ids.insert(body_handle, body_id);
            }
            None => {}
        }
    }

    (body_set, collider_set)
}

fn body_to_rigid_body(body: Body) -> RigidBody {
    let (transx, transy, transz) = body.translation;
    let (linvelx, linvely, linvelz) = body.linvel;
    let (_rotx, _roty, rotz) = body.rotation;
    let (angvelx, angvely, angvelz) = body.angvel;

    RigidBodyBuilder::new(BodyStatus::Dynamic)
        .translation(transx, transy, transz)
        .rotation(Vector3::z() * rotz)
        .linvel(linvelx, linvely, linvelz)
        .angvel(Vector3::new(angvelx, angvely, angvelz))
        .mass(body.mass)
        .build()
}

fn rigid_body_to_body(body: &rapier3d::dynamics::RigidBody, id: Option<String>) -> Body {
    let orientation = body.position();
    let translation = orientation.translation;
    let rotation = orientation.rotation.vector();
    let linvel = body.linvel();
    let angvel = body.angvel();

    // clamp player to floor
    let (z_translation, z_vel) =
        if body.is_dynamic() && is_player_on_floor(body) && is_body_falling(body) {
            (PLAYER_HEIGHT / 2.0, 0.0)
        } else {
            (translation.z, linvel.z)
        };

    Body {
        id: id,
        translation: (translation.x, translation.y, z_translation),
        rotation: (rotation.x, rotation.y, rotation.z),
        linvel: (linvel.x, linvel.y, z_vel),
        angvel: (angvel.x, angvel.y, angvel.z),
        mass: 100.0,
    }
}

fn is_player_on_floor(body: &rapier3d::dynamics::RigidBody) -> bool {
    let orientation = body.position();
    let translation = orientation.translation;
    let origin_height = translation.z;

    origin_height <= PLAYER_HEIGHT / 2.0
}

fn is_body_falling(body: &rapier3d::dynamics::RigidBody) -> bool {
    let linvel = body.linvel();

    linvel.z <= 0.0
}

fn can_player_jump(body: &rapier3d::dynamics::RigidBody) -> bool {
    is_player_on_floor(body) && is_body_falling(body)
}

fn get_player_collider() -> Collider {
    ColliderBuilder::new(SharedShape::cylinder(PLAYER_HEIGHT, PLAYER_RADIUS)).build()
}
