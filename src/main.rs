use bevy::prelude::*;

#[derive(Component)]
struct PlayerData {
    health: i32,
    x: f32,
    y: f32,
    z: f32,
    player_name: String
}

fn spawn_player(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let player = PlayerData {
        health: 100,
        x: 0.0,
        y: 0.0,
        z: 0.0,
        player_name: String::from("Player1")
    };
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(200, 50, 50))),
        PlayerData {
            health: 100,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            player_name: "Player1".into(),
        },
        Transform::from_xyz(player.x, player.y, player.z),
    ));
    println!("Spawned player: {} with health: {} at X: {} Y: {} Z: {}", player.player_name, player.health, player.x, player.y, player.z);
}

fn player_movement(keyboard_input: Res<ButtonInput<KeyCode>>, mut player_pos: ) {
    if keyboard_input.pressed(KeyCode::KeyW) {
        player.y += 0.1;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        player.y -= 0.1;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        player.x -= 0.1;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        player.x += 0.1;
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn main() {
        App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (spawn_player, setup))
//        .add_systems(Update, systems)
        .run();
}
