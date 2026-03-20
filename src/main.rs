use std::collections::HashMap;
use bevy::color::palettes::basic::{BLUE, GREEN, RED};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};

const GREY: Srgba = Srgba::new(0.5, 0.5, 0.5, 1.0);


#[derive(Component, Default)]
struct Particle {
    color: String,
    attractions: HashMap<String, f32>,
    range: f32,
}

#[derive(Component, Default)]
struct Position {
    position: Vec2,
}
#[derive(Component, Default)]
struct TotalForce {
    total_force: Vec2,
}
#[derive(Component, Default)]
struct Velocity {
    velocity: Vec2,
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


fn game_loop (
    mut query: Query<(&Particle, &Position, &mut TotalForce)>
) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([particle1, particle2]) = combinations.fetch_next() {

        let (particle1, position1, mut total_force1) = (particle1.0, particle1.1.position, particle1.2.total_force);
        let (particle2, position2, mut total_force2) = (particle2.0, particle2.1.position, particle2.2.total_force);

        let delta:Vec2 = position1 - position2;
        let distance = delta.length_squared();

        if distance <= 700.0*700.0 {

            let direction = delta.normalize();

            let mut central_force1 = 0.0;
            let mut central_force2 = 0.0;

            if let Some(force) = particle1.attractions.get(particle2.color.as_str()) {
                if distance <= particle1.range {
                    if distance <= 20.0 {
                        if *force != 0.0 {
                            central_force1 = (*force*2.0).abs();
                        } else {
                            central_force1 = 18.0;
                        }
                    } else {
                        central_force1 = *force;
                    }
                }
            }

            if let Some(force) = particle2.attractions.get(particle1.color.as_str()) {
                if distance <= particle2.range {
                    if distance <= 20.0 {
                        if *force != 0.0 {
                            central_force2 = (*force*2.0).abs();
                        } else {
                            central_force2 = 18.0;
                        }
                    } else {
                        central_force2 = *force
                    }
                }
            }
            total_force1 += (central_force1 / (distance*distance)) * direction;
            total_force2 -= (central_force2 / (distance*distance)) * direction;
        }
    }
}

fn movement_system_parallelisation(mut query: Query<(&mut Transform, &mut Velocity, &mut Position, &mut TotalForce)>) {
    query.par_iter_mut().for_each(|(mut transform, mut velocity, mut position, mut total_force)| {

        velocity.velocity = (velocity.velocity + total_force.total_force) * 0.9;

        transform.translation.x += velocity.velocity.x;
        transform.translation.y += velocity.velocity.y;

        position.position += velocity.velocity;

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

    let mut proton:HashMap<String, f32> = HashMap::new();
    proton.insert("blue".parse().unwrap(), repel+60.0); // repel+60.0
    proton.insert("red".parse().unwrap(),  attract); //a
    proton.insert("grey".parse().unwrap(), attract-20.0); //attract-20.0

    let mut electron:HashMap<String, f32> = HashMap::new();
    electron.insert("blue".parse().unwrap(), attract-30.0); //attract-30.0
    electron.insert("red".parse().unwrap(),  repel+15.0); //repel+15.0
    electron.insert("grey".parse().unwrap(), neutral); //neutral

    let mut neutron:HashMap<String, f32> = HashMap::new();
    neutron.insert("blue".parse().unwrap(), attract-100.0); //a-100.0
    neutron.insert("red".parse().unwrap(),  neutral); // n
    neutron.insert("grey".parse().unwrap(), attract-40.0); //a-20.0
    
    println!("blue: {:?}", proton);
    println!("red: {:?}", electron);
    println!("grey: {:?}", neutron);


    for _ in 1..50 {
        let range = -300.0..300.0;

        let random_vector1: Vec2 = Vec2::new(rng.random_range(range.clone()), rng.random_range(range.clone()));
        let random_vector2: Vec2 = Vec2::new(rng.random_range(range.clone()), rng.random_range(range.clone()));
        let random_vector3: Vec2 = Vec2::new(rng.random_range(range.clone()), rng.random_range(range.clone()));

        commands.spawn ((
            Particle {
                color: "blue".to_string(),
                attractions: proton.clone(),
                range: 700.0
            },
            Position {position: random_vector1},
            Velocity {velocity: Vec2::ZERO},
            TotalForce {total_force: Vec2::ZERO},
            Mesh2d(meshes.add(Circle::default())),
            MeshMaterial2d(materials.add(Color::from(BLUE))),
            Transform::from_xyz(random_vector1.x, random_vector1.y, 0.0)
                .with_scale(Vec3::splat(10.0)),
        ));

        commands.spawn ((
            Particle {
                color: "grey".to_string(),
                attractions: neutron.clone(),
                range: 40.0 //40.0
            },
            Position {position: random_vector2},
            Velocity {velocity: Vec2::ZERO},
            TotalForce {total_force: Vec2::ZERO},
            Mesh2d(meshes.add(Circle::default())),
            MeshMaterial2d(materials.add(Color::from(GREY))),
            Transform::from_xyz(random_vector2.x, random_vector2.y, 0.0)
                .with_scale(Vec3::splat(10.0)),
        ));

        commands.spawn ((
            Particle {
                color: "red".to_string(),
                attractions: electron.clone(),
                range: 700.0
            },
            Position {position: random_vector3},
            Velocity {velocity: Vec2::ZERO},
            TotalForce {total_force: Vec2::ZERO},
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