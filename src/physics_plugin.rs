use crate::quadtree::{Node, QuadTree};
use bevy::prelude::{Circle, Color, *};
use rand::distr::StandardUniform;
use rand::prelude::*;

const G: f32 = 0.000_1;

#[derive(Component)]
struct Mass(f32);

#[derive(Component)]
struct Velocity(Vec2);

fn spawn_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let circle = meshes.add(Circle::new(1.));
    let material = materials.add(ColorMaterial::from(Color::srgb_u8(255, 0, 0)));

    let min_offset = 100.;
    let offset_random_margin = 200.;
    let count = 2_000;
    let min_speed = 100.0;
    let speed_random_margin = 100.0;
    let min_mass = 1_000_000.;
    let mass_random_margin = 19_000_000.;

    commands.spawn((
        Velocity(Vec2::ZERO),
        Mass(100_000_000_000.),
        Mesh2d(circle.clone()),
        MeshMaterial2d(material.clone()),
        Transform {
            translation: Vec3::new(0., 0., 0.),
            scale: Vec3::new(50., 50., 1.),
            ..Default::default()
        },
    ));

    let increment_angle = 360. / count as f32;
    for i in 0..count {
        let angle: f32 = (increment_angle * i as f32)
            + rand::rng().sample::<f32, StandardUniform>(StandardUniform) * increment_angle;
        let dir = Vec2::from_angle(angle.to_radians());
        let offset = min_offset
            + offset_random_margin * rand::rng().sample::<f32, StandardUniform>(StandardUniform);
        let speed = min_speed
            + speed_random_margin * rand::rng().sample::<f32, StandardUniform>(StandardUniform);
        let mass_addition =
            mass_random_margin * rand::rng().sample::<f32, StandardUniform>(StandardUniform);
        let mass = min_mass + mass_addition;

        let direction = Vec2::new(
            rand::rng().random_range(-1.0..1.0),
            rand::rng().random_range(-1.0..1.0),
        )
        .normalize();

        commands.spawn((
            Velocity(direction * speed),
            Mass(mass),
            Mesh2d(circle.clone()),
            MeshMaterial2d(material.clone()),
            Transform {
                translation: Vec3::new(dir.x * offset, dir.y * offset, 0.), // Offset them a bit
                scale: Vec3::new(
                    3. + mass_addition / 800_000.0,
                    3. + mass_addition / 800_000.0,
                    1.,
                ),
                ..Default::default()
            },
        ));
    }
}

fn update_position(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut pos, vel) in &mut query {
        pos.translation.x += vel.0.x * time.delta_secs();
        pos.translation.y += vel.0.y * time.delta_secs();
    }
}

fn apply_acceleration(
    time: Res<Time>,
    subquery: Query<(Entity, &Mass, &Transform)>,
    mut query: Query<(Entity, &Transform, &mut Velocity)>,
) {
    let mut q_tree = QuadTree::new(Vec2::new(0., 0.), 1000.);
    for (_, mass, transform) in &subquery {
        q_tree.add_node(transform.translation.xy(), mass.0);
    }
    for (_, transform, mut velocity) in &mut query {
        let bodies = q_tree.collect_bodies(transform.translation.xy(), 3.);

        for body in bodies {
            if body.center_of_mass != transform.translation.xy() {
                let dir_vec = body.center_of_mass - transform.translation.xy();
                velocity.0 += (G * (body.mass / dir_vec.length_squared()) * dir_vec.normalize())
                    * time.delta_secs();
            }
        }
    }
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_objects)
            .add_systems(Update, (update_position, apply_acceleration).chain());
    }
}
