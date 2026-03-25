use bevy::color::palettes::basic::{BLUE, RED};
use bevy::prelude::*;
use rand::Rng;
use crate::{Attractions, Particle, Position, TotalForce, Velocity};

const GREY: Srgba = Srgba::new(0.5, 0.5, 0.5, 1.0);

pub fn spawn_entities(
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
                color: 0, //blue
                attractions: [
                    Attractions::Blue(attract), //a+60
                    Attractions::Red(repel), //a
                    Attractions::Grey(repel) // a-20.0
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
                color: 1, //grey
                attractions: [
                    Attractions::Blue(repel), //a-100
                    Attractions::Red(repel), //n
                    Attractions::Grey(attract), //a-40.0
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
                color: 2, //red
                attractions: [
                    Attractions::Blue(repel), //a-30
                    Attractions::Red(attract), //r+15
                    Attractions::Grey(repel) //n
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