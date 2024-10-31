use rapier3d::na::Vector3;
use rapier3d::prelude::*;
use serde;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "Elixir.SettleIt.GameServer.State.Body")]
pub struct Body {
    pub id: String,
    pub team_id: Option<String>,
    pub owner_id: Option<String>,
    pub translation: (f32, f32, f32),
    pub rotation: (f32, f32, f32),
    pub linvel: (f32, f32, f32),
    pub angvel: (f32, f32, f32),
    pub dimensions: (f32, f32, f32),
    pub mass: f32,
    #[serde(rename = "class")]
    pub class: BodyClass,
    pub hp: i32,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum BodyClass {
    #[serde(rename = "player")]
    Player,
    #[serde(rename = "bullet")]
    Bullet,
    #[serde(rename = "obstacle")]
    Obstacle,
    #[serde(rename = "test")]
    Test,
}

pub fn overlaps_existing_bodies(body: &Body, bodies: &HashMap<String, Body>) -> bool {
    let mut overlaps = false;
    for (_k, current_body) in bodies {
        if bounding_boxes_overlap(body, current_body) {
            overlaps = true;
            break;
        }
    }

    overlaps
}

fn bounding_boxes_overlap(a: &Body, b: &Body) -> bool {
    let (ax_min, ax_max) = (a.translation.0, a.translation.0 + a.dimensions.0);
    let (bx_min, bx_max) = (b.translation.0, b.translation.0 + b.dimensions.0);
    let (ay_min, ay_max) = (a.translation.1, a.translation.1 + a.dimensions.1);
    let (by_min, by_max) = (b.translation.1, b.translation.1 + b.dimensions.1);
    let (az_min, az_max) = (a.translation.2, a.translation.2 + a.dimensions.2);
    let (bz_min, bz_max) = (b.translation.2, b.translation.2 + b.dimensions.2);

    ax_min < bx_max
        && ax_max > bx_min
        && ay_min < by_max
        && ay_max > by_min
        && az_min < bz_max
        && az_max > bz_min
}

const PLAYER_COLLIDER_RADIUS: f32 = 0.525;
pub fn get_collider(body: &Body) -> Collider {
    let half_height = body.dimensions.2 / 2.0;
    match body.class {
        BodyClass::Player => {
            ColliderBuilder::capsule_z(half_height - PLAYER_COLLIDER_RADIUS, PLAYER_COLLIDER_RADIUS)
                .active_events(ActiveEvents::CONTACT_EVENTS)
                .build()
        }
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

pub fn to_rigid_body(body: &Body) -> RigidBody {
    match body.class {
        BodyClass::Player => to_dynamic_rigid_body(body),
        BodyClass::Bullet => to_dynamic_rigid_body(body),
        BodyClass::Test => to_dynamic_rigid_body(body),
        BodyClass::Obstacle => to_static_rigid_body(body),
    }
}

fn to_dynamic_rigid_body(body: &Body) -> RigidBody {
    RigidBodyBuilder::new(RigidBodyType::Dynamic)
        .translation(Vector3::new(
            body.translation.0,
            body.translation.1,
            body.translation.2,
        ))
        .rotation(Vector3::new(
            body.rotation.0,
            body.rotation.1,
            body.rotation.2,
        ))
        .lock_rotations()
        .linvel(Vector3::new(body.linvel.0, body.linvel.1, body.linvel.2))
        .angvel(Vector3::new(body.angvel.0, body.angvel.1, body.angvel.2))
        .additional_mass(body.mass)
        .sleeping(match (body.class, body.hp) {
            (BodyClass::Player, 0) => true,
            _ => false,
        })
        .build()
}

fn to_static_rigid_body(body: &Body) -> RigidBody {
    RigidBodyBuilder::new(RigidBodyType::Static)
        .translation(Vector3::new(
            body.translation.0,
            body.translation.1,
            body.translation.2,
        ))
        .rotation(Vector3::z() * body.rotation.2)
        .lock_rotations()
        .additional_mass(body.mass)
        .build()
}
