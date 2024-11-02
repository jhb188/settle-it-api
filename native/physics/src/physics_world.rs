use crate::body;
use crossbeam::channel::Receiver;
use rapier3d::dynamics::{ImpulseJointSet, IntegrationParameters, RigidBodySet};
use rapier3d::prelude::{
    CollisionEvent, MultibodyJointSet, QueryPipeline, RigidBody, RigidBodyHandle, Rotation,
};
use rapier3d::{
    na::Vector3,
    prelude::{
        CCDSolver, ChannelEventCollector, ColliderSet, DefaultBroadPhase, IslandManager,
        NarrowPhase, PhysicsPipeline,
    },
};
use std::collections::HashSet;

pub struct PhysicsWorld {
    pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    gravity: Vector3<f32>,
    integration_parameters: IntegrationParameters,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
    physics_hooks: (),
    event_handler: ChannelEventCollector,
    collision_receiver: Receiver<CollisionEvent>,
}

const GRAVITY: f32 = -9.80665;

pub fn init() -> PhysicsWorld {
    let pipeline = PhysicsPipeline::new();
    let island_manager = IslandManager::new();
    let gravity = Vector3::new(0.0, 0.0, GRAVITY);
    let integration_parameters = IntegrationParameters::default();
    let broad_phase = DefaultBroadPhase::new();
    let narrow_phase = NarrowPhase::new();
    let impulse_joint_set = ImpulseJointSet::new();
    let multibody_joint_set = MultibodyJointSet::new();
    let ccd_solver = CCDSolver::new();
    let query_pipeline = QueryPipeline::new();
    let physics_hooks = ();
    let (contact_send, _contact_receiver) = crossbeam::channel::unbounded();
    let (collision_send, collision_receiver) = crossbeam::channel::unbounded();
    let event_handler = ChannelEventCollector::new(collision_send, contact_send);

    PhysicsWorld {
        pipeline,
        island_manager,
        gravity,
        integration_parameters,
        broad_phase,
        narrow_phase,
        bodies: RigidBodySet::new(),
        colliders: ColliderSet::new(),
        impulse_joint_set,
        multibody_joint_set,
        ccd_solver,
        query_pipeline,
        physics_hooks,
        event_handler,
        collision_receiver,
    }
}

pub fn step(physics_world: &mut PhysicsWorld) -> Vec<CollisionEvent> {
    physics_world.pipeline.step(
        &physics_world.gravity,
        &physics_world.integration_parameters,
        &mut physics_world.island_manager,
        &mut physics_world.broad_phase,
        &mut physics_world.narrow_phase,
        &mut physics_world.bodies,
        &mut physics_world.colliders,
        &mut physics_world.impulse_joint_set,
        &mut physics_world.multibody_joint_set,
        &mut physics_world.ccd_solver,
        Some(&mut physics_world.query_pipeline),
        &physics_world.physics_hooks,
        &physics_world.event_handler,
    );

    let mut collisions = Vec::new();
    while let Ok(collision_event) = physics_world.collision_receiver.try_recv() {
        collisions.push(collision_event);
    }
    collisions
}

pub fn get_dt(physics_world: &PhysicsWorld) -> f32 {
    physics_world.integration_parameters.dt
}

pub fn remove_body(physics_world: &mut PhysicsWorld, rigid_body_handle: RigidBodyHandle) {
    physics_world.bodies.remove(
        rigid_body_handle,
        &mut physics_world.island_manager,
        &mut physics_world.colliders,
        &mut physics_world.impulse_joint_set,
        &mut physics_world.multibody_joint_set,
        true,
    );
}

pub fn get_colliders(physics_world: &PhysicsWorld) -> &ColliderSet {
    &physics_world.colliders
}

pub fn get_bodies(physics_world: &PhysicsWorld) -> &RigidBodySet {
    &physics_world.bodies
}

pub fn get_active_handles(physics_world: &PhysicsWorld) -> HashSet<RigidBodyHandle> {
    physics_world
        .island_manager
        .active_dynamic_bodies()
        .into_iter()
        .map(|handle| handle.clone())
        .collect()
}

pub fn get_body<'a>(
    physics_world: &'a PhysicsWorld,
    rigid_body_handle: &RigidBodyHandle,
) -> Option<&'a RigidBody> {
    physics_world.bodies.get(*rigid_body_handle)
}

pub fn add_body(physics_world: &mut PhysicsWorld, body: &body::Body) -> RigidBodyHandle {
    let rigid_body = body::to_rigid_body(body);
    let collider = body::get_collider(body);
    let body_handle = physics_world.bodies.insert(rigid_body);
    physics_world
        .colliders
        .insert_with_parent(collider, body_handle, &mut physics_world.bodies);
    body_handle
}

pub fn get_body_mut<'a>(
    physics_world: &'a mut PhysicsWorld,
    handle: &RigidBodyHandle,
) -> Option<&'a mut RigidBody> {
    physics_world.bodies.get_mut(*handle)
}

pub fn move_body(world: &mut PhysicsWorld, handle: &RigidBodyHandle, x: f32, y: f32) {
    if let Some(existing_body) = get_body_mut(world, handle) {
        existing_body.set_translation(Vector3::new(x, y, existing_body.translation().z), true);
    }
}

pub fn rotate_body(world: &mut PhysicsWorld, handle: &RigidBodyHandle, rotation_angle: f32) {
    if let Some(existing_body) = get_body_mut(world, handle) {
        existing_body.set_rotation(Rotation::from_euler_angles(0.0, 0.0, rotation_angle), true);
    }
}

pub fn jump_body(world: &mut PhysicsWorld, handle: &RigidBodyHandle, linvelz: f32) {
    if let Some(existing_body) = get_body_mut(world, handle) {
        let linvel = existing_body.linvel();
        existing_body.set_linvel(Vector3::new(linvel.x, linvel.y, linvelz), true);
    }
}
