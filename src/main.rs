mod spawn;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, EntityCountDiagnosticsPlugin, LogDiagnosticsPlugin};use bevy::tasks::{AsyncComputeTaskPool, Task};
use crate::spawn::spawn_entities;

#[derive(Component)]
pub struct ForceCalculationTask(Task<Vec2>);

#[derive(Debug, Clone)]
pub enum Attractions{
    Blue(f32),
    Red(f32),
    Grey(f32)
}
#[derive(Component, Clone, Debug)]
pub struct Particle {
    color: usize,
    attractions: [Attractions; 3],
    range: f32,
}

#[derive(Component, Default, Clone)]
pub struct Position {
    position: Vec2,
}
#[derive(Component, Default)]
pub struct Velocity{
    velocity: Vec2
}
#[derive(Component, Default)]
pub struct TotalForce {
    total_force: Vec2,
}

#[derive(Component)]
struct ComputeTransform(Task<()>);

struct Screen {
    width: f32,
    height: f32,
}
impl Screen {
    fn new(width: f32, height: f32) -> Self { Screen { width, height } }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(EntityCountDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_systems(Startup, (spawn_camera, spawn_entities))
        .add_systems(Update, (
             async_game_loop,
             async_apply_forces,
             movement_system_parallelization,
             border_system,
             clear_terminal
        ).chain()).run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn clear_terminal() {
    print!("\x1B[2J\x1B[1;1H");
}

fn calculate_total_force (distance: f32, range: f32, force: f32) -> f32 {
    if distance <= range*range {
        if distance <= 20.0 * 20.0 {
            if force != 0.0 {
                (force * 2.0).abs()
            } else {
                18.0
            }
        } else {
            force
        }
    } else {
        0.0
    }
}

fn get_central_force (force: &Attractions, distance: f32, range: f32) -> f32{
    match force {
        Attractions::Blue(force) => {
            calculate_total_force(distance, range, *force)
        },
        Attractions::Grey(force) => {
            calculate_total_force(distance, range, *force)
        },
        Attractions::Red(force) => {
            calculate_total_force(distance, range, *force)
        }
    }
}
/*
fn game_loop (
    commands: &mut Commands,
    mut query: Query<(Entity, &Particle, &Position, &mut TotalForce)>
) {

    let thread_pool = AsyncComputeTaskPool::get();

    let mut combinations = query.iter_combinations_mut();
    while let Some([particle1, particle2]) = combinations.fetch_next() {

        let (entity1, particle1, position1, mut total_force1) = (particle1.0, particle1.1, particle1.2, particle1.3);
        let (entity2, particle2, position2, mut total_force2) = (particle2.0, particle2.1, particle2.2, particle2.3);

        let delta: Vec2 = position1.position - position2.position;
        let distance = delta.length_squared();
        let direction = delta.normalize();

        let task1 = thread_pool.spawn(async move {
            if distance <= 700.0 * 700.0 {
                let force1:&Attractions = &particle1.attractions[particle2.color];
                let central_force1 = get_central_force(force1, distance, particle1.range);
                total_force1.total_force += (central_force1 / distance) * direction;
            }
        });

        let task2 = thread_pool.spawn(async move {
            if distance <= 700.0 * 700.0 {
                let force2:&Attractions = &particle2.attractions[particle1.color];
                let central_force2 = get_central_force(force2, distance, particle2.range);
                total_force2.total_force -= (central_force2 / distance) * direction;
            }
        });

        commands.entity(entity1).insert(ComputeTransform(task1));
        commands.entity(entity2).insert(ComputeTransform(task2));
    }
}

 */


fn async_game_loop(
    mut commands: Commands,
    query: Query<(Entity, &Particle, &Position), Without<ForceCalculationTask>>,  // Don't recalc entities with pending tasks
) {
    let thread_pool = AsyncComputeTaskPool::get();

    // Collect all pairs first (you can't iterate mutably and spawn tasks)
    let pairs: Vec<[(Entity, Particle, Position); 2]> = query
        .iter_combinations()
        .map(|[(e1, p1, pos1), (e2, p2, pos2)]| {
            [(e1.clone(), p1.clone(), pos1.clone()), (e2.clone(), p2.clone(), pos2.clone())]
        })
        .collect();

    for [particle1, particle2] in pairs {
        let (entity1, particle1, position1) = particle1;
        let (entity2, particle2, position2) = particle2;

        let delta = position1.position - position2.position;
        let distance_sq = delta.length_squared();

        if distance_sq <= 700.0 * 700.0 && distance_sq > 0.0 {

            let direction = (delta*delta) / distance_sq;

            // Spawn async task for particle 1
            let particle1_clone = particle1.clone();
            let task1 = thread_pool.spawn(async move {
                let force = &particle1_clone.attractions[particle2.color];
                let central_force = get_central_force(force, distance_sq, particle1_clone.range);
                (central_force / distance_sq) * direction  // Return the force vector
            });
            commands.entity(entity1).insert(ForceCalculationTask(task1));

            // Spawn async task for particle 2
            let particle2_clone = particle2.clone();
            let task2 = thread_pool.spawn(async move {
                let force = &particle2_clone.attractions[particle1.color];
                let central_force = get_central_force(force, distance_sq, particle2_clone.range);
                -(central_force / distance_sq) * direction
            });
            commands.entity(entity2).insert(ForceCalculationTask(task2));
        }
    }
}

fn async_apply_forces(
    mut commands: Commands,
    mut query: Query<(Entity, &mut TotalForce, &mut ForceCalculationTask)>,
) {
    for (entity, mut total_force, mut task) in query.iter_mut() {
        if let Some(force_vector) = futures_lite::future::block_on(futures_lite::future::poll_once(&mut task.0)) {
            total_force.total_force += force_vector;
            commands.entity(entity).remove::<ForceCalculationTask>();
        }
    }
}

fn movement_system_parallelization(mut query: Query<(&mut Transform, &mut Position, &mut Velocity, &mut TotalForce)>) {
    query.par_iter_mut().for_each(|(mut transform, mut position, mut velocity, mut total_force)| {
        //calculate change
        velocity.velocity = (velocity.velocity + total_force.total_force) * 0.9;
        //animate change
        transform.translation.x += velocity.velocity.x;
        transform.translation.y += velocity.velocity.y;
        //save change
        position.position.x = transform.translation.x;
        position.position.y = transform.translation.y;
        //reset
        total_force.total_force = Vec2::ZERO;
    });
}

fn border_system(
    q_windows: Single<&Window, With<PrimaryWindow>>,
    mut query: Query<(&mut Transform, &mut Velocity)>
){
    let screen = check_screen(*q_windows);
    for (mut transform, mut velocity ) in query.iter_mut() {
        if transform.translation.x.abs() >= screen.width-5.0 {
            //println!("pos:{:?}, width:{:?}", transform.translation.x.abs(), screen.width);

            if transform.translation.x.abs() >= screen.width {
                transform.translation.x = screen.width.copysign(transform.translation.x);
            }

            velocity.velocity.x *= -1.0
        }
        if transform.translation.y.abs() >= screen.height-5.0 {
            //println!("pos:{:?}, height:{:?}", transform.translation.y, screen.height);

            if transform.translation.y.abs() >= screen.height{
                transform.translation.y = screen.height.copysign(transform.translation.y);
            }

            velocity.velocity.y *= -1.0
        }
    }
}

fn check_screen(window: &Window) -> Screen {
    let width = window.resolution.width()/2.0;
    let height = window.resolution.height()/2.0;
    Screen::new(width, height)
}