use bevy::{post_process::bloom::Bloom, prelude::*, ui::RelativeCursorPosition, window::WindowResolution, window::{CursorGrabMode, CursorOptions}, picking::backend::ray::RayMap};
use bevy::input::mouse::AccumulatedMouseMotion;
use avian3d::prelude::*;
use std::time::Duration;
use avian3d::math::PI;
use rand::prelude::*;

const LASER_SPEED: f32 = 0.03;

#[derive(Component)]
pub struct Lighting;

#[derive(Component)]
struct Crosshair;

#[derive(Component)]
struct Bots;

#[derive(Component)]
struct HealthBarUI;

#[derive(Component)]
struct BotData {
    health: i32,
    bot_id: u32,
    bot_quantity: u32,
    bot_offset: f32,
    hit_number: i32,
}

#[derive(Component)]
struct CharacterController {
    pub move_direction: Vec3,
}

#[derive(Resource, Default)]
struct FloatingCrosshair(Vec2);

#[derive(Component)]
struct GunBarrel;

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

fn cursor_handling(mut cursor: Single<&mut CursorOptions, With<Window>>, keycode: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>) {
    if mouse.just_pressed(MouseButton::Left) {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
    if keycode.just_pressed(KeyCode::Escape) {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }
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
        GlobalTransform::default(),
        Player,
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 3.0, 1.0),
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("Player\\Player.glb"))),
        Transform::from_xyz(0.0, 10.0, 0.0),
        PlayerData {
            health: 5,
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
        parent.spawn((
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("Gun\\Gun.glb"))),
            Transform::from_xyz(1.5, 2.0, 1.0),
        ));
    });
}

fn bouncing_raycast(
    mut ray_cast: MeshRayCast,
    mut gizmos: Gizmos,
    time: Res<Time>,
    // The ray map stores rays cast by the cursor
    ray_map: Res<RayMap>,
) {
    // Cast an automatically moving ray and bounce it off of surfaces
    let t = ops::cos((time.elapsed_secs() - 4.0).max(0.0) * LASER_SPEED) * PI;
    let ray_pos = Vec3::new(ops::sin(t), ops::cos(3.0 * t) * 0.5, ops::cos(t)) * 0.5;
    let ray_dir = Dir3::new(-ray_pos).unwrap();
    let ray = Ray3d::new(ray_pos, ray_dir);
    gizmos.sphere(ray_pos, 0.1, Color::WHITE);
}

fn bot_spawn(mut commands: Commands, asset_server: Res<AssetServer>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let bot_number = 1;
    let mut rng = rand::rng();
    let hits = rng.random_range(1..20);
    let hits_num = rng.random_range(1..5);
    let mut bots = BotData {
        health: hits,
        bot_id: 1,
        bot_quantity: bot_number,
        bot_offset: 0.0,
        hit_number: hits_num
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
                health: hits,
                bot_id: i + 1,
                bot_quantity: bots.bot_quantity,
                bot_offset: i as f32 * 2.0 - (bots.bot_quantity as f32 - 1.0) * 2.0 / 2.0,
                hit_number: hits_num,
            },
            Transform::from_xyz(bots.bot_offset, 10.0, -5.0),
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
            Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::new(0.0, 3.0, 10.0), Vec3::Y),
        ));
    });
    }
}

fn bot_handling(mut query: Query<(&mut Transform, &mut CharacterController, &BotData), (With<Bots>, Without<Player>)>, mut player_data: Query<(&mut Transform, &mut PlayerData), With<Player>>, mut commands: Commands, asset_server: Res<AssetServer>) {
    let Ok((mut player_transform, mut player_data)) = player_data.single_mut() else {
        return;
    };
    for (mut transform, mut controller, bot_data) in query.iter_mut() {
        let direction_to_player = (player_transform.translation - transform.translation).normalize_or_zero();
        controller.move_direction = direction_to_player * 2.0;
        transform.rotation = Quat::from_rotation_y(direction_to_player.x.atan2(direction_to_player.z));
        let mut rng = rand::rng();
        if rng.random_range(1..500) == bot_data.hit_number {
            player_data.health -= 1;
        }
    }
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
    for (transform, mut linear_velocity, player, mut controller) in query.iter_mut() {
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

fn camera_positioning(mut query: Query<&mut Node, With<Crosshair>>, mut crosshair_offset: Local<Vec2>, mouse_button: Res<ButtonInput<MouseButton>>, mouse_movement: Res<AccumulatedMouseMotion>, mut player_data: Query<&mut Transform, With<Player>>, mut camera_data: Query<&mut Transform, (With<Camera3d>, Without<Player>)>, mut rotation: Local<Vec2>) {
    let Ok(mut player_transform) = player_data.single_mut() else {
        return;
    };
    let Ok(mut camera_transform) = camera_data.single_mut() else {
        return;
    };
    let camera_distance = 10.0;
    let camera_height_offset = 4.0;
    let focus_offset_y = 1.5;
    let focus_distance = 2.0;
    let sens = 0.1;
    rotation.x += -mouse_movement.delta.x * sens;
    rotation.y += mouse_movement.delta.y * sens;
    rotation.y = rotation.y.clamp(-34.9, 89.9);
    *crosshair_offset += mouse_movement.delta * 0.5;
    *crosshair_offset = crosshair_offset.lerp(Vec2::ZERO, 0.02);
    *crosshair_offset = crosshair_offset.clamp(Vec2::splat(-150.0), Vec2::splat(150.0));
    if let Ok(mut node) = query.single_mut() {
        node.left = Val::Px(crosshair_offset.x);
        node.top = Val::Px(crosshair_offset.y);
    }
    let yaw = rotation.x.to_radians();
    let pitch = rotation.y.to_radians();
    let horizontal_distance = camera_distance * pitch.cos();
    let vertical_distance = camera_distance * pitch.sin();
    let offset_x = -horizontal_distance * yaw.sin();
    let offset_z = -horizontal_distance * yaw.cos();
    let offset_y = vertical_distance + camera_height_offset;
    camera_transform.translation = player_transform.translation + Vec3::new(offset_x, offset_y, offset_z);
    let forward_direction = Vec3::new(yaw.sin(), 0.0, yaw.cos());
    let focus_point = player_transform.translation + Vec3::new(0.0, focus_offset_y, 0.0) + forward_direction * focus_distance;
    camera_transform.look_at(focus_point, Vec3::Y);
    player_transform.rotation = Quat::from_rotation_y(yaw);
    if mouse_button.pressed(MouseButton::Middle) {
        let camera_distance = 10.0;
        let camera_height_offset = 1.0;
        let focus_offset_y = 1.5;
        let focus_distance = 2.0;
        let sens = 0.1;
        rotation.x += mouse_movement.delta.x * sens;
        rotation.y += mouse_movement.delta.y * sens;
        rotation.y = rotation.y.clamp(-89.9, 89.9);
        let yaw = rotation.x.to_radians();
        let pitch = rotation.y.to_radians();
        let horizontal_distance = camera_distance * pitch.cos();
        let vertical_distance = camera_distance * pitch.sin();
        let offset_x = horizontal_distance * yaw.sin();
        let offset_z = -horizontal_distance * yaw.cos();
        let offset_y = vertical_distance + camera_height_offset;
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
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        RelativeCursorPosition::default(),
    )).with_children(|parent| {
        parent.spawn((
            ImageNode::new(asset_server.load("crosshair.png")),
            Node {
                width: Val::Px(24.0),
                height: Val::Px(24.0),
                ..default()
            },
            Crosshair,
        ));
    });
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

fn shoot_gun(mouse: Res<ButtonInput<MouseButton>>, window: Single<&Window>, cam_q: Query<(&Camera, &GlobalTransform)>, gun_q: Query<&GlobalTransform, With<GunBarrel>>, sq: SpatialQuery, mut bots: Query<&mut BotData>, crosshair: Res<FloatingCrosshair>) {
    if !mouse.just_pressed(MouseButton::Left) { return; }
    let (Ok((cam, cam_tf)), Ok(gun_tf)) = (cam_q.single(), gun_q.single()) else { return; };
    let Ok(ray) = cam.viewport_to_world(cam_tf, Vec2::new(window.width() / 2.0, window.height() / 2.0) + crosshair.0) else { return; };
    let target = sq.cast_ray(ray.origin, ray.direction, 1000.0, true, &SpatialQueryFilter::default()).map_or(ray.get_point(1000.0), |hit| ray.get_point(hit.distance));
    let Ok(gun_dir) = Dir3::new(target - gun_tf.translation()) else { return; };
    if let Some(hit) = sq.cast_ray(gun_tf.translation(), gun_dir, 1000.0, true, &SpatialQueryFilter::default()) { if let Ok(mut bot) = bots.get_mut(hit.entity) { bot.health -= 1; println!("Hit! Bot HP: {}", bot.health); } }
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
        .add_systems(Update, (player_movement, setup_scene_once_loaded, movement_animations, camera_positioning, setup_lighting, bot_handling, cursor_handling, shoot_gun))
        .run();
}