use std::collections::HashMap;
use bevy::color::palettes::basic::{BLUE, RED};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

const GREY: Srgba = Srgba::new(0.5, 0.5, 0.5, 1.0);


#[derive(Component, Default)]
struct Particle {
    position: Vec2,
    velocity: Vec2,
    color: String,
    total_force: Vec2,
    attractions: HashMap<String, f32>,
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
        .add_systems(Startup, (spawn_camera, spawn_entitys))
        .add_systems(Update, (game_loop,
                              border_system,
                              clear_terminal).chain())
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn clear_terminal() {
    print!("\x1B[2J\x1B[1;1H");
}


fn game_loop (
    mut query: Query<(&mut Particle, &mut Transform)>,
) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([mut particle1, mut particle2]) = combinations.fetch_next() {

        let (mut particle1, mut translation1) = (particle1.0, particle1.1);
        let (mut particle2, mut translation2) = (particle2.0, particle2.1);
        let distance = translation1.translation.distance(translation2.translation);

        if distance.abs() <= 400.0 {

            let direction = Vec2::new(translation1.translation.x - translation2.translation.x, translation1.translation.y - translation2.translation.y).normalize();

            let mut centeral_force1 = 0.0;
            let mut centeral_force2 = 0.0;

            match particle1.attractions.get(particle2.color.as_str()) {
                Some(force) => {centeral_force1 = *force}
                None => {}
            }

            match particle2.attractions.get(particle1.color.as_str()) {
                Some(force) => {centeral_force2 = *force}
                None => {}
            }

            particle1.total_force += (centeral_force1 / distance) * direction;
            particle2.total_force -= (centeral_force2 / distance) * direction;
        }
    }

    for (mut particle, mut translation) in query.iter_mut() {

        particle.velocity = (particle.velocity + particle.total_force) * 0.5;
        translation.translation += Vec3::new(particle.velocity.x, particle.velocity.y, 0.0);
        particle.total_force = Vec2::ZERO;
    }
}

fn spawn_entitys(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::rng();

    let attract = -0.333;
    let to_blue = -0.666;

    let neutral = 0.0;

    let to_grey = 0.2;
    let repulse = 1.0;

    let mut blue:HashMap<String, f32> = HashMap::new();
    blue.insert("blue".parse().unwrap(), repulse);
    blue.insert("red".parse().unwrap(), attract);
    blue.insert("grey".parse().unwrap(), attract);

     let mut red = HashMap::new();
    red.insert("blue".parse().unwrap(), attract);
    red.insert("red".parse().unwrap(), repulse);
    red.insert("grey".parse().unwrap(), neutral);


     let mut grey = HashMap::new();
    grey.insert("blue".parse().unwrap(), to_blue);
    grey.insert("red".parse().unwrap(), neutral);
    grey.insert("grey".parse().unwrap(), to_grey);
    
    println!("blue: {:?}", blue);
    println!("red: {:?}", red);
    println!("grey: {:?}", grey);




    for _ in 1..40 {


        let random_vector1: Vec2 = Vec2::new(rng.random_range(-200.0..200.0), rng.random_range(-200.0..200.0));
        let random_vector2: Vec2 = Vec2::new(rng.random_range(-200.0..200.0), rng.random_range(-200.0..200.0));
        let random_vector3: Vec2 = Vec2::new(rng.random_range(-200.0..200.0), rng.random_range(-200.0..200.0));


        commands.spawn ((
            Particle {
                position: random_vector1,
                velocity: Vec2::ZERO,
                color: "blue".to_string(),
                total_force: Vec2::ZERO,
                attractions: blue.clone(),
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
                attractions: grey.clone(),
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
                attractions: red.clone(),
                total_force: Vec2::ZERO,
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
        if transform.translation.x.abs() >= screen.width {
            //println!("pos:{:?}, width:{:?}", transform.translation.x, screen.width);
            if transform.translation.x.abs() >= screen.width + 5.0{
                transform.translation.x = screen.width.copysign(transform.translation.x) + 40.0_f32.copysign(-transform.translation.x)
            }
            particle.velocity.x *= -1.0
        }
        if transform.translation.y.abs() >= screen.height {
            //println!("pos:{:?}, height:{:?}", transform.translation.y, screen.height);
            if transform.translation.y.abs() >= screen.height + 5.0{
                transform.translation.y = screen.height.copysign(transform.translation.y) + 40.0_f32.copysign(-transform.translation.y)
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