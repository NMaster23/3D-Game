use bevy::{color::palettes::css, core_pipeline::tonemapping::Tonemapping, input::mouse::AccumulatedMouseMotion, post_process::bloom::Bloom, prelude::*, render::{render_resource::AsBindGroup, view::Hdr}, ui::RelativeCursorPosition, window::{CursorGrabMode, CursorOptions, WindowResolution}};
use avian3d::prelude::*;
use std::{collections::HashMap, ops::{Deref, DerefMut}, time::Duration};
use rand::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_hanabi::*;
use std::thread::sleep;
use std::io;

#[derive(Component)]
pub struct Lighting;

#[derive(Component)]
pub struct IsBot;

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct JumpIndicator {
    #[uniform(0)]
    pub progress: f32,
    #[uniform(0)]
    pub color: LinearRgba,
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone, Component)]
pub struct HealthBarUI {
    #[uniform(0)]
    pub progress: f32,
    #[uniform(0)]
    pub color: LinearRgba,
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone, Component)]
pub struct MuzzleFlash {
    #[uniform(0)]
    pub power: f32,
    #[uniform(0)]
    pub color: LinearRgba,
}

#[derive(Component)]
struct Crosshair;

#[derive(Component)]
struct Bots;

#[derive(Component)]
struct BottomThrusterLeft;

#[derive(Component)]
struct BottomThrusterRight;

#[derive(Resource, Default)]
struct TerrainGen {
    terrain: Handle<Scene>,
    loading_collision: Option<Entity>,
}

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
struct Player;

#[derive(Component)]
pub struct PlayerData {
    health: i32,
    player_name: String,
    player_id: u32,
    jumps: u32,
    jump_timer: Timer,
}

#[derive(Asset, TypePath, Debug, Clone)]
struct WeaponData {
    id: u32,
    name: String,
    damage: i32,
    range: f32,
    fire_rate: f32,
    power: f32,
}

#[derive(Resource, Default)]
struct SelectedWeapon {
    pub id: u32,
}

#[derive(Resource)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    graph_handle: Handle<AnimationGraph>,
}

impl UiMaterial for JumpIndicator {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shaders/jump_indicator.wgsl".into()
    }
}

impl UiMaterial for HealthBarUI {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shaders/health_bar.wgsl".into()
    }
}

impl UiMaterial for MuzzleFlash {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shaders/muzzle_flash.wgsl".into()
    }
}

impl Deref for FloatingCrosshair {
    type Target = Vec2;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for FloatingCrosshair {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

const MAX_BOUNCES: usize = 2;

fn ray_handling(ray_pos: Vec3, ray_dir: Dir3, time: Res<Time>, mut ray_cast: MeshRayCast, gizmos: &mut Gizmos, mut bot_query: Query<&mut BotData>, parents: Query<&ChildOf>) {
    let mut ray = Ray3d::new(ray_pos, ray_dir);
    let mut intersections = Vec::with_capacity(MAX_BOUNCES + 1);
    intersections.push((ray.origin, Color::srgb(30.0, 0.0, 0.0)));
    let color = Color::from(css::RED);
    let mut total_length = 0.0;
    for i in 0..MAX_BOUNCES {
        let Some((entity, hit)) = ray_cast
            .cast_ray(ray, &MeshRayCastSettings::default())
            .first()
        else {
            break;
        };
        total_length += hit.distance;
        if total_length > 10.0 {
            break;
        }
        let brightness = 1.0 + 10.0 * (1.0 - i as f32 / MAX_BOUNCES as f32);
        intersections.push((hit.point, color.mix(&color, brightness)));
        ray.direction = Dir3::new(ray.direction.reflect(hit.normal)).unwrap();
        ray.origin = hit.point + ray.direction * 1e-6;
        let mut current_entity = *entity;
        let mut dir_y: f32 = 0.0;
        loop {
            if let Ok(parent) = parents.get(current_entity) {
                current_entity = parent.0;
            } else {
                break;
            }
            if let Ok(mut bot_data) = bot_query.get_mut(current_entity) {
                bot_data.health -= 1;
                println!("Health -1")
            }
            dir_y = 100.0 * dir_y.sin() - 15.0 * time.delta_secs();
        }
    }
    gizmos.linestrip_gradient(intersections);
}

fn botdead(mut query: Query<(&BotData, &mut Transform), Changed<BotData>>) {
    for (botdata, mut transform) in query.iter_mut() {
        if botdata.health < 0 {
            transform.rotation = Quat::from_rotation_x(90.0f32.to_radians());
            transform.translation.y = 0.5;
        }
    }
}

fn jump_indicator(mut commands: Commands, mut materials: ResMut<Assets<JumpIndicator>>) {
    commands.spawn((
        MaterialNode(materials.add(JumpIndicator {
            progress: 1.0,
            color: LinearRgba::new(1.0, 1.0, 1.0, 0.75),
        })),
        Node {
            width: Val::Px(1000.0),
            height: Val::Px(1000.0),
            position_type: PositionType::Absolute,
            bottom: Val::Px(50.0),
            right: Val::Px(30.0),
            ..Default::default()
        },
    ));
}

fn health_bar(mut commands: Commands, mut materials: ResMut<Assets<HealthBarUI>>) {
    commands.spawn((
        MaterialNode(materials.add(HealthBarUI {
            progress: 1.0,
            color: LinearRgba::new(0.2, 0.8, 0.2, 1.0),
        })),
        Node {
            width: Val::Px(1000.0),
            height: Val::Px(1000.0),
            position_type: PositionType::Absolute,
            bottom: Val::Px(50.0),
            left: Val::Px(30.0),
            ..Default::default()
        },
        HealthBarUI {
            progress: 1.0,
            color: LinearRgba::new(0.2, 0.8, 0.2, 1.0),
        },
    ));
}

fn health_bar_handling(mut commands: Commands, mut materials: ResMut<Assets<HealthBarUI>>, query: Query<&PlayerData, With<Player>>) {
    if let Ok(player) = query.single() {
        for (_, material) in materials.iter_mut() {
            material.progress = (player.health as f32 / 5.0).clamp(0.0, 1.0);
            material.color = LinearRgba::new(0.2, 0.8, 0.2, 1.0);
        }
    }
}

fn jump_indicator_handling(time: Res<Time>, mut materials: ResMut<Assets<JumpIndicator>>, query: Query<&PlayerData, With<Player>>) {
    if let Ok(player) = query.single() {
        for (_, material) in materials.iter_mut() {
            if player.jumps == 0 {
                material.progress = player.jump_timer.fraction();
                material.color = LinearRgba::new(0.6, 0.6, 0.6, 0.8); // Dimmer white while recharging
            } else {
                material.progress = player.jumps as f32 / 2.0;
                material.color = LinearRgba::new(1.0, 1.0, 1.0, 0.9); // Bright white when ready
            }
        }
    }
}

fn gun_select_setup(mut weapons: ResMut<Assets<WeaponData>>, mut commands: Commands) {
    weapons.add(WeaponData {
        id: 0,
        name: "Pistol".into(),
        damage: 10,
        range: 50.0,
        fire_rate: 0.5,
        power: 1.0,
    });
    weapons.add(WeaponData {
        id: 1,
        name: "Rifle".into(),
        damage: 5,
        range: 100.0,
        fire_rate: 0.1,
        power: 0.5,
    });
    weapons.add(WeaponData {
        id: 2,
        name: "Shotgun".into(),
        damage: 20,
        range: 30.0,
        fire_rate: 1.0,
        power: 2.0,
    });
    weapons.add(WeaponData {
        id: 3,
        name: "Missile Launcher".into(),
        damage: 50,
        range: 200.0,
        fire_rate: 2.0,
        power: 5.0,
    });
}

fn gun_select_handling(mut selected: ResMut<SelectedWeapon>, keycode: Res<ButtonInput<KeyCode>>) {
    if keycode.just_pressed(KeyCode::Digit1) {
        selected.id = 0;
        println!("Selected Pistol");
    }
    if keycode.just_pressed(KeyCode::Digit2) {
        selected.id = 1;
        println!("Selected Rifle");
    }
    if keycode.just_pressed(KeyCode::Digit3) {
        selected.id = 2;
        println!("Selected Shotgun");
    }
    if keycode.just_pressed(KeyCode::Digit4) {
        selected.id = 3;
        println!("Selected Missile Launcher");
    }
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
        asset_server.load(GltfAssetLabel::Animation(0).from_asset("Player/Player.glb")),
    ]);
    let graph_handle = graphs.add(graph);
    commands.insert_resource(Animations {
        animations: node_indices,
        graph_handle,
    });
    let player_model = asset_server.load(GltfAssetLabel::Scene(0).from_asset("Player/Player.glb"));
    commands.spawn((
        GlobalTransform::default(),
        Player,
        RigidBody::Dynamic,
        SceneRoot(player_model),
        Collider::capsule(1.0, 1.5),
        Transform::from_xyz(0.0, 10.0, 0.0),
        PlayerData {
            health: 100,
            player_name: "Admin".into(),
            player_id: 1,
            jumps: 2,
            jump_timer: Timer::from_seconds(1.0, TimerMode::Once)
        },
        CharacterController {
            move_direction: Vec3::ZERO,
        },
        LockedAxes::ROTATION_LOCKED
    ));
}

fn bot_spawn(mut commands: Commands, asset_server: Res<AssetServer>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let bot_number = 10;
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
            Collider::capsule(1.0, 1.5),
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("Player/Player.glb"))),
            BotData {
                health: hits,
                bot_id: i + 1,
                bot_quantity: bots.bot_quantity,
                bot_offset: i as f32 * 2.0 - (bots.bot_quantity as f32 - 1.0) * 2.0 / 2.0,
                hit_number: hits_num,
            },
            IsBot,
            Transform::from_xyz(bots.bot_offset, 10.0, -5.0),
            CharacterController {
                move_direction: Vec3::ZERO,
            },
            LockedAxes::ROTATION_LOCKED,
        ));
    }
}

fn bot_handling(
    time: Res<Time>,
    mut q: Query<(Entity, &mut Transform, &mut CharacterController, &BotData, &mut LinearVelocity), (With<Bots>, Without<Player>)>,
    mut p: Query<(&Transform, &mut PlayerData), With<Player>>,
) {
    let Ok((pt, mut pd)) = p.single_mut() else { return; };
    let pos: Vec<_> = q.iter().map(|(e, t, _, _, _)| (e, t.translation)).collect();
    for (e, mut t, mut c, b, mut lv) in q.iter_mut() {
        if b.health >= 0 {
            let dir = (pt.translation - t.translation).normalize_or_zero();
            let sep: Vec3 = pos.iter().filter(|(oe, _)| e != *oe).filter_map(|(_, ot)| {
                let d = t.translation.distance(*ot);
                (d > 0.0 && d < 2.0).then(|| (t.translation - *ot).normalize_or_zero() * (1.0 - d / 2.0))
            }).sum();
            let f_dir = (dir + sep * 1.5).normalize_or_zero();
            c.move_direction = f_dir * 2.5;
            if f_dir.length_squared() > 0.0 {
                t.rotation = t.rotation.slerp(Quat::from_rotation_y(f_dir.x.atan2(f_dir.z)), time.delta_secs() * 5.0);
            }
            let rand_number = rand::rng().random_range(1..500);            
            lv.x = c.move_direction.x + rand::rng().random_range(-5.0..5.0);
            lv.z = c.move_direction.z + rand::rng().random_range(-5.0..5.0);
            if rand_number == b.hit_number { pd.health -= 1; }
        } else {
            println!("Dead")
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
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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

fn player_movement(time: Res<Time>, keyboard_input: Res<ButtonInput<KeyCode>>, mut query: Query<(&Transform, &mut LinearVelocity, &mut PlayerData, &mut CharacterController), With<Player>>) {
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
            if player.jumps == 0 {
                player.jump_timer.tick(time.delta());
                if player.jump_timer.just_finished() {
                    player.jumps = 2;
                    player.jump_timer.reset();
                }
            }
            if player.jumps > 0 {
                linear_velocity.y = 10.0;
                player.jumps -= 1;
            }
        }
        
        let velocity = (transform.rotation * move_direction).normalize_or_zero() * 5.0;
        linear_velocity.x = velocity.x;
        linear_velocity.z = velocity.z;

        controller.move_direction = (linear_velocity.x, 0.0, linear_velocity.z).into();
    }
}

fn camera_positioning(mut query: Query<&mut Node, With<Crosshair>>, mut crosshair_offset: ResMut<FloatingCrosshair>, mouse_button: Res<ButtonInput<MouseButton>>, mouse_movement: Res<AccumulatedMouseMotion>, mut player_data: Query<&mut Transform, With<Player>>, mut camera_data: Query<&mut Transform, (With<Camera3d>, Without<Player>)>, mut rotation: Local<Vec2>) {
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
    rotation.y = rotation.y.clamp(-14.9, 89.9);
    **crosshair_offset += mouse_movement.delta * 0.5;
    **crosshair_offset = crosshair_offset.lerp(Vec2::ZERO, 0.02);
    **crosshair_offset = crosshair_offset.clamp(Vec2::splat(-150.0), Vec2::splat(150.0));
    if let Ok(mut node) = query.single_mut() {
        node.left = Val::Px(crosshair_offset.x);
        node.top = Val::Px(crosshair_offset.y - 100.0);
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
    mut terrain_gen: ResMut<TerrainGen>
) {
    let floor_id = commands.spawn((
        Collider::cuboid(100.0, 1.0, 100.0),
        RigidBody::Static,
        Transform::from_xyz(0.0, -5.0, 0.0)
    )).id();
    terrain_gen.loading_collision = Some(floor_id);
    let terrain = asset_server.load(GltfAssetLabel::Scene(0).from_asset("Environment/Terrain.glb"));
    terrain_gen.terrain = terrain.clone();
    commands.spawn((
        SceneRoot(terrain),
        RigidBody::Static,
        Transform::from_xyz(0.0, -10.0, 0.0).with_scale(Vec3::splat(2000.0)),
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh)
    ));
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
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        Bloom::NATURAL,
        Hdr,
        Tonemapping::None,
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: true,
            ..default()
        },
    ));
    let sky = asset_server.load(GltfAssetLabel::Scene(0).from_asset("Environment/Sky.glb"));
    commands.spawn((
        SceneRoot(sky),
        Transform::from_scale(Vec3::splat(200.0)),
    ));
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::FlexEnd,
        justify_content: JustifyContent::Center,
        padding: UiRect::bottom(Val::Px(40.0)),
        ..default()
    });
}

fn particle_effects_setup(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>, player_query: Query<Entity, Added<Player>>) {
    let Ok(player) = player_query.single() else {
        return;
    };
    let mut gradient = bevy_hanabi::Gradient::new();
    gradient.add_key(0.0, Vec4::new(0.9, 0.98, 1.0, 1.0));
    gradient.add_key(0.15, Vec4::new(0.0, 0.843, 1.0, 1.0));
    gradient.add_key(0.6, Vec4::new(0.0, 0.2, 0.7, 0.5));
    gradient.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let mut size_tapering = bevy_hanabi::Gradient::new();
    size_tapering.add_key(0.0, Vec3::splat(0.2));
    size_tapering.add_key(0.5, Vec3::splat(0.075));
    size_tapering.add_key(1.0, Vec3::splat(0.0));
    let mut module = Module::default();
    let accel = module.lit(Vec3::new(0., -3., 0.));
    let update_accel = AccelModifier::new(accel);
    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed: module.lit(1.5),
    };
    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(0.15),
        dimension: ShapeDimension::Surface,
    };
    let lifetime = module.lit(0.1); // literal value "0.1"
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
    let bottom_thruster = effects.add(
        EffectAsset::new(63000, SpawnerSettings::rate(1000.0.into()), module)
            .init(init_pos)
            .init(init_vel)
            .init(init_lifetime)
            .render(ColorOverLifetimeModifier {
                gradient: gradient,
                ..Default::default()
            })
            .render(SizeOverLifetimeModifier {
                gradient: size_tapering,
                screen_space_size: false,
                ..Default::default()
            })
            .update(update_accel)
    );
    let thruster_left = commands.spawn((
        Name::new("Thruster Left"),
        ParticleEffect::new(bottom_thruster.clone()),
        Transform::from_xyz(-0.3, -1.5, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        BottomThrusterLeft,
    )).id();
    let thruster_right = commands.spawn((
        Name::new("Thruster Right"),
        ParticleEffect::new(bottom_thruster),
        Transform::from_xyz(0.3, -1.5, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        BottomThrusterRight,
    )).id();
    commands
        .entity(player)
        .add_children(&[thruster_left, thruster_right]);
}

fn particle_effects(keycode: Res<ButtonInput<KeyCode>>, mut spawners: Query<&mut EffectSpawner, Or<(With<BottomThrusterLeft>, With<BottomThrusterRight>)>>) {
    let jumping = keycode.just_pressed(KeyCode::Space);
    for spawner in &mut spawners {
        spawner.with_active(jumping);
        sleep(Duration::from_millis(100));
        spawner.with_active(!jumping);
    }
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

fn mesh_load_check(mut commands: Commands, mut events: MessageReader<AssetEvent<Scene>>, mut terrain_gen: ResMut<TerrainGen>) {
    let terrain_id = terrain_gen.terrain.id();
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            if *id == terrain_id {
                if let Some(entity) = terrain_gen.loading_collision {
                    commands.entity(entity).despawn();
                    terrain_gen.loading_collision = None;
                }
            }
        }
    }
}

fn shooting(window: Single<&Window>, camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>, mouse_button: Res<ButtonInput<MouseButton>>, mut crosshair: ResMut<FloatingCrosshair>, mut player_query: Query<&mut Transform, With<Player>>, mut gizmos: Gizmos, mut query: Query<&mut BotData>, time: Res<Time>, mut ray_cast: MeshRayCast, parent: Query<&ChildOf>) {
    if !mouse_button.pressed(MouseButton::Left) { return; }
    let Ok(mut player_transform) = player_query.single_mut() else {
        return;
    };
    let in_screen_pos = Vec2::new(window.width() / 2.0 + crosshair.x, window.height() / 2.0 + crosshair.y - 100.0,);
    let (inner_camera, camera_transform) = *camera;
    let Ok(camera_ray) = inner_camera.viewport_to_world(camera_transform, in_screen_pos) else { return; };
    let target = if let Some((_, hit)) = ray_cast.cast_ray(camera_ray, &MeshRayCastSettings::default()).first() {
        hit.point
    } else {
        camera_ray.origin + *camera_ray.direction * 100.0
    };
    let crosshair_pos = Vec3::new(crosshair.x, 0.0, crosshair.y);
    crosshair.y -= 200.0;
    let forward = -player_transform.forward();
    let ray_pos = player_transform.translation + *forward * 2.25;
    let dir = Dir3::new(target - ray_pos).unwrap_or(camera_ray.direction);
    ray_handling(ray_pos, dir, time, ray_cast, &mut gizmos, query, parent);
}

fn muzzle_flash(entity: Entity, mut commands: Commands, mut materials: ResMut<Assets<MuzzleFlash>>) {
    let flash = materials.add(MuzzleFlash {
        power: 1.0,
        color: LinearRgba::new(1.0, 0.8, 0.5, 1.0),
    });
    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            MaterialNode(flash),
            Node {
                width: Val::Px(200.0),
                height: Val::Px(200.0),
                position_type: PositionType::Absolute,
                left: Val::Px(-100.0),
                top: Val::Px(-100.0),
                ..default()
            },
        ));
    });
}

fn main() {
    App::new() 
        .add_plugins(EmbeddedAssetPlugin {
            mode: bevy_embedded_assets::PluginMode::ReplaceDefault,
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Mech Game".into(),
                resolution: WindowResolution::new(1920, 1080),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(UiMaterialPlugin::<JumpIndicator>::default())
        .add_plugins(UiMaterialPlugin::<HealthBarUI>::default())
        .add_plugins(HanabiPlugin)
        .init_asset::<WeaponData>()
        .init_resource::<TerrainGen>()
        .init_resource::<FloatingCrosshair>()
        .init_resource::<SelectedWeapon>()
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity(Vec3::new(0.0, -15.0, 0.0))) 
        .add_systems(Startup, (spawn_player, setup, gun_select_setup))
        .add_systems(Startup, (bot_spawn, jump_indicator, health_bar))
        .add_systems(Update, (player_movement, setup_scene_once_loaded, movement_animations, camera_positioning, setup_lighting, bot_handling, cursor_handling, mesh_load_check, shooting, botdead, particle_effects_setup, particle_effects, jump_indicator_handling, health_bar_handling, gun_select_handling, muzzle_flash))
        .run();
}