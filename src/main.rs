use bevy::prelude::*;
use avian3d::prelude::*;

#[derive(Component)]
struct Player;

#[derive(Component)]
pub struct PlayerData {
    health: i32,
    player_name: String,
    player_id: u32,
}

fn spawn_player(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, asset_server: Res<AssetServer>) {
    let player = PlayerData {
        health: 100,
        player_name: String::from("Player1"),
        player_id: 1,
    };
    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("Player\\player.glb"))),
        AngularVelocity(Vec3::new(2.5, 3.5, 1.5)),
        PlayerData {
            health: 100,
            player_name: "Admin".into(),
            player_id: 1,
        },
    ));
}

fn player_movement(keyboard_input: Res<ButtonInput<KeyCode>>, mut query: Query<(&mut LinearVelocity, &mut PlayerData)>) {
    let mut jump_times = 0;
    for (mut linear_velocity, mut player) in query.iter_mut() {
        let mut move_direction = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::KeyW) && player.health > 0 {
            move_direction.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) && player.health > 0 {
            move_direction.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) && player.health > 0 {
            move_direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) && player.health > 0 {
            move_direction.x += 1.0;
        }
        if keyboard_input.just_pressed(KeyCode::Space) && player.health > 0 {
            linear_velocity.y = 10.0;
            jump_times += 1;
        }
        if jump_times >= 2 {
            let mut timer = Timer::from_seconds(1.0, TimerMode::Once);
            if timer.just_finished() ==  true {
                jump_times = 0;
            }
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) && player.health > 0 {
            move_direction.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) && player.health > 0 {
            move_direction.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowLeft) && player.health > 0 {
            move_direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) && player.health > 0 {
            move_direction.x += 1.0;
        }
        
        linear_velocity.x = move_direction.x * 5.0; linear_velocity.z = move_direction.z * 5.0;
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
        Mesh3d(meshes.add(Cylinder::new(10.0, 0.5))),
        Collider::cylinder(10.0, 0.5),
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

fn lock_camera(query: Query<(&Transform, &PlayerData), With<Player>>, mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<Player>)>) {
    if let Ok((player_transform, _)) = query.single() {
        for mut camera_transform in camera_query.iter_mut() {
            *camera_transform = Transform::from_xyz(
                player_transform.translation.x - 2.5,
                player_transform.translation.y + 4.5,
                player_transform.translation.z + 9.0,
            ).looking_at(player_transform.translation, Vec3::Y);
        }
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        //.add_plugins(VoxelWorldPlugin::with_config(MyWorld))
        .insert_resource(Gravity(Vec3::new(0.0, -35.0, 0.0)))
        .add_systems(Startup, (spawn_player, setup))
        .add_systems(Update, (player_movement, lock_camera))
        .run();
}
