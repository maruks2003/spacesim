use crate::quadtree::{Node, QuadTree};
use bevy::prelude::{Circle, Color, *};

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

    let offset = 100.;
    let count = 3;
    let speed = 100.0;

    commands.spawn((
        Velocity(Vec2::ZERO),
        Mass(10_000_000_000.),
        Mesh2d(circle.clone()),
        MeshMaterial2d(material.clone()),
        Transform {
            translation: Vec3::new(0., 0., 0.),
            scale: Vec3::new(10., 10., 1.),
            ..Default::default()
        },
    ));

    for i in 0..count {
        let dir = Vec2::from_angle(((360. / count as f32) * i as f32).to_radians());

        commands.spawn((
            Velocity(Vec2::new(-dir.x * speed, dir.y * speed)),
            Mass(1_000_000.),
            Mesh2d(circle.clone()),
            MeshMaterial2d(material.clone()),
            Transform {
                translation: Vec3::new(dir.x * offset, dir.y * offset, 0.), // Offset them a bit
                scale: Vec3::new(3., 3., 1.),
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

fn calculate_gravitational_acceleration(
    query: &Query<(Entity, &Mass, &Transform)>,
    target: Entity,
    target_position: Vec2,
) -> Vec2 {
    let mut g = Vec2::new(0., 0.);

    for (entity, mass, transform) in query {
        if entity != target {
            let position = transform.translation.xy();
            let dir_vec = position - target_position;
            g += G * (mass.0 / dir_vec.length_squared()) * dir_vec.normalize();
        }
    }

    return g;
}

fn apply_acceleration(
    time: Res<Time>,
    subquery: Query<(Entity, &Mass, &Transform)>,
    mut query: Query<(Entity, &Transform, &mut Velocity)>,
) {
    if true {
        let mut q_tree = QuadTree::new(Vec2::new(0., 0.), 10.);
        for (_, mass, transform) in &subquery {
            q_tree.add_node(transform.translation.xy(), mass.0);
        }
        for (_, transform, mut velocity) in &mut query {
            let bodies = q_tree.collect_bodies(transform.translation.xy(), 0.);

            for body in bodies {
                if body.center_of_mass != transform.translation.xy() {
                    let dir_vec = body.center_of_mass - transform.translation.xy();
                    velocity.0 +=
                        (G * (body.mass / dir_vec.length_squared()) * dir_vec.normalize())
                            * time.delta_secs();
                }
            }
        }
    } else {
        for (entity, transform, mut velocity) in &mut query {
            velocity.0 += calculate_gravitational_acceleration(
                &subquery,
                entity,
                Vec2::new(transform.translation.x, transform.translation.y),
            ) * time.delta_secs();
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
