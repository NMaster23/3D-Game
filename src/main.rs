use bevy::{prelude::*, ui::RelativeCursorPosition};
use avian3d::prelude::*;
use std::time::Duration;

#[derive(Component)]
struct Player;

#[derive(Component)]
pub struct PlayerData {
    health: i32,
    player_name: String,
    player_id: u32,
}

#[derive(Resource)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    graph_handle: Handle<AnimationGraph>,
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>, mut graphs: ResMut<Assets<AnimationGraph>>,) {
    let (graph, node_indices) = AnimationGraph::from_clips([
        asset_server.load(GltfAssetLabel::Animation(0).from_asset("Player\\player.glb")),
    ]);
    let graph_handle = graphs.add(graph);
    commands.insert_resource(Animations {
        animations: node_indices,
        graph_handle,
    });
    commands.spawn((
        Transform::default(),
        GlobalTransform::default(),
        Player,
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("Player\\player.glb"))),
        LockedAxes::ROTATION_LOCKED,
        PlayerData {
            health: 100,
            player_name: "Admin".into(),
            player_id: 1,
        },
    ));
}

fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();
        // directly via the `AnimationPlayer`.
        transitions
            .play(&mut player, animations.animations[0], Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animations.graph_handle.clone()))
            .insert(transitions);
    }
}

fn movement_animations(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    _animations: Res<Animations>,
    _current_animation: Local<usize>,
) {
    for (mut player, _transitions) in &mut animation_players {
        let Some((&playing_animation_index, _)) = player.playing_animations().next() else {
            continue;
        };

        if keyboard_input.pressed(KeyCode::KeyW) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let _ = playing_animation.speed() == 0.5;
            playing_animation.resume();
        } else if keyboard_input.pressed(KeyCode::KeyS) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let _ = playing_animation.speed() == -0.5;
            playing_animation.resume();
        }
        else {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            playing_animation.pause();
        }
    }
}

fn player_movement(keyboard_input: Res<ButtonInput<KeyCode>>, mut query: Query<(&mut LinearVelocity, &mut PlayerData)>) {
    for (mut linear_velocity, player) in query.iter_mut() {
        let mut move_direction = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::KeyW) && player.health > 0 {
            move_direction.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) && player.health > 0 {
            move_direction.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) && player.health > 0 {
            move_direction.x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) && player.health > 0 {
            move_direction.x -= 1.0;
        }
        if keyboard_input.just_pressed(KeyCode::Space) && player.health > 0 {
            linear_velocity.y = 10.0;
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
        
        let velocity = move_direction.normalize_or_zero() * 5.0;
        linear_velocity.x = velocity.x;
        linear_velocity.z = velocity.z;
    }
}

fn camera_positioning(mouse_button: Res<ButtonInput<MouseButton>>, relative_cursor_position: Single<&RelativeCursorPosition>, player_data: Query<&Transform, With<Player>>, mut camera_data: Query<&mut Transform, (With<Camera3d>, Without<Player>)>) {
    let Ok(player_transform) = player_data.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_data.single_mut() else {
        return;
    };
    
    let camera_distance = 12.0;
    let camera_height_offset = 6.0;
    let player_height = 1.0;
    let focus_offset_y = player_height * 0.5;
    
    if let Some(mouse_pos) = relative_cursor_position.normalized {
        if mouse_button.pressed(MouseButton::Middle) {
            let yaw = (mouse_pos.x - 0.5) * std::f32::consts::PI * 2.0;
            let raw_pitch = (mouse_pos.y - 0.5) * std::f32::consts::PI * 0.4;
            let pitch = raw_pitch.clamp(-std::f32::consts::PI * 0.2, std::f32::consts::PI * 0.25);
            let horizontal_distance = camera_distance * pitch.cos();
            let vertical_distance = camera_distance * pitch.sin();
            let offset_x = horizontal_distance * yaw.sin();
            let offset_z = horizontal_distance * yaw.cos();
            let offset_y = vertical_distance + camera_height_offset;
            camera_transform.translation = player_transform.translation + Vec3::new(offset_x, offset_y, offset_z);
            let focus_point = player_transform.translation + Vec3::new(0.0, focus_offset_y, 0.0);
            camera_transform.look_at(focus_point, Vec3::Y);
        }
        else {
            let yaw = std::f32::consts::PI;
            let pitch = std::f32::consts::PI * 0.15;
            let horizontal_distance = camera_distance * pitch.cos();
            let vertical_distance = camera_distance * pitch.sin();
            let offset_x = horizontal_distance * yaw.sin();
            let offset_z = horizontal_distance * yaw.cos();
            let offset_y = vertical_distance + camera_height_offset;
            camera_transform.translation = player_transform.translation + Vec3::new(offset_x, offset_y, offset_z);
            let focus_point = player_transform.translation + Vec3::new(0.0, focus_offset_y, 0.0);
            camera_transform.look_at(focus_point, Vec3::Y);
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        RelativeCursorPosition::default(),
    ));
    // circular base
    commands.spawn((
        RigidBody::Static,
        Mesh3d(meshes.add(Cylinder::new(10.0, 0.5))),
        Collider::cylinder(10.0, 0.5),
        MeshMaterial3d(materials.add(Color::BLACK)),
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
        .insert_resource(Gravity(Vec3::new(0.0, -35.0, 0.0)))
        .add_systems(Startup, (spawn_player, setup))
        .add_systems(Update, (player_movement, setup_scene_once_loaded, movement_animations, camera_positioning))
        .run();
}
