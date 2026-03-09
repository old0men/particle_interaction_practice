use std::collections::HashMap;
use bevy::color::palettes::basic::{BLUE, GREEN, RED};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};

const GREY: Srgba = Srgba::new(0.5, 0.5, 0.5, 1.0);


#[derive(Component, Default)]
struct Particle {
    position: Vec2,
    velocity: Vec2,
    color: String,
    total_force: Vec2,
    attractions: HashMap<String, f32>,
    range: f32,
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
    mut query: Query<(&mut Particle, &mut Transform)>
) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([mut particle1, mut particle2]) = combinations.fetch_next() {

        let (mut particle1, mut translation1) = (particle1.0, particle1.1);
        let (mut particle2, mut translation2) = (particle2.0, particle2.1);
        let distance = translation1.translation.distance(translation2.translation).abs();

        if distance <= 700.0 {

            let direction = Vec2::new(translation1.translation.x - translation2.translation.x, translation1.translation.y - translation2.translation.y).normalize();

            let mut central_force1 = 0.0;
            let mut central_force2 = 0.0;

            match particle1.attractions.get(particle2.color.as_str()) {
                Some(force) => {
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
                None => {}
            }
                    // second particle get the attraction force equal at the position of the first particle
            match particle2.attractions.get(particle1.color.as_str()) {
                Some(force) => {
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
                None => {}
            }

            particle1.total_force += (central_force1 / (distance*distance)) * direction;
            particle2.total_force -= (central_force2 / (distance*distance)) * direction;
        }
    }
}


fn position_update(mut query: Query<(&mut Particle, &mut Transform)>){
    for (mut particle, mut translation) in query.iter_mut() {

        particle.velocity = (particle.velocity + particle.total_force) * 0.9;

        translation.translation += Vec3::new(particle.velocity.x, particle.velocity.y, 0.0);
        particle.total_force = Vec2::ZERO;
    }
}

fn movement_system_parallelisation(mut query: Query<(&mut Transform, &mut Particle)>) {
    query.par_iter_mut().for_each(|(mut transform, mut particle)| {
        particle.velocity = (particle.velocity + particle.total_force) * 0.9;

        transform.translation += Vec3::new(particle.velocity.x, particle.velocity.y, 0.0);
        particle.total_force = Vec2::ZERO;
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


    for _ in 1..300 {
        let range = -300.0..300.0;

        let random_vector1: Vec2 = Vec2::new(rng.random_range(range.clone()), rng.random_range(range.clone()));
        let random_vector2: Vec2 = Vec2::new(rng.random_range(range.clone()), rng.random_range(range.clone()));
        let random_vector3: Vec2 = Vec2::new(rng.random_range(range.clone()), rng.random_range(range.clone()));

        commands.spawn ((
            Particle {
                position: random_vector1,
                velocity: Vec2::ZERO,
                color: "blue".to_string(),
                total_force: Vec2::ZERO,
                attractions: proton.clone(),
                range: 700.0
            },
            Mesh2d(meshes.add(Circle::default())),
            MeshMaterial2d(materials.add(Color::from(BLUE))),
            Transform::from_xyz(random_vector1.x, random_vector1.y, 0.0)
                .with_scale(Vec3::splat(10.0)),
        ));

        commands.spawn ((
            Particle {
                position: random_vector2,
                velocity: Vec2::ZERO,
                color: "grey".to_string(),
                total_force: Vec2::ZERO,
                attractions: neutron.clone(),
                range: 40.0 //40.0
            },
            Mesh2d(meshes.add(Circle::default())),
            MeshMaterial2d(materials.add(Color::from(GREY))),
            Transform::from_xyz(random_vector2.x, random_vector2.y, 0.0)
                .with_scale(Vec3::splat(10.0)),
        ));

        commands.spawn ((
            Particle {
                position: random_vector3,
                velocity: Vec2::ZERO,
                color: "red".to_string(),
                attractions: electron.clone(),
                total_force: Vec2::ZERO,
                range: 700.0
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
    mut query: Query<(&mut Transform, &mut Particle)>
){
    let screen = check_screen(*q_windows);
    for (mut transform, mut particle ) in query.iter_mut() {
        if transform.translation.x.abs() >= screen.width-5.0 {
            //println!("pos:{:?}, width:{:?}", transform.translation.x.abs(), screen.width);

            if transform.translation.x.abs() >= screen.width {
                transform.translation.x = screen.width.copysign(transform.translation.x);
            }

            particle.velocity.x *= -1.0
        }
        if transform.translation.y.abs() >= screen.height-5.0 {
            //println!("pos:{:?}, height:{:?}", transform.translation.y, screen.height);

            if transform.translation.y.abs() >= screen.height{
                transform.translation.y = screen.height.copysign(transform.translation.y);
            }

            particle.velocity.y *= -1.0
        }
    }
}

fn check_screen(window: &Window) -> Screen {
    let width = window.resolution.width()/2.0;
    let height = window.resolution.height()/2.0;
    Screen::new(width, height)
}