use bevy::prelude::*;
use avian3d::prelude::*;

#[derive(Component)]
struct PlayerData {
    health: i32,
    player_name: String,
    player_id: u32,
}

fn spawn_player(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let player = PlayerData {
        health: 100,
        player_name: String::from("Player1"),
        player_id: 1,
    };
    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(200, 50, 50))),
        AngularVelocity(Vec3::new(2.5, 3.5, 1.5)),
        PlayerData {
            health: 100,
            player_name: "Admin".into(),
            player_id: 1,
        },
    ));
}

fn player_movement(keyboard_input: Res<ButtonInput<KeyCode>>, mut query: Query<(&mut LinearVelocity, &mut PlayerData)>) {
    for (mut linear_velocity, mut player) in query.iter_mut() {
        let mut move_direction = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::KeyW) && player.health > 0 {
            move_direction.z -= 0.1;
        }
        if keyboard_input.pressed(KeyCode::KeyS) && player.health > 0 {
            move_direction.z += 0.1;
        }
        if keyboard_input.pressed(KeyCode::KeyA) && player.health > 0 {
            move_direction.x -= 0.1;
        }
        if keyboard_input.pressed(KeyCode::KeyD) && player.health > 0 {
            move_direction.x += 0.1;
        }
        if keyboard_input.pressed(KeyCode::Space) && player.health > 0 {
            move_direction.y += 0.1;
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) && player.health > 0 {
            move_direction.z -= 0.1;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) && player.health > 0 {
            move_direction.z += 0.1;
        }
        if keyboard_input.pressed(KeyCode::ArrowLeft) && player.health > 0 {
            move_direction.x -= 0.1;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) && player.health > 0 {
            move_direction.x += 0.1;
        }
        
        linear_velocity.0 = move_direction;
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(4.0, 0.1),
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
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
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_systems(Startup, (spawn_player, setup))
        .add_systems(Update, player_movement)
        .run();
}
