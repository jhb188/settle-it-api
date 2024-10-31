use crate::body;
use crossbeam::channel::Receiver;
use rapier3d::dynamics::{IntegrationParameters, JointSet, RigidBodySet};
use rapier3d::prelude::{RigidBody, RigidBodyHandle};
use rapier3d::{
    na::Vector3,
    prelude::{
        BroadPhase, CCDSolver, ChannelEventCollector, ColliderSet, ContactEvent, IslandManager,
        NarrowPhase, PhysicsPipeline,
    },
};
use std::collections::HashSet;

pub struct PhysicsWorld {
    pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    gravity: Vector3<f32>,
    integration_parameters: IntegrationParameters,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    joints: JointSet,
    ccd_solver: CCDSolver,
    physics_hooks: (),
    event_handler: ChannelEventCollector,
    contact_receiver: Receiver<ContactEvent>,
}

const GRAVITY: f32 = -9.80665;

pub fn init() -> PhysicsWorld {
    let pipeline = PhysicsPipeline::new();
    let island_manager = IslandManager::new();
    let gravity = Vector3::new(0.0, 0.0, GRAVITY);
    let integration_parameters = IntegrationParameters::default();
    let broad_phase = BroadPhase::new();
    let narrow_phase = NarrowPhase::new();
    let joints = JointSet::new();
    let ccd_solver = CCDSolver::new();
    let physics_hooks = ();
    let (contact_send, contact_receiver) = crossbeam::channel::unbounded();
    let (intersection_send, _intersection_receiver) = crossbeam::channel::unbounded();
    let event_handler = ChannelEventCollector::new(intersection_send, contact_send);

    PhysicsWorld {
        pipeline,
        island_manager,
        gravity,
        integration_parameters,
        broad_phase,
        narrow_phase,
        bodies: RigidBodySet::new(),
        colliders: ColliderSet::new(),
        joints,
        ccd_solver,
        physics_hooks,
        event_handler,
        contact_receiver,
    }
}

pub fn step(physics_world: &mut PhysicsWorld) {
    physics_world.pipeline.step(
        &physics_world.gravity,
        &physics_world.integration_parameters,
        &mut physics_world.island_manager,
        &mut physics_world.broad_phase,
        &mut physics_world.narrow_phase,
        &mut physics_world.bodies,
        &mut physics_world.colliders,
        &mut physics_world.joints,
        &mut physics_world.ccd_solver,
        &physics_world.physics_hooks,
        &physics_world.event_handler,
    );
}

pub fn get_tick_ms(physics_world: &PhysicsWorld) -> f32 {
    physics_world.integration_parameters.dt * 1000.0
}

pub fn remove_body(physics_world: &mut PhysicsWorld, rigid_body_handle: RigidBodyHandle) {
    physics_world.bodies.remove(
        rigid_body_handle,
        &mut physics_world.island_manager,
        &mut physics_world.colliders,
        &mut physics_world.joints,
    );
}

pub fn get_colliders(physics_world: &PhysicsWorld) -> &ColliderSet {
    &physics_world.colliders
}

pub fn get_bodies(physics_world: &PhysicsWorld) -> &RigidBodySet {
    &physics_world.bodies
}

pub fn get_contact_receiver(physics_world: &PhysicsWorld) -> &Receiver<ContactEvent> {
    &physics_world.contact_receiver
}

pub fn get_active_handles(physics_world: &PhysicsWorld) -> HashSet<RigidBodyHandle> {
    physics_world
        .island_manager
        .active_dynamic_bodies()
        .into_iter()
        .map(|handle_ref| handle_ref.clone())
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
) -> &'a mut RigidBody {
    physics_world.bodies.get_mut(*handle).unwrap()
}
