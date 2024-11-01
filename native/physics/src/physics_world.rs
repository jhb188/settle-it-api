use crate::body;
use crossbeam::channel::Receiver;
use rapier3d::dynamics::{ImpulseJointSet, IntegrationParameters, RigidBodySet};
use rapier3d::prelude::{
    CollisionEvent, MultibodyJointSet, QueryPipeline, RigidBody, RigidBodyHandle,
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

pub fn step<F: FnMut(CollisionEvent, &mut PhysicsWorld)>(
    physics_world: &mut PhysicsWorld,
    mut f_handle_contact: F,
) -> HashSet<RigidBodyHandle> {
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

    while let Ok(collision_event) = physics_world.collision_receiver.try_recv() {
        f_handle_contact(collision_event, physics_world);
    }

    get_active_handles(physics_world)
}

pub fn get_tick_ms(physics_world: &PhysicsWorld) -> f32 {
    physics_world.integration_parameters.dt * 1000.0
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

fn get_active_handles(physics_world: &PhysicsWorld) -> HashSet<RigidBodyHandle> {
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
) -> Option<&'a mut RigidBody> {
    physics_world.bodies.get_mut(*handle)
}
