use std::collections::HashMap;
use bevy::color::palettes::basic::{BLUE, GREEN, RED};
use bevy::prelude::*;
use bevy::reflect::Array;
use bevy::window::PrimaryWindow;
use rand::Rng;
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};

const GREY: Srgba = Srgba::new(0.5, 0.5, 0.5, 1.0);

#[derive(Debug)]
enum Attractions{
    Blue(f32),
    Red(f32),
    Grey(f32)
}
#[derive(Component)]
struct Particle {
    color: usize,
    attractions: [Attractions; 3],
    range: f32,
}

#[derive(Component, Default)]
struct Position {
    position: Vec2,
}
#[derive(Component, Default)]
struct Velocity{
    velocity: Vec2
}
#[derive(Component, Default)]
struct TotalForce {
    total_force: Vec2,
}


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
        .add_plugins(ScreenDiagnosticsPlugin::default())
        .add_plugins(ScreenFrameDiagnosticsPlugin)
        .add_systems(Startup, (spawn_camera, spawn_entities))
        .add_systems(Update,
                     (game_loop,
                              border_system,
                              movement_system_parallelisation,
                              clear_terminal)
        ).run();
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


fn game_loop (
    mut query: Query<(&Particle, &Position, &mut TotalForce)>
) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([particle1, particle2]) = combinations.fetch_next() {

        let (particle1, position1, mut total_force1) = (particle1.0, particle1.1, particle1.2);
        let (particle2, position2, mut total_force2) = (particle2.0, particle2.1, particle2.2);
        let delta:Vec2 = position1.position - position2.position;
        let distance = delta.length_squared();

        if distance <= 700.0*700.0 {

            let direction = delta.normalize();

            let force1 = &particle1.attractions[particle2.color];
            let force2 = &particle2.attractions[particle1.color];

            let central_force1 = match force1 {
                Attractions::Blue(force) => {
                    calculate_total_force(distance, particle1.range, *force)
                },
                Attractions::Grey(force) => {
                    calculate_total_force(distance, particle1.range, *force)
                },
                Attractions::Red(force) => {
                    calculate_total_force(distance, particle1.range, *force)
                }
            };

            let central_force2:f32 = match force2 {
                Attractions::Blue(force) => {
                    calculate_total_force(distance, particle2.range, *force)
                },
                Attractions::Grey(force) => {
                    calculate_total_force(distance, particle2.range, *force)
                },
                Attractions::Red(force) => {
                    calculate_total_force(distance, particle2.range, *force)
                }
            };

            total_force1.total_force += (central_force1 / (distance)) * direction;
            total_force2.total_force -= (central_force2 / (distance)) * direction;
        }
    }
}


fn movement_system_parallelisation(mut query: Query<(&mut Transform, &mut Position, &mut Velocity, &mut TotalForce)>) {
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

fn spawn_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::rng();

    let repel = 10.0;
    let neutral = 0.0;
    let attract = -10.0;

    for _ in 1..100 {
        let range = -300.0..300.0;

        let random_vector1: Vec2 = Vec2::new(rng.random_range(range.clone()), rng.random_range(range.clone()));
        let random_vector2: Vec2 = Vec2::new(rng.random_range(range.clone()), rng.random_range(range.clone()));
        let random_vector3: Vec2 = Vec2::new(rng.random_range(range.clone()), rng.random_range(range.clone()));

        commands.spawn ((
            Particle {
                color: 0,
                attractions: [
                    Attractions::Blue(attract+60.0),
                    Attractions::Red(attract),
                    Attractions::Grey(attract-20.0)
                ],
                range: 700.0
            },
            Position {
                position: random_vector1
            },
            Velocity {
                velocity: Vec2::ZERO
            },
            TotalForce {
                total_force: Vec2::ZERO
            },
            Mesh2d(meshes.add(Circle::default())),
            MeshMaterial2d(materials.add(Color::from(BLUE))),
            Transform::from_xyz(random_vector1.x, random_vector1.y, 0.0)
                .with_scale(Vec3::splat(10.0)),
        ));

        commands.spawn ((
            Particle {
                color: 1,
                attractions: [
                    Attractions::Blue(attract-100.0),
                    Attractions::Red(neutral),
                    Attractions::Grey(attract-40.0),
                ],
                range: 40.0 //40.0
            },
            Position {
                position: random_vector2
            },
            Velocity {
                velocity: Vec2::ZERO
            },
            TotalForce {
                total_force: Vec2::ZERO
            },
            Mesh2d(meshes.add(Circle::default())),
            MeshMaterial2d(materials.add(Color::from(GREY))),
            Transform::from_xyz(random_vector2.x, random_vector2.y, 0.0)
                .with_scale(Vec3::splat(10.0)),
        ));

        commands.spawn ((
            Particle {
                color: 2,
                attractions: [
                    Attractions::Blue(attract-30.0),
                    Attractions::Red(repel+15.0),
                    Attractions::Grey(neutral)
                ],
                range: 700.0
            },
            Position {
                position: random_vector3
            },
            Velocity {
                velocity: Vec2::ZERO
            },
            TotalForce {
                total_force: Vec2::ZERO
            },
            Mesh2d(meshes.add(Circle::default())),
            MeshMaterial2d(materials.add(Color::from(RED))),
            Transform::from_xyz(random_vector3.x, random_vector3.y, 0.0)
                .with_scale(Vec3::splat(8.0)),
        ));
    }
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