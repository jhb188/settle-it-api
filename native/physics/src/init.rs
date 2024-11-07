use crate::body;
use rand::Rng;
use uuid::Uuid;

const ARENA_WIDTH: f32 = 200.0;
const MAX_OBSTACLES: usize = 500;

fn create_floor() -> body::Body {
    body::Body {
        id: String::from("floor"),
        team_id: None,
        owner_id: None,
        translation: (0.0, 0.0, -0.5),
        rotation: (0.0, 0.0, 0.0),
        linvel: (0.0, 0.0, 0.0),
        angvel: (0.0, 0.0, 0.0),
        dimensions: (ARENA_WIDTH + 0.1, ARENA_WIDTH + 0.1, 1.0),
        mass: 0.0,
        class: body::BodyClass::Obstacle,
        hp: 0,
    }
}

fn seed_obstacle_in_open_space(bodies: &mut Vec<body::Body>) {
    let obstacle = create_random_obstacle();

    if !body::overlaps_existing_bodies(&obstacle, bodies) {
        bodies.push(obstacle);
    } else {
        seed_obstacle_in_open_space(bodies);
    }
}

fn seed_obstacles(bodies: &mut Vec<body::Body>) {
    for _ in 0..MAX_OBSTACLES {
        seed_obstacle_in_open_space(bodies);
    }
}

fn create_random_obstacle() -> body::Body {
    let mut rng = rand::thread_rng();
    let margin = 25.0;
    let position_max = (ARENA_WIDTH / 2.0) - margin;
    let position_min = -position_max;
    let position_x = rng.gen_range(position_min..position_max);
    let position_y = rng.gen_range(position_min..position_max);
    let length = rng.gen_range(1.0..5.0);
    let width = rng.gen_range(1.0..5.0);
    let height = rng.gen_range(0.2..5.0);
    body::Body {
        id: Uuid::new_v4().to_string(),
        team_id: None,
        owner_id: None,
        translation: (position_x, position_y, height / 2.0),
        rotation: (0.0, 0.0, 0.0),
        linvel: (0.0, 0.0, 0.0),
        angvel: (0.0, 0.0, 0.0),
        dimensions: (length, width, height),
        mass: 100.0,
        class: body::BodyClass::Obstacle,
        hp: 0,
    }
}

pub fn get_initial_bodies() -> Vec<body::Body> {
    let mut initial_bodies: Vec<body::Body> = Vec::new();
    let floor = create_floor();
    initial_bodies.push(floor);
    seed_obstacles(&mut initial_bodies);
    initial_bodies
}
