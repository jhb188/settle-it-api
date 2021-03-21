use rapier3d::dynamics::{IntegrationParameters, JointSet, RigidBodySet};
#[macro_use]
extern crate rustler;

use crossbeam;
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
use std::collections::HashMap;
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
    ("apply_jump", 1, apply_jump),
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
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "Elixir.SettleIt.GameServer.Physics.Body")]
struct Body {
    id: String,
    owner_id: Option<String>,
    translation: (f32, f32, f32),
    rotation: (f32, f32, f32),
    linvel: (f32, f32, f32),
    angvel: (f32, f32, f32),
    mass: f32,
    #[serde(rename = "class")]
    class: BodyClass,
    hp: i32,
}

#[derive(Clone)]
struct BodyMetadata {
    id: String,
    owner_id: Option<String>,
    class: BodyClass,
    rotation: (f32, f32, f32),
    hp: i32,
}

struct BodyClassProperties {
    mass: f32,
    height: f32,
}

fn apply_jump<'a>(env: Env<'a>, args: &[Term<'a>]) -> NifResult<Term<'a>> {
    let body: Body = from_term(args[0])?;
    let body_id = body.id.clone();
    let owner_id = body.owner_id.clone();
    let body_rotation = body.rotation.clone();
    let body_hp = body.hp.clone();

    let rigid_body: RigidBody = body_to_rigid_body(body);
    let mut body_set = RigidBodySet::new();

    let rigid_body = if can_player_jump(&rigid_body) {
        let mut collider_set = ColliderSet::new();
        let collider = get_collider_for_body_class(BodyClass::Player);
        let body_handle = body_set.insert(rigid_body);
        collider_set.insert(collider, body_handle, &mut body_set);

        let rigid_body_to_jump = body_set.get_mut(body_handle).unwrap();
        let impulse = Vector3::z() * 1000.0;
        rigid_body_to_jump.apply_impulse(impulse, true);

        rigid_body_to_jump
    } else {
        &rigid_body
    };

    let body = rigid_body_to_body(
        rigid_body,
        BodyMetadata {
            id: body_id,
            owner_id: owner_id,
            class: BodyClass::Player,
            rotation: body_rotation,
            hp: body_hp,
        },
    );
    to_term(env, body).map_err(|err| err.into())
}

fn init_world<'a>(env: Env<'a>, _args: &[Term<'a>]) -> NifResult<Term<'a>> {
    let origin_sphere_id = Uuid::new_v4().to_string();
    let origin_sphere_key = origin_sphere_id.clone();
    let origin_sphere = Body {
        id: origin_sphere_id,
        owner_id: None,
        translation: (0.0, 0.0, 0.5),
        rotation: (0.0, 0.0, 0.0),
        linvel: (0.0, 0.0, 0.0),
        angvel: (0.0, 0.0, 0.0),
        mass: 100.0,
        class: BodyClass::Test,
        hp: 0,
    };

    let initial_bodies: HashMap<String, Body> = vec![(origin_sphere_key, origin_sphere)]
        .into_iter()
        .collect();

    to_term(env, initial_bodies).map_err(|err| err.into())
}

fn step<'a>(env: Env<'a>, args: &[Term<'a>]) -> NifResult<Term<'a>> {
    let bodies: HashMap<String, Body> = from_term(args[0])?;
    let dt: f32 = from_term(args[1])?;
    let bodies: HashMap<String, Body> = step_bodies(bodies, dt);

    to_term(env, bodies).map_err(|err| err.into())
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
            let body_metadata = metadata.clone();
            let body_id = body_metadata.id.clone();
            if is_stale(body, metadata) {
                None
            } else {
                Some((body_id, rigid_body_to_body(body, body_metadata)))
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
                .expect("missing metadata for body handle");
            let metadata_b = metadata_by_handle
                .get(&body_handle_b)
                .expect("missing metadata for body handle");

            let metadata_a_id = metadata_a.id.clone();
            let metadata_b_id = metadata_b.id.clone();

            match (metadata_a.class, metadata_b.class) {
                (BodyClass::Player, BodyClass::Bullet) => {
                    if Some(metadata_a_id) != metadata_b.owner_id {
                        metadata_by_handle.insert(
                            body_handle_a,
                            BodyMetadata {
                                id: metadata_a.id.clone(),
                                owner_id: metadata_a.owner_id.clone(),
                                class: metadata_a.class,
                                rotation: metadata_a.rotation,
                                hp: std::cmp::max(metadata_a.hp - 1, 0),
                            },
                        );
                        body_set.remove(body_handle_b, collider_set, joint_set);
                        metadata_by_handle.remove(&body_handle_b);
                    }
                }
                (BodyClass::Bullet, BodyClass::Player) => {
                    if metadata_a.owner_id != Some(metadata_b_id) {
                        metadata_by_handle.insert(
                            body_handle_b,
                            BodyMetadata {
                                id: metadata_b.id.clone(),
                                owner_id: metadata_b.owner_id.clone(),
                                class: metadata_b.class,
                                rotation: metadata_b.rotation,
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

    for (body_id, body) in input_bodies {
        let owner_id = body.owner_id.clone();
        let body_class = body.class.clone();
        let body_hp = body.hp.clone();
        let body_rotation = body.rotation.clone();
        let rigid_body = body_to_rigid_body(body);
        let collider = get_collider_for_body_class(body_class);
        let body_handle = body_set.insert(rigid_body);
        collider_set.insert(collider, body_handle, &mut body_set);
        metadata_by_handle.insert(
            body_handle,
            BodyMetadata {
                id: body_id,
                owner_id: owner_id,
                class: body_class,
                rotation: body_rotation,
                hp: body_hp,
            },
        );
    }

    (body_set, collider_set)
}

fn body_to_rigid_body(body: Body) -> RigidBody {
    match body.class {
        BodyClass::Player => body_to_dynamic_rigid_body(body),
        BodyClass::Bullet => body_to_dynamic_rigid_body(body),
        BodyClass::Test => body_to_dynamic_rigid_body(body),
    }
}

fn body_to_dynamic_rigid_body(body: Body) -> RigidBody {
    let (transx, transy, transz) = body.translation;
    let (linvelx, linvely, linvelz) = body.linvel;
    let (_rotx, _roty, rotz) = body.rotation;
    let (angvelx, angvely, angvelz) = body.angvel;
    let physics_properties = get_physics_properties_for_class(body.class);

    let base_rigid_body_builder = RigidBodyBuilder::new(BodyStatus::Dynamic)
        .translation(transx, transy, transz)
        .rotation(Vector3::z() * rotz)
        .lock_rotations()
        .linvel(linvelx, linvely, linvelz)
        .angvel(Vector3::new(angvelx, angvely, angvelz))
        .mass(physics_properties.mass);

    let rigid_body_builder = match body.class {
        BodyClass::Player => base_rigid_body_builder.sleeping(body.hp == 0),
        _ => base_rigid_body_builder,
    };

    rigid_body_builder.build()
}

fn body_to_static_rigid_body(body: Body) -> RigidBody {
    let (transx, transy, transz) = body.translation;
    let (_rotx, _roty, rotz) = body.rotation;
    let physics_properties = get_physics_properties_for_class(body.class);

    RigidBodyBuilder::new(BodyStatus::Static)
        .translation(transx, transy, transz)
        .rotation(Vector3::z() * rotz)
        .lock_rotations()
        .mass(physics_properties.mass)
        .build()
}

fn get_physics_properties_for_class(body_class: BodyClass) -> BodyClassProperties {
    match body_class {
        BodyClass::Player => BodyClassProperties {
            mass: 100.0,
            height: 2.0,
        },
        BodyClass::Bullet => BodyClassProperties {
            mass: 0.05,
            height: 0.10,
        },
        BodyClass::Test => BodyClassProperties {
            mass: 100.0,
            height: 1.0,
        },
    }
}

fn is_stale(body: &rapier3d::dynamics::RigidBody, metadata: BodyMetadata) -> bool {
    let class = metadata.class;

    match class {
        BodyClass::Bullet => is_on_floor(body, class) || is_at_rest(body),
        _ => false,
    }
}

fn is_at_rest(body: &rapier3d::dynamics::RigidBody) -> bool {
    let linvel = body.linvel();

    linvel.x.round() == 0.0 && linvel.y.round() == 0.0 && linvel.z.round() == 0.0
}

fn rigid_body_to_body(body: &rapier3d::dynamics::RigidBody, metadata: BodyMetadata) -> Body {
    let orientation = body.position();
    let translation = orientation.translation;
    let rotation = metadata.rotation;
    let linvel = body.linvel();
    let angvel = body.angvel();
    let body_class = metadata.class;
    let body_class_properties = get_physics_properties_for_class(body_class);

    // clamp to floor
    let (z_translation, z_vel) =
        if body.is_dynamic() && is_on_floor(body, body_class) && is_body_falling(body) {
            (body_class_properties.height / 2.0, 0.0)
        } else {
            (translation.z, linvel.z)
        };

    Body {
        id: metadata.id,
        owner_id: metadata.owner_id,
        translation: (translation.x, translation.y, z_translation),
        rotation: (rotation.0, rotation.1, rotation.2),
        linvel: (linvel.x, linvel.y, z_vel),
        angvel: (angvel.x, angvel.y, angvel.z),
        mass: body.mass(),
        class: metadata.class,
        hp: metadata.hp,
    }
}

fn is_on_floor(body: &rapier3d::dynamics::RigidBody, body_class: BodyClass) -> bool {
    let orientation = body.position();
    let translation = orientation.translation;
    let origin_height = translation.z;
    let object_height = get_physics_properties_for_class(body_class).height;

    origin_height <= (object_height / 2.0)
}

fn is_body_falling(body: &rapier3d::dynamics::RigidBody) -> bool {
    let linvel = body.linvel();

    linvel.z <= 0.0
}

fn can_player_jump(body: &rapier3d::dynamics::RigidBody) -> bool {
    is_on_floor(body, BodyClass::Player) && is_body_falling(body)
}

fn get_collider_for_body_class(body_class: BodyClass) -> Collider {
    let body_class_properties = get_physics_properties_for_class(body_class);
    let height = body_class_properties.height;

    match body_class {
        BodyClass::Player => ColliderBuilder::new(SharedShape::cylinder(height, 0.50)).build(),
        BodyClass::Bullet => ColliderBuilder::new(SharedShape::ball(height / 2.0)).build(),
        BodyClass::Test => ColliderBuilder::new(SharedShape::ball(height / 2.0)).build(),
    }
}
