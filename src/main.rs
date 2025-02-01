use bevy::prelude::{Circle, Color, *};

#[derive(Component)]
struct Mass(u64);

#[derive(Component)]
struct Velocity(Vec2);

pub trait Vec2Extensions {
    fn from_degrees(degrees: f32) -> Self;
    fn from_radians(radians: f32) -> Self;
}

impl Vec2Extensions for Vec2 {
    fn from_radians(radians: f32) -> Self {
        Vec2::new(radians.cos(), radians.sin())
    }
    fn from_degrees(degrees: f32) -> Self {
        Vec2::from_radians(degrees.to_radians())
    }
}

fn spawn_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let circle = meshes.add(Circle::new(1.));
    let material = materials.add(ColorMaterial::from(Color::srgb_u8(255, 0, 0)));

    let offset = 20.;

    for i in 0..20000 {
        let dir = Vec2::from_degrees((360. / 1000.) * i as f32);

        commands.spawn((
            Velocity(dir * 30.0),
            Mass(1),
            Mesh2d(circle.clone()),
            MeshMaterial2d(material.clone()),
            Transform {
                translation: Vec3::new(dir.x * offset, dir.y * offset, 0.), // Offset them a bit
                scale: Vec3::new(1., 1., 1.),
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

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_objects)
            .add_systems(Update, update_position);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugin)
        .run();
}
