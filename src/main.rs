use bevy::{post_process::bloom::Bloom, prelude::*, ui::RelativeCursorPosition, window::WindowResolution};
use avian3d::prelude::*;
use std::time::Duration;
use avian3d::math::PI;

#[derive(Component)]
pub struct Lighting;

#[derive(Component)]
struct Bots;

#[derive(Component)]
struct BotData {
    health: i32,
    bot_id: u32,
    bot_quantity: u32,
    bot_offset: f32,
}

#[derive(Component)]
struct CharacterController {
    pub move_direction: Vec3,
}

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

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>, mut graphs: ResMut<Assets<AnimationGraph>>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let (graph, node_indices) = AnimationGraph::from_clips([
        asset_server.load(GltfAssetLabel::Animation(0).from_asset("Player\\Player.glb")),
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
        Collider::cuboid(1.0, 3.0, 1.0),
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("Player\\Player.glb"))),
        PlayerData {
            health: 100,
            player_name: "Admin".into(),
            player_id: 1,
        },
        CharacterController {
            move_direction: Vec3::ZERO,
        },
    ))
    .with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(Sphere::new(0.1))),
            MeshMaterial3d(materials.add(StandardMaterial {
                emissive: (Color::WHITE).into(),
                ..default()
            })),
            SpotLight {
                intensity: 1_000_000.0,
                range: 10.0,
                inner_angle: PI / 8.0,
                outer_angle: PI / 6.0,
                shadows_enabled: true,
                shadow_depth_bias: 0.2,
                shadow_normal_bias: 0.2,
                color: Color::WHITE,
                ..default()
            },
            Lighting,
            Visibility::Visible,
            Transform::from_xyz(0.0, 3.0, 0.0).looking_at(Vec3::new(0.0, 3.0, 10.0), Vec3::Y),
        ));
    });
}

fn bot_spawn(mut commands: Commands, asset_server: Res<AssetServer>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let bot_number = 1;
    let mut bots = BotData {
        health: 100,
        bot_id: 1,
        bot_quantity: bot_number,
        bot_offset: 0.0,
    };
    for i in 0..bots.bot_quantity {
        bots.bot_offset = i as f32 * bots.bot_quantity as f32 - 10.0;
        commands.spawn((
            GlobalTransform::default(),
            Bots,
            RigidBody::Dynamic,
            Collider::cuboid(1.0, 3.0, 1.0),
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("Player\\Player.glb"))),
            BotData {
                health: 100,
                bot_id: i + 1,
                bot_quantity: bots.bot_quantity,
                bot_offset: i as f32 * 2.0 - (bots.bot_quantity as f32 - 1.0) * 2.0 / 2.0,
            },
            Transform::from_xyz(bots.bot_offset, 0.0, -5.0),
            CharacterController {
                move_direction: Vec3::ZERO,
            },
        ))
        .with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(Sphere::new(0.1))),
            MeshMaterial3d(materials.add(StandardMaterial {
                emissive: (Color::WHITE).into(),
                ..default()
            })),
            SpotLight {
                intensity: 500_000.0,
                range: 30.0,
                shadows_enabled: true,
                shadow_depth_bias: 0.2,
                shadow_normal_bias: 0.2,
                color: Color::WHITE,
                ..default()
            },
            Transform::from_xyz(0.0, 3.0, 0.0).looking_at(Vec3::new(0.0, 3.0, 10.0), Vec3::Y),
        ));
    });
    }
}

fn bot_handling(mut query: Query<(&mut Transform, &mut CharacterController), (With<Bots>, Without<Player>)>, mut player_data: Query<&mut Transform, With<Player>>, mut commands: Commands, asset_server: Res<AssetServer>) {
    let Ok(mut player_transform) = player_data.single_mut() else {
        return;
    };
    
    
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
            playing_animation.set_speed(1.0);
            playing_animation.resume();
        } else if keyboard_input.pressed(KeyCode::KeyS) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            playing_animation.set_speed(-1.0);
            playing_animation.resume();
        }
        else {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            playing_animation.pause();
        }
    }
}

fn player_movement(keyboard_input: Res<ButtonInput<KeyCode>>, mut query: Query<(&Transform, &mut LinearVelocity, &mut PlayerData, &mut CharacterController), With<Player>>) {
    for (transform, mut linear_velocity, mut player, mut controller) in query.iter_mut() {
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
        
        let velocity = (transform.rotation * move_direction).normalize_or_zero() * 5.0;
        linear_velocity.x = velocity.x;
        linear_velocity.z = velocity.z;

        controller.move_direction = (linear_velocity.x, 0.0, linear_velocity.z).into();
    }
}

fn camera_positioning(relative_cursor_position: Single<&RelativeCursorPosition>, mut player_data: Query<&mut Transform, With<Player>>, mut camera_data: Query<&mut Transform, (With<Camera3d>, Without<Player>)>) {
    let Ok(mut player_transform) = player_data.single_mut() else {
        return;
    };
    let Ok(mut camera_transform) = camera_data.single_mut() else {
        return;
    };
    
    let camera_distance = 2.0;
    let camera_height_offset = 4.0;
    let player_height = 2.0;
    let focus_offset_y = player_height * 0.5;
    let focus_distance = 2.0;
    
    if let Some(mouse_pos) = relative_cursor_position.normalized {
        let yaw = (mouse_pos.x - 0.5) * std::f32::consts::PI * 2.0;
        let raw_pitch = ((mouse_pos.y - 0.5) * std::f32::consts::PI * 0.4) + std::f32::consts::FRAC_PI_4;
        let pitch = raw_pitch.clamp(-std::f32::consts::PI * 0.2, std::f32::consts::PI * 0.25);
        let horizontal_distance = camera_distance * pitch.cos();
        let vertical_distance = camera_distance * pitch.sin();
        let offset_x = -horizontal_distance * yaw.sin();
        let offset_z = -horizontal_distance * yaw.cos();
        let offset_y = vertical_distance + camera_height_offset + 2.0;
        let forward_direction = Vec3::new(yaw.sin(), 0.0, yaw.cos());
        camera_transform.translation = player_transform.translation + Vec3::new(offset_x, offset_y, offset_z);
        let focus_point = player_transform.translation + Vec3::new(0.0, focus_offset_y, 0.0) + forward_direction * focus_distance;
        camera_transform.look_at(focus_point, Vec3::Y);
        player_transform.rotation = Quat::from_rotation_y(yaw);
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
    commands.spawn((
        RigidBody::Static,
        Mesh3d(meshes.add(Cuboid::new(100.0, 0.25, 100.0))),
        Collider::cuboid(100.0, 0.25, 100.0),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        Bloom::NATURAL,
    ));
}

pub fn setup_lighting(mut query: Query<&mut Visibility, With<Lighting>>, keycode: Res<ButtonInput<KeyCode>>) {
    if keycode.just_pressed(KeyCode::KeyQ) {
        for mut visibility in &mut query {
            *visibility = match *visibility {
                Visibility::Hidden => Visibility::Visible,
                _ => Visibility::Hidden,
            }
        }
    }
}

fn main() {
    App::new() 
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Mech Game".into(),
                resolution: WindowResolution::new(1920, 1080),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity(Vec3::new(0.0, -35.0, 0.0))) 
        .add_systems(Startup, (spawn_player, setup))
        .add_systems(Startup, bot_spawn)
        .add_systems(Update, (player_movement, setup_scene_once_loaded, movement_animations, camera_positioning, setup_lighting, bot_handling))
        .run();
}