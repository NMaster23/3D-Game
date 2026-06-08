use bevy::{anti_alias::taa::TemporalAntiAliasing, camera::Exposure, color::palettes::css, core_pipeline::{Skybox, prepass::{DepthPrepass, MotionVectorPrepass, NormalPrepass}, tonemapping::Tonemapping}, input::mouse::AccumulatedMouseMotion, light::{CascadeShadowConfigBuilder, VolumetricFog, VolumetricLight}, math::VectorSpace, pbr::{ScreenSpaceAmbientOcclusion, ScreenSpaceAmbientOcclusionQualityLevel, ScreenSpaceReflections}, post_process::{bloom::Bloom, dof::{DepthOfField, DepthOfFieldMode}, motion_blur::MotionBlur}, prelude::*, render::{RenderPlugin, camera::{self, TemporalJitter}, render_resource::AsBindGroup, settings::{RenderCreation, WgpuLimits, WgpuSettings}, view::Hdr}, ui::RelativeCursorPosition, window::{CursorGrabMode, CursorOptions, WindowResolution}};
use avian3d::prelude::*;
use std::{ops::{Deref, DerefMut}, time::Duration};
use rand::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use std::f32::consts::PI;
use bevy_hanabi::prelude::*;
use bevy_hanabi::AlphaMode;
use bevy_flair::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_symbios_ground::{
    FbmNoise, GroundMaterialSettings, HeightMap, HeightMapMeshBuilder, NormalMethod, TerrainGenerator, build_heightfield_collider, splat_to_image, SplatMapper, SplatTexture,
};

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
pub struct SprintBarUI {
    #[uniform(0)]
    pub progress: f32,
    #[uniform(0)]
    pub color: LinearRgba,
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone, Component)]
pub struct ProjectileFlash {
    #[uniform(0)]
    pub power: f32,
    #[uniform(0)]
    pub color: LinearRgba,
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone, Component)]
pub struct ShotIndicator {
    #[uniform(0)]
    pub health_or: u32,
    #[uniform(0)]
    pub color: LinearRgba,
    #[uniform(0)]
    pub shot: f32,
    #[uniform(0)]
    pub magazine: f32,
}

#[derive(Component)]
struct Crosshair;

#[derive(Component)]
struct Bots;

#[derive(Component)]
struct BottomThrusterLeft;

#[derive(Component)]
struct BottomThrusterRight;

#[derive(Component)]
struct DashThruster;

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
    fire_timer: Timer,
    footstep_timer: Timer,
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
    footstep_timer: Timer,
    dash_timer: Timer,
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

#[derive(Resource)]
struct SelectedWeapon {
    pub id: u32,
}

#[derive(Component)]
struct MainMenuUi;

#[derive(Component)]
struct StartButton;

#[derive(Component)]
struct SettingsButton;

#[derive(Component)]
struct ApplySettingsButton;

#[derive(Resource)]
struct BotConfig {
    pub accuracy_range: std::ops::Range<i32>,
    pub disruptor_timer: Timer,
    pub is_disrupted: bool,
}

#[derive(Component)]
struct DespawnTimer(Timer);

#[derive(Resource)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    graph_handle: Handle<AnimationGraph>,
}

#[derive(Component)]
struct ProjectileFlashEffect(pub u32);

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone, Component)]
pub struct WeaponSelectorUI {
    #[uniform(0)]
    pub selected_weapon: u32,
    #[texture(1)]
    #[sampler(2)]
    pub weapon1: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub weapon2: Handle<Image>,
    #[texture(5)]
    #[sampler(6)]
    pub weapon3: Handle<Image>,
}

#[derive(Resource)]
pub struct LivingBots(pub u32);

#[derive(Resource)]
struct ScreenShake {
    strength: f32,
}

#[derive(Resource, Component)]
struct Hitmarker;

#[derive(Resource)]
struct HitmarkerTimer(Timer);

#[derive(Resource)]
struct CrosshairSpread {
    spread: f32,
}

#[derive(Resource)]
struct ImpactEffects {
    spark_effect: Handle<EffectAsset>,
    arc_effect: Handle<EffectAsset>,
}

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
}

#[derive(Resource)]
struct CycleMenu {
    pub options: Vec<String>,
    pub index: usize,
}

#[derive(Component)]
struct CycleTextTarget;

#[derive(Resource)]
struct PlayerModel {
    model_name: String,
}

#[derive(Resource)]
struct SsaoEnabled {
    pub enabled: bool,
}

#[derive(Resource, Default)]
struct AimDistance(f32);

#[derive(Resource)]
struct CameraDashEffect {
    active_multiplier: f32,
}

#[derive(Component)]
struct PrevOptionButton;

#[derive(Component)]
struct NextOptionButton;

#[derive(Resource, Clone, Copy, PartialEq)]
pub enum TerrainAlgorithm {
    AreaWeighted,
    Sobel,
}

#[derive(Resource)]
pub struct CurrentHeightMap(pub HeightMap);

impl Default for SsaoEnabled {
    fn default() -> Self {
        Self { enabled: false }
    }
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

impl UiMaterial for SprintBarUI {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shaders/sprint_bar.wgsl".into() // Reuse the health bar shader for our sprint bar
    }
}

impl UiMaterial for WeaponSelectorUI {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shaders/weapon_selector.wgsl".into()
    }
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            accuracy_range: 1..15,
            disruptor_timer: Timer::from_seconds(5.0, TimerMode::Once),
            is_disrupted: false,
        }
    }
}

impl Default for SelectedWeapon {
    fn default() -> Self {
        Self { id: 1 }
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

fn setup_impact_effects(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let mut writer_spark = ExprWriter::new();
    let mut gradient_spark = bevy_hanabi::Gradient::new();
    gradient_spark.add_key(0.0, Vec4::new(1.0, 0.9, 0.6, 1.0));
    gradient_spark.add_key(0.3, Vec4::new(1.0, 0.4, 0.1, 0.8));
    gradient_spark.add_key(1.0, Vec4::new(0.2, 0.1, 0.0, 0.0));
    let mut size_spark = bevy_hanabi::Gradient::new();
    size_spark.add_key(0.0, Vec3::splat(0.12));
    size_spark.add_key(0.2, Vec3::splat(0.08));
    size_spark.add_key(1.0, Vec3::splat(0.0));
    let mut module = Module::default();
    let accel = module.lit(Vec3::new(0., -10., 0.));
    let update_accel = AccelModifier::new(accel);
    let init_pos = SetPositionSphereModifier {
        center: writer_spark.lit(Vec3::ZERO).expr(),
        radius: writer_spark.lit(0.1).expr(), 
        dimension: ShapeDimension::Surface, 
    };
    let init_vel = SetVelocitySphereModifier {
        center: writer_spark.lit(Vec3::Y).expr(),
        speed: writer_spark.lit(10.0).expr(),
    };
    let init_life = SetAttributeModifier::new(
        Attribute::LIFETIME,
        writer_spark.lit(0.5).expr()
    );
    let module = writer_spark.finish();
    let spark_effect = effects.add(
        EffectAsset::new(1000, SpawnerSettings::once(1000.0.into()), module)
            .with_simulation_space(SimulationSpace::Local)
            .with_alpha_mode(AlphaMode::Add)
            .init(init_pos)
            .init(init_life)
            .init(init_vel)
            .update(update_accel)
            .render(ColorOverLifetimeModifier {
                gradient: gradient_spark,
                ..Default::default()
            })
            .render(SizeOverLifetimeModifier {
                gradient: size_spark,
                screen_space_size: false,
                ..Default::default()
            })
    );
    let writer_arc = ExprWriter::new();
    let mut gradient_arc = bevy_hanabi::Gradient::new();
    gradient_arc.add_key(0.0, Vec4::new(1.0, 0.9, 0.6, 1.0));
    gradient_arc.add_key(0.3, Vec4::new(1.0, 0.4, 0.1, 0.8));
    gradient_arc.add_key(1.0, Vec4::new(0.2, 0.1, 0.0, 0.0));
    let mut size_arc = bevy_hanabi::Gradient::new();
    size_arc.add_key(0.0, Vec3::splat(0.16));
    size_arc.add_key(0.1, Vec3::splat(0.24));
    size_arc.add_key(0.3, Vec3::splat(0.0));
    size_arc.add_key(1.0, Vec3::splat(0.0));
    let mut module = Module::default();
    let accel = module.lit(Vec3::new(0., -10., 0.));
    let update_accel = AccelModifier::new(accel);
    let init_pos_arc = SetPositionSphereModifier {
        center: writer_arc.lit(Vec3::ZERO).expr(),
        radius: writer_arc.lit(0.1).expr(), 
        dimension: ShapeDimension::Surface, 
    };
    let init_vel_arc = SetVelocitySphereModifier {
        center: writer_arc.lit(Vec3::Y).expr(),
        speed: writer_arc.lit(10.0).expr(),
    };
    let init_life_arc = SetAttributeModifier::new(
        Attribute::LIFETIME,
        writer_arc.lit(0.5).expr()
    );
    let module = writer_arc.finish();
    let arc_effect = effects.add(
        EffectAsset::new(2000, SpawnerSettings::once(2000.0.into()), module)
            .with_simulation_space(SimulationSpace::Local)
            .with_alpha_mode(AlphaMode::Add)
            .init(init_pos_arc)
            .init(init_life_arc)
            .init(init_vel_arc)
            .update(update_accel)
            .render(ColorOverLifetimeModifier {
                gradient: gradient_arc,
                ..Default::default()
            })
            .render(SizeOverLifetimeModifier {
                gradient: size_arc,
                screen_space_size: false,
                ..Default::default()
            })
    );
    commands.insert_resource(ImpactEffects {
        spark_effect,
        arc_effect,
    });
}

fn ray_handling(audio: Res<Audio>, asset_server: Res<AssetServer>, current_weapon: u32, impact_effects: Res<ImpactEffects>, mut commands: Commands, mut timer: ResMut<HitmarkerTimer>, mut query: Query<&mut BackgroundColor, With<Hitmarker>>, ray_pos: Vec3, ray_dir: Dir3, damage: i32, max_range: f32, time: Res<Time>, mut ray_cast: MeshRayCast, gizmos: &mut Gizmos, mut bot_query: Query<&mut BotData>, parents: Query<&ChildOf>) {
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
        if total_length > max_range {
            break;
        }
        let brightness = 1.0 + 10.0 * (1.0 - i as f32 / MAX_BOUNCES as f32);
        intersections.push((hit.point, color.mix(&color, brightness)));
        ray.direction = Dir3::new(ray.direction.reflect(hit.normal)).unwrap();
        ray.origin = hit.point + ray.direction * 1e-6;
        let mut current_entity = *entity;
        loop {
            if let Ok(parent) = parents.get(current_entity) {
                current_entity = parent.0;
            } else {
                break;
            }
            if let Ok(mut bot_data) = bot_query.get_mut(current_entity) {
                let final_damage = (damage as f32 * (1.0 - total_length / max_range)).max(1.0) as i32;
                bot_data.health -= final_damage;
                timer.0.reset();
                if current_weapon == 2 {
                    let shot_sound = audio.play(asset_server.load("Player/Rocket_Explode.ogg")).handle();
                    commands.spawn((
                        Transform::from_translation(hit.point),
                        SpatialAudioEmitter {
                            instances: vec![shot_sound]
                        }
                    ));
                }
                println!("Bot hit! Damage: {}, Distance: {:.2}", final_damage, total_length);
                for mut color in &mut query {
                    color.0 = Color::srgba(1.0, 1.0, 1.0, 0.75);
                }
                commands.spawn((
                    ParticleEffect::new(impact_effects.spark_effect.clone()),
                    Transform::from_translation(hit.point),
                ));
                commands.spawn((
                    ParticleEffect::new(impact_effects.arc_effect.clone()),
                    Transform::from_translation(hit.point),
                ));
            }
        }
    }
    gizmos.linestrip_gradient(intersections);
}

fn botdead(player_data: Query<&mut Transform, With<Player>>, mut commands: Commands, mut query: Query<(Entity, &BotData, &mut Transform), (Changed<BotData>, Without<Player>)>, mut living_bots: ResMut<LivingBots>, materials: ResMut<Assets<StandardMaterial>>) {
    let Ok(player_transform) = player_data.single() else {
        return;
    };
    for (entity, botdata, mut transform) in query.iter_mut() {
        if botdata.health <= 0 {
            transform.rotation = Quat::from_rotation_x(90.0f32.to_radians());
            transform.translation.y = 0.5;
            commands.entity(entity)
                .remove::<BotData>()
                .insert(RigidBody::Static)
                .remove::<Collider>();
            living_bots.0 -= 1;
        }
        if living_bots.0 == 0 {
            println!("All bots defeated! You win!");
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
        ZIndex(10),
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
        ZIndex(10),
    ));
}

fn sprint_bar(mut commands: Commands, mut materials: ResMut<Assets<SprintBarUI>>) {
    commands.spawn((
        MaterialNode(materials.add(SprintBarUI {
            progress: 1.0,
            color: LinearRgba::new(0.2, 0.6, 1.0, 1.0), // Blue sprint bar
        })),
        Node {
            width: Val::Px(300.0),
            height: Val::Px(40.0),
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Percent(50.0),
            margin: UiRect::left(Val::Px(-150.0)),
            ..Default::default()
        },
        SprintBarUI {
            progress: 1.0,
            color: LinearRgba::new(0.2, 0.6, 1.0, 1.0),
        },
        ZIndex(10),
    ));
}

fn health_bar_handling(mut commands: Commands, mut materials: ResMut<Assets<HealthBarUI>>, query: Query<&PlayerData, With<Player>>) {
    if let Ok(player) = query.single() {
        for (_, material) in materials.iter_mut() {
            let progress = (player.health as f32 / 100.0).clamp(0.0, 1.0);
            material.progress = progress;
            let red = ((1.0 - progress) * 2.0).clamp(0.0, 1.0);
            let green = progress;
            material.color = LinearRgba::new(red, green, 0.1, 1.0);
        }
    }
}

fn sprint_bar_handling(mut materials: ResMut<Assets<SprintBarUI>>, query: Query<&PlayerData, With<Player>>) {
    if let Ok(player) = query.single() {
        for (_, material) in materials.iter_mut() {
            material.progress = player.dash_timer.fraction();
            material.color = LinearRgba::new(0.1, 0.6, 1.0, 1.0); // Light blue
        }
    }
}

fn jump_indicator_handling(time: Res<Time>, mut materials: ResMut<Assets<JumpIndicator>>, query: Query<&PlayerData, With<Player>>) {
    if let Ok(player) = query.single() {
        for (_, material) in materials.iter_mut() {
            if player.jumps == 0 {
                material.progress = player.jump_timer.fraction();
                material.color = LinearRgba::new(0.6, 0.6, 0.6, 0.8);
            } else {
                material.progress = player.jumps as f32 / 2.0;
                material.color = LinearRgba::new(1.0, 1.0, 1.0, 0.9);
            }
        }
    }
}

fn weapon_selector_setup(mut commands: Commands, mut materials: ResMut<Assets<WeaponSelectorUI>>, asset_server: Res<AssetServer>) { 
    let ui_material = WeaponSelectorUI { 
        selected_weapon: 1,
        weapon1: asset_server.load("Pistol.png"),
        weapon2: asset_server.load("Rifle.png"),
        weapon3: asset_server.load("Rocket.png"),
    };

    commands.spawn((
        MaterialNode(materials.add(ui_material.clone())),
        Node {
            width: Val::Px(1000.0),
            height: Val::Px(250.0),
            position_type: PositionType::Absolute,
            top: Val::Px(720.0),
            left: Val::Percent(50.0),
            margin: UiRect::left(Val::Px(-500.0)),
            ..Default::default()
        },
        ui_material,
        ZIndex(10),
    ));
}

fn weapon_selector(keycode: Res<ButtonInput<KeyCode>>, mut selected_weapon: ResMut<SelectedWeapon>, mut materials: ResMut<Assets<WeaponSelectorUI>>) {
    let mut selection = None;
    if keycode.just_pressed(KeyCode::Digit1) {
        selection = Some(1);
        println!("Selected weapon: 1");
    }
    if keycode.just_pressed(KeyCode::Digit2) {
        selection = Some(2);
        println!("Selected weapon: 2");
    }
    if keycode.just_pressed(KeyCode::Digit3) {
        selection = Some(3);
        println!("Selected weapon: 3");
    }
    if let Some(id) = selection {
        selected_weapon.id = id;
        println!("Selected weapon: {}", id);
    }
    if selected_weapon.is_changed() {
        for (_, material) in materials.iter_mut() {
            material.selected_weapon = selected_weapon.id;
        }
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

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>, mut graphs: ResMut<Assets<AnimationGraph>>, player_model: Res<PlayerModel>) {
    let (graph, node_indices) = AnimationGraph::from_clips([
        asset_server.load(GltfAssetLabel::Animation(0).from_asset(player_model.model_name.clone())),
    ]);
    let graph_handle = graphs.add(graph);
    commands.insert_resource(Animations {
        animations: node_indices,
        graph_handle,
    });
    let player_model = asset_server.load(GltfAssetLabel::Scene(0).from_asset(player_model.model_name.clone()));
    commands.spawn((
        GlobalTransform::default(),
        Player,
        RigidBody::Dynamic,
        LinearDamping(2.0),
        SceneRoot(player_model),
        Collider::capsule(1.0, 0.5),
        Transform::from_xyz(0.0, 500.0, 0.0),
        PlayerData {
            health: 100,
            player_name: "None".into(),
            player_id: 1,
            jumps: 2,
            jump_timer: Timer::from_seconds(1.0, TimerMode::Once),
            footstep_timer: Timer::from_seconds(0.7, TimerMode::Repeating),
            dash_timer: {
                let mut timer = Timer::from_seconds(2.0, TimerMode::Once);
                timer.set_elapsed(Duration::from_secs(2));
                timer
            },
        },
        CharacterController {
            move_direction: Vec3::ZERO,
        },
        LockedAxes::ROTATION_LOCKED
    ));
}

fn bot_spawn(mut commands: Commands, asset_server: Res<AssetServer>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut effects: ResMut<Assets<EffectAsset>>) {
    let bot_number = 1;
    let mut rng = rand::rng();
    let hits = rng.random_range(75..150);
    let hits_num = rng.random_range(1..5);
    let mut bots = BotData {
        health: hits,
        bot_id: 1,
        bot_quantity: bot_number,
        bot_offset: 0.0,
        hit_number: hits_num,
        fire_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        footstep_timer: Timer::from_seconds(0.6, TimerMode::Repeating),
    };
    commands.insert_resource(LivingBots(bot_number));
    for i in 0..bots.bot_quantity {
        bots.bot_offset = i as f32 * bots.bot_quantity as f32 - 10.0;
        let bot = commands.spawn((
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
                fire_timer: Timer::from_seconds(rand::rng().random_range(1.5..2.0), TimerMode::Repeating),
                footstep_timer: Timer::from_seconds(0.7, TimerMode::Repeating),
            },
            IsBot,
            Transform::from_xyz(bots.bot_offset, 500.0, -5.0),
            CharacterController {
                move_direction: Vec3::ZERO,
            },
            LockedAxes::ROTATION_LOCKED,
        )).id();
    }
}

fn bot_handling(
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ProjectileFlash>>,
    mut ray_cast: MeshRayCast,
    mut gizmos: Gizmos,
    mut q: Query<(Entity, &mut Transform, &mut CharacterController, &mut BotData, &mut LinearVelocity), (With<Bots>, Without<Player>)>,
    mut p: Query<(&Transform, &mut PlayerData), With<Player>>,
    p_entity: Query<Entity, With<Player>>,
    parents: Query<&ChildOf>,
    mut effects: ResMut<Assets<EffectAsset>>,
    bot_config: Res<BotConfig>,
    audio: Res<Audio>,
) {
    let Ok((pt, mut pd)) = p.single_mut() else { return; };
    let pos: Vec<_> = q.iter().map(|(e, t, _, _, _)| (e, t.translation)).collect();
    for (e, mut t, mut c, mut b, mut lv) in q.iter_mut() {
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
                if b.footstep_timer.tick(time.delta()).just_finished() {
                    let footstep = audio.play(asset_server.load("Player/Footstep.ogg")).handle();
                    commands.spawn((
                        Transform::from_translation(t.translation),
                        SpatialAudioEmitter {
                            instances: vec![footstep]
                        }
                    ));
                }
            }
            lv.x = c.move_direction.x + rand::rng().random_range(-5.0..5.0);
            lv.z = c.move_direction.z + rand::rng().random_range(-5.0..5.0);
            let Ok(player_entity) = p_entity.single() else { continue; };
            if b.fire_timer.tick(time.delta()).just_finished() {
                let mut total_length = 0.0;
                let max_range = 100.0;
                let ray_dir = Dir3::new(pt.translation - t.translation).unwrap_or(Dir3::Z);
                let ray = Ray3d::new(t.translation + Vec3::new(0.0, 1.25, 0.0), ray_dir);
                let hits = ray_cast.cast_ray(ray, &MeshRayCastSettings::default());
                let hit_number = 1;
                let hit_chance = rand::rng().random_range(bot_config.accuracy_range.clone());
                if hit_number == hit_chance {
                    for (hit_entity, hit_data) in hits {
                        total_length += hit_data.distance;
                        if total_length > max_range {
                            break;
                        }
                        if hit_data.distance > 30.0 {
                            break;
                        }
                        let mut current_entity = *hit_entity;
                        let mut hit_self = false;
                        let mut hit_player = false;
                        loop {
                            if current_entity == player_entity {
                                hit_player = true;
                            }
                            if current_entity == e {
                                hit_self = true;
                                break;
                            }
                            if let Ok(parent) = parents.get(current_entity) {
                                current_entity = parent.0;
                            } else {
                                break;
                            }
                        }
                        if hit_self {
                            continue;
                        }
                        if hit_player {
                            gizmos.line(t.translation, hit_data.point, Color::srgb(1.0, 0.0, 0.0));
                            let damage = (15.0 * (1.0 - total_length / max_range)).max(1.0) as i32;
                            pd.health -= damage;
                            println!("Player hit! Health: {}, Distance: {:.2}", pd.health, hit_data.distance);
                            let flash = materials.add(ProjectileFlash {
                                power: 1.0,
                                color: LinearRgba::new(1.0, 0.8, 0.5, 1.0),
                            });
                        }
                        break;
                    }
                }
            }
        }
    }
}

fn procedural_terrain(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>, 
    algorithm: Res<TerrainAlgorithm>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut heightmap = HeightMap::new(1024, 1024, 1.0);
    let mut noise = FbmNoise::new(1337);
    noise.octaves = 6;
    noise.lacunarity = 2.2;
    noise.persistence = 0.45;
    noise.base_frequency = 0.75;
    noise.generate(&mut heightmap);
    for val in heightmap.data_mut().iter_mut() {
        *val = 1.0 - (*val * 2.0 - 1.0).abs();
        *val = val.powi(2);
    }
    heightmap.normalize();
    let width = heightmap.width();
    let height = heightmap.height();
    let mut slope_data = vec![0.0; width * height];
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let h_right = heightmap.get(x + 1, y);
            let h_left = heightmap.get(x - 1, y);
            let h_up = heightmap.get(x, y + 1);
            let h_down = heightmap.get(x, y - 1);
            let dx = h_right - h_left;
            let dy = h_up - h_down;
            let slope = (dx * dx + dy * dy).sqrt();
            slope_data[y * width + x] = slope.clamp(0.0, 1.0);
        }
    }
    let normal_algorithm = match *algorithm {
        TerrainAlgorithm::AreaWeighted => NormalMethod::AreaWeighted,
        TerrainAlgorithm::Sobel => NormalMethod::Sobel,
    };
    let collider = build_heightfield_collider(&heightmap);
    let mesh = HeightMapMeshBuilder::new()
        .with_normal_method(normal_algorithm)
        .with_uv_tile_size(2.0)
        .build(&heightmap);
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.28, 0.22), // A natural, dark earthy tone
            perceptual_roughness: 0.85, // High roughness so it doesn't look wet/shiny
            reflectance: 0.1, // Low reflectance
            ..default()
        })),
        Transform::from_xyz(-512.0, 0.0, -512.0).with_scale(Vec3::new(1.0, 300.0, 1.0)),
    ));
    commands.spawn((
        collider,
        RigidBody::Static,
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(1.0, 300.0, 1.0)),
    ));
    let weight_map = SplatMapper::default().generate(&heightmap);
    let image = splat_to_image(&weight_map);
    commands.insert_resource(SplatTexture {
        handle: images.add(image)
    });
    commands.insert_resource(GroundMaterialSettings::new(weight_map));
    commands.insert_resource(CurrentHeightMap(heightmap));
}

fn despawn_ray(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut DespawnTimer)>) {
    for (e, mut t) in q.iter_mut() {
        if t.0.tick(time.delta()).just_finished() {
            commands.entity(e).despawn();
        }
    }
}

fn targeting_disruptor(keycode: Res<ButtonInput<KeyCode>>, mut bot_config: ResMut<BotConfig>, time: Res<Time>) {
    if bot_config.is_disrupted {
        if bot_config.disruptor_timer.tick(time.delta()).just_finished() {
            bot_config.is_disrupted = false;
            bot_config.accuracy_range = 1..15;
            println!("Targeting systems restored.");
        }
    } else if keycode.just_pressed(KeyCode::KeyT) {
        bot_config.is_disrupted = true;
        bot_config.disruptor_timer.reset();
        bot_config.accuracy_range = 1..100;
        println!("Targeting systems disrupted!");
    }
}

fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();
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
    mut animation_players: Query<(Entity, &mut AnimationPlayer, &mut AnimationTransitions)>,
    parents: Query<&ChildOf>,
    player_query: Query<&Player>,
    bot_query: Query<&CharacterController, With<Bots>>,
) {
    for (entity, mut player, _transitions) in &mut animation_players {
        let Some((&playing_animation_index, _)) = player.playing_animations().next() else {
            continue;
        };
        let mut current_entity = entity;
        let mut is_moving = false;
        let mut speed = 1.0;
        
        loop {
            if player_query.contains(current_entity) {
                if keyboard_input.pressed(KeyCode::KeyW) {
                    is_moving = true;
                    speed = 1.0;
                } else if keyboard_input.pressed(KeyCode::KeyS) {
                    is_moving = true;
                    speed = -1.0;
                } else if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::KeyD) {
                    is_moving = true;
                }
                break;
            } else if let Ok(controller) = bot_query.get(current_entity) {
                if controller.move_direction.length_squared() > 0.01 {
                    is_moving = true;
                    speed = 1.0;
                }
                break;
            }
            if let Ok(parent) = parents.get(current_entity) {
                current_entity = parent.0;
            } else {
                break;
            }
        }
        
        let playing_animation = player.animation_mut(playing_animation_index).unwrap();
        if is_moving {
            playing_animation.set_speed(speed);
            playing_animation.resume();
        } else {
            playing_animation.pause();
        }
    }
}

fn player_movement(mut commands: Commands, mut camera_effect: ResMut<CameraDashEffect>, mut screen_shake: ResMut<ScreenShake>, mut dash_spawners: Query<&mut EffectSpawner, With<DashThruster>>, asset_server: Res<AssetServer>, mut audio: Res<Audio>, time: Res<Time>, keyboard_input: Res<ButtonInput<KeyCode>>, mut query: Query<(&Transform, &mut LinearVelocity, &mut PlayerData, &mut CharacterController), With<Player>>, mut spawners: Query<(&mut EffectSpawner, Option<&mut PointLight>), (Or<(With<BottomThrusterLeft>, With<BottomThrusterRight>)>, Without<DashThruster>)>) {
    for (transform, mut linear_velocity, mut player, mut controller) in query.iter_mut() {
        player.dash_timer.tick(time.delta());
        if player.jumps < 2 {
            player.jump_timer.tick(time.delta());
            if player.jump_timer.just_finished() {
                player.jumps += 1;
                player.jump_timer.reset();
            }
        } else {
            player.jump_timer.reset();
        }
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
            if player.jumps > 0 {
                linear_velocity.y = 10.0;
                player.jumps -= 1;
                for (mut spawner, _) in spawners.iter_mut() {
                    spawner.active = true;
                    spawner.reset();
                }
            }
        }
        if move_direction != Vec3::ZERO && player.health > 0 && player.footstep_timer.tick(time.delta()).is_finished() {
            audio.play(asset_server.load("Player/Footstep.ogg"));
        }
        for (_, light) in spawners.iter_mut() {
            if let Some(mut l) = light {
                if l.intensity > 0.01 {
                    l.intensity *= 0.001_f32.powf(time.delta_secs()); 
                } else {
                    l.intensity = 0.0;
                }
            }
        }
        let mut velocity = (transform.rotation * move_direction).normalize_or_zero() * 5.0;
        
        let is_dashing = player.dash_timer.elapsed_secs() < 0.2;
        
        if keyboard_input.just_pressed(KeyCode::ShiftLeft) && player.dash_timer.is_finished() && player.health > 0 {
            let dash_power = 45.0;
            if move_direction == Vec3::ZERO {
                move_direction.z = 1.0;
            }
            let dash_velocity = (transform.rotation * move_direction).normalize_or_zero() * dash_power;
            linear_velocity.x = dash_velocity.x;
            linear_velocity.z = dash_velocity.z;
            let dash_sound = audio.play(asset_server.load("Player/Rocket_Explode.ogg")).handle();
            commands.spawn((
                Transform::from_translation(transform.translation),
                SpatialAudioEmitter { instances: vec![dash_sound] }
            ));
            camera_effect.active_multiplier = 1.5;
            screen_shake.strength = 1.0;
            for mut spawner in dash_spawners.iter_mut() {
                spawner.active = true;
                spawner.reset();
            }
            player.dash_timer.reset();
        } else if !is_dashing {
            linear_velocity.x = velocity.x;
            linear_velocity.z = velocity.z;
        }

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
    crosshair_offset.0 += mouse_movement.delta * 0.5;
    crosshair_offset.0 = crosshair_offset.lerp(Vec2::ZERO, 0.02);
    crosshair_offset.0 = crosshair_offset.clamp(Vec2::splat(-150.0), Vec2::splat(150.0));
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

fn crosshair_spread(mut query: Query<&mut Node, With<Crosshair>>, time: Res<Time>, mut spread: ResMut<CrosshairSpread>) {
    spread.spread += (0.0 - spread.spread) * (3.0 * time.delta_secs()).min(1.0);
    for mut node in &mut query {
        node.width = Val::Px(24.0 + spread.spread);
        node.height = Val::Px(24.0 + spread.spread);
    }
}

fn setup(
    mut commands: Commands,
    graphics: Res<SsaoEnabled>,
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
    let sky = asset_server.load("Environment/Sky.ktx2");
    let mut camera = commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        Hdr,
        Msaa::Off,
        TemporalAntiAliasing::default(),
        TemporalJitter::default(),
        Tonemapping::TonyMcMapface,
        DepthPrepass,
        NormalPrepass,
        MotionVectorPrepass,
    ));
    camera.insert((
        VolumetricFog {
        ambient_color: Color::srgb(0.1, 0.1, 0.12),
        ambient_intensity: 0.1,
        step_count: 64,
        jitter: 0.5,
        ..default()
        },
        Skybox {
        image: sky.clone(),
        brightness: 350.0,
        ..Default::default()
        },
        SpatialAudioReceiver,
        Bloom {
        intensity: 0.15,
        low_frequency_boost: 0.5,
        high_pass_frequency: 1.0,
        ..Default::default()
        },
        Exposure {
        ev100: 10.0,
        },
    ));
    if graphics.enabled {
        camera.insert(ScreenSpaceAmbientOcclusion {
            quality_level: ScreenSpaceAmbientOcclusionQualityLevel::Ultra,
            constant_object_thickness: 0.2,
            ..default()
        })
        .insert(ScreenSpaceReflections::default())
        .insert(MotionBlur {
            shutter_angle: 0.5,
            samples: 4,
        })
        .insert(DepthOfField {
            mode: DepthOfFieldMode::Bokeh,
            focal_distance: 8.0,
            sensor_height: 0.024,
            aperture_f_stops: 2.8,
            max_circle_of_confusion_diameter: 64.0,
            max_depth: 1000.0,
        });
    }
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::FlexEnd,
        justify_content: JustifyContent::Center,
        padding: UiRect::bottom(Val::Px(40.0)),
        ..default()
    });
    commands.spawn((
        DirectionalLight {
            illuminance: 100_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -PI / 4.0, -PI / 4.0, 0.0)),
        CascadeShadowConfigBuilder {
            num_cascades: 4,
            minimum_distance: 0.1,
            maximum_distance: 1000.0,
            first_cascade_far_bound: 5.0,
            overlap_proportion: 0.2,
        }
        .build()
    ));
}

fn particle_effects(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>, player_query: Query<Entity, Added<Player>>, bot_query: Query<(Entity, &BotData), Added<IsBot>>) {
    let Ok(player) = player_query.single() else {
        return;
    };
    let mut writer_smoke = ExprWriter::new();
    let mut color_smoke = bevy_hanabi::Gradient::new();
    color_smoke.add_key(0.0, Vec4::new(1.2, 1.6, 2.0, 0.8)); 
    color_smoke.add_key(0.2, Vec4::new(0.3, 0.3, 0.35, 0.6));
    color_smoke.add_key(1.0, Vec4::new(0.1, 0.1, 0.1, 0.0));
    let mut size_smoke = bevy_hanabi::Gradient::new();
    size_smoke.add_key(0.0, Vec3::splat(0.2));
    size_smoke.add_key(1.0, Vec3::splat(1.5));
    let smoke_pos_center = writer_smoke.lit(Vec3::ZERO).expr();
    let smoke_pos_radius = writer_smoke.lit(0.05).expr();
    let smoke_vel_center = writer_smoke.lit(Vec3::new(0.0, 1.5, 0.0)).expr();
    let smoke_vel_speed = writer_smoke.lit(18.0).expr();
    let smoke_lifetime = writer_smoke.lit(1.2).expr();
    let smoke_accel = writer_smoke.lit(Vec3::new(0.0, -2.0, 0.0)).expr();
    let smoke_drag = writer_smoke.lit(3.0).expr();
    let smoke_module = writer_smoke.finish();
    let smoke_effect = effects.add(
        EffectAsset::new(32000, SpawnerSettings::once(150.0.into()), smoke_module)
            .with_simulation_space(SimulationSpace::Global)
            .with_alpha_mode(AlphaMode::Blend)
            .init(SetPositionSphereModifier {
                center: smoke_pos_center,
                radius: smoke_pos_radius,
                dimension: ShapeDimension::Volume,
            })
            .init(SetVelocitySphereModifier {
                center: smoke_vel_center,
                speed: smoke_vel_speed,
            })
            .init(SetAttributeModifier::new(Attribute::LIFETIME, smoke_lifetime))
            .update(AccelModifier::new(smoke_accel))
            .update(LinearDragModifier::new(smoke_drag))
            .render(ColorOverLifetimeModifier {
                gradient: color_smoke,
                ..Default::default()
            })
            .render(SizeOverLifetimeModifier {
                gradient: size_smoke,
                screen_space_size: false,
                ..Default::default()
            })
    );
    let mut writer_flame = ExprWriter::new();
    let mut color_flame = bevy_hanabi::Gradient::new();
    color_flame.add_key(0.0, Vec4::new(10.0, 10.0, 10.0, 1.0));
    color_flame.add_key(0.3, Vec4::new(0.0, 8.0, 10.0, 1.0));
    color_flame.add_key(1.0, Vec4::new(0.0, 0.2, 0.8, 0.0));

    let mut size_flame = bevy_hanabi::Gradient::new();
    size_flame.add_key(0.0, Vec3::splat(0.4));
    size_flame.add_key(1.0, Vec3::splat(0.0));

    let flame_pos_center = writer_flame.lit(Vec3::ZERO).expr();
    let flame_pos_radius = writer_flame.lit(0.02).expr();
    let flame_vel_center = writer_flame.lit(Vec3::new(0.0, 2.0, 0.0)).expr();
    let flame_vel_speed = writer_flame.lit(25.0).expr();
    let flame_lifetime = writer_flame.lit(0.3).expr();

    let flame_module = writer_flame.finish();
    let flame_effect = effects.add(
        EffectAsset::new(16000, SpawnerSettings::once(150.0.into()), flame_module)
            .with_simulation_space(SimulationSpace::Local)
            .with_alpha_mode(AlphaMode::Add)
            .init(SetPositionSphereModifier {
                center: flame_pos_center,
                radius: flame_pos_radius,
                dimension: ShapeDimension::Volume,
            })
            .init(SetVelocitySphereModifier {
                center: flame_vel_center,
                speed: flame_vel_speed,
            })
            .init(SetAttributeModifier::new(Attribute::LIFETIME, flame_lifetime))
            .render(ColorOverLifetimeModifier {
                gradient: color_flame,
                ..Default::default()
            })
            .render(SizeOverLifetimeModifier {
                gradient: size_flame,
                screen_space_size: false,
                ..Default::default()
            }
        )
    );

    let mut writer_dash = ExprWriter::new();
    let mut color_dash = bevy_hanabi::Gradient::new();
    // HDR Cyan/Blue fading to dark smoke
    color_dash.add_key(0.0, Vec4::new(0.0, 8.0, 10.0, 1.0)); 
    color_dash.add_key(0.2, Vec4::new(0.0, 1.0, 4.0, 0.8));
    color_dash.add_key(1.0, Vec4::new(0.05, 0.05, 0.05, 0.0));

    let mut size_dash = bevy_hanabi::Gradient::new();
    size_dash.add_key(0.0, Vec3::splat(0.8));
    size_dash.add_key(1.0, Vec3::splat(0.0));

    let dash_pos_center = writer_dash.lit(Vec3::ZERO).expr();
    let dash_pos_radius = writer_dash.lit(0.5).expr();
    let dash_vel_center = writer_dash.lit(Vec3::ZERO).expr();
    let dash_vel_speed = writer_dash.lit(40.0).expr();
    let dash_lifetime = writer_dash.lit(0.4).expr();
    let dash_drag = writer_dash.lit(8.0).expr();

    let dash_module = writer_dash.finish();
    let dash_effect = effects.add(
        EffectAsset::new(4000, SpawnerSettings::once(500.0.into()), dash_module) // 500 particles instantly
            .with_simulation_space(SimulationSpace::Global)
            .with_alpha_mode(AlphaMode::Add)
            .init(SetPositionSphereModifier {
                center: dash_pos_center,
                radius: dash_pos_radius,
                dimension: ShapeDimension::Volume,
            })
            .init(SetVelocitySphereModifier {
                center: dash_vel_center,
                speed: dash_vel_speed,
            })
            .init(SetAttributeModifier::new(Attribute::LIFETIME, dash_lifetime))
            .update(LinearDragModifier::new(dash_drag)) // High drag stops them quickly
            .render(ColorOverLifetimeModifier {
                gradient: color_dash,
                ..Default::default()
            })
            .render(SizeOverLifetimeModifier {
                gradient: size_dash,
                screen_space_size: false,
                ..Default::default()
            })
    );
    let dash_thruster = commands.spawn((
        Name::new("Dash Thruster"),
        ParticleEffect::new(dash_effect),
        EffectSpawner::default(), // Defaults to inactive/waiting
        Transform::from_xyz(0.0, 1.5, 0.0), 
        DashThruster,
    )).id();
    let left_thruster_smoke = commands.spawn((
        Name::new("Left Thruster Smoke"),
        ParticleEffect::new(smoke_effect.clone()),
        EffectSpawner::default(),
        Transform::from_xyz(-0.3, -1.0, 0.0), 
        BottomThrusterLeft,
    )).id();
    let right_thruster_smoke = commands.spawn((
        Name::new("Right Thruster Smoke"),
        ParticleEffect::new(smoke_effect.clone()),
        EffectSpawner::default(),
        Transform::from_xyz(0.3, -1.0, 0.0), 
        BottomThrusterRight,
    )).id();

    let left_thruster_flame = commands.spawn((
        Name::new("Left Thruster Flame"),
        ParticleEffect::new(flame_effect.clone()),
        EffectSpawner::default(),
        Transform::from_xyz(-0.3, -1.0, 0.0), 
        BottomThrusterLeft,
        PointLight {
            color: Color::srgb(0.0, 0.8, 1.0),
            intensity: 0.0,
            range: 15.0,
            ..Default::default()
        }
    )).id();
    let right_thruster_flame = commands.spawn((
        Name::new("Right Thruster Flame"),
        ParticleEffect::new(flame_effect.clone()),
        EffectSpawner::default(),
        Transform::from_xyz(0.3, -1.0, 0.0), 
        BottomThrusterRight,
        PointLight {
            color: Color::srgb(0.0, 0.8, 1.0),
            intensity: 0.0,
            range: 15.0,
            ..Default::default()
        }
    )).id();
    commands
        .entity(player)
        .add_children(&[left_thruster_smoke, right_thruster_smoke, left_thruster_flame, right_thruster_flame, dash_thruster]);

    let mut gradient_projectile = bevy_hanabi::Gradient::new();
    gradient_projectile.add_key(0.0, Vec4::new(0.9, 0.98, 1.0, 1.0));
    gradient_projectile.add_key(0.15, Vec4::new(0.0, 0.843, 1.0, 1.0));
    gradient_projectile.add_key(0.6, Vec4::new(0.0, 0.2, 0.7, 0.5));
    gradient_projectile.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let mut size_projectile = bevy_hanabi::Gradient::new();
    size_projectile.add_key(0.0, Vec3::splat(0.2));
    size_projectile.add_key(0.05, Vec3::splat(1.5));
    size_projectile.add_key(0.2, Vec3::splat(0.8));
    size_projectile.add_key(1.0, Vec3::splat(0.0));
    let mut module = Module::default();
    let accel = module.lit(Vec3::new(0., 0., 0.));
    let update_accel = AccelModifier::new(accel);
    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(0.03), 
        dimension: ShapeDimension::Surface, 
    };
    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::new(0.0, 0.0, 1.0)),
        speed: module.lit(15.0),
    };
    let lifetime = module.lit(1.5);
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
    let init_size = SetAttributeModifier::new(Attribute::SIZE, module.lit(1.0));
    let init_color = SetAttributeModifier::new(Attribute::COLOR, module.lit(0xFFFFFFFFu32));
    let projectile_effect_base = effects.add(
        EffectAsset::new(63000, SpawnerSettings::once(300.0.into()), module)
            .with_simulation_space(SimulationSpace::Local)
            .with_alpha_mode(AlphaMode::Add)
            .init(init_pos)
            .init(init_vel)
            .init(init_lifetime)
            .init(init_size)
            .init(init_color)
            .render(ColorOverLifetimeModifier {
                gradient: gradient_projectile,
                ..Default::default()
            })
            .render(SizeOverLifetimeModifier {
                gradient: size_projectile,
                screen_space_size: false,
                ..Default::default()
            })
            .update(update_accel)
    );
    let projectile_flash = commands.spawn((
        Name::new("projectile Flash"),
        ParticleEffect::new(projectile_effect_base),
        EffectSpawner::default(),
        Transform::from_xyz(0.0, 2.0, -0.5), 
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        ProjectileFlashEffect(1),
    )).id();
    let mut gradient_projectile_2 = bevy_hanabi::Gradient::new();
    gradient_projectile_2.add_key(0.0, Vec4::new(0.8, 1.0, 1.0, 1.0));
    gradient_projectile_2.add_key(0.1, Vec4::new(0.0, 1.0, 0.8, 1.0));
    gradient_projectile_2.add_key(0.5, Vec4::new(0.0, 0.5, 0.2, 0.8));
    gradient_projectile_2.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let mut size_projectile_2 = bevy_hanabi::Gradient::new();
    size_projectile_2.add_key(0.0, Vec3::splat(0.1));
    size_projectile_2.add_key(0.05, Vec3::splat(0.6));
    size_projectile_2.add_key(0.2, Vec3::splat(0.1));
    size_projectile_2.add_key(1.0, Vec3::splat(0.0));
    let mut module = Module::default();
    let accel = module.lit(Vec3::new(0., 0., 0.));
    let update_accel = AccelModifier::new(accel);
    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(0.03), 
        dimension: ShapeDimension::Surface, 
    };
    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::new(0.0, 0.0, 1.0)),
        speed: module.lit(80.0),
    };
    let lifetime = module.lit(1.5);
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
    let init_size = SetAttributeModifier::new(Attribute::SIZE, module.lit(1.0));
    let init_color = SetAttributeModifier::new(Attribute::COLOR, module.lit(0xFFFFFFFFu32));
    let projectile_effect_2_base = effects.add(
        EffectAsset::new(63000, SpawnerSettings::once(300.0.into()), module)
            .with_simulation_space(SimulationSpace::Local)
            .with_alpha_mode(AlphaMode::Add)
            .init(init_pos)
            .init(init_vel)
            .init(init_lifetime)
            .init(init_size)
            .init(init_color)
            .render(ColorOverLifetimeModifier {
                gradient: gradient_projectile_2,
                ..Default::default()
            })
            .render(SizeOverLifetimeModifier {
                gradient: size_projectile_2,
                screen_space_size: false,
                ..Default::default()
            })
            .update(update_accel)
    );
    let projectile_effect_2 = commands.spawn((
        Name::new("projectile Effect 2"),
        ParticleEffect::new(projectile_effect_2_base),
        EffectSpawner::default(),
        Transform::from_xyz(0.0, 2.0, -0.5), 
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        ProjectileFlashEffect(2),
    )).id();

    let mut gradient_projectile_3 = bevy_hanabi::Gradient::new();
    gradient_projectile_3.add_key(0.0, Vec4::new(1.0, 0.9, 0.5, 1.0));
    gradient_projectile_3.add_key(0.1, Vec4::new(1.0, 0.4, 0.0, 1.0));
    gradient_projectile_3.add_key(0.3, Vec4::new(0.2, 0.2, 0.2, 0.8));
    gradient_projectile_3.add_key(1.0, Vec4::new(0.1, 0.1, 0.1, 0.0));

    let mut size_projectile_3 = bevy_hanabi::Gradient::new();
    size_projectile_3.add_key(0.0, Vec3::splat(0.5));
    size_projectile_3.add_key(0.1, Vec3::splat(2.5));
    size_projectile_3.add_key(0.4, Vec3::splat(1.5));
    size_projectile_3.add_key(1.0, Vec3::splat(0.0));
    
    let mut module = Module::default();
    let accel = module.lit(Vec3::new(0., 1.5, 0.));
    let update_accel = AccelModifier::new(accel);
    
    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(0.08), 
        dimension: ShapeDimension::Volume,
    };
    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::new(0.0, 0.0, 1.0)),
        speed: module.lit(20.0),
    };
    let lifetime = module.lit(2.5);
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
    let init_size = SetAttributeModifier::new(Attribute::SIZE, module.lit(1.0));
    let init_color = SetAttributeModifier::new(Attribute::COLOR, module.lit(0xFFFFFFFFu32));
    
    let projectile_effect_3_base = effects.add(
        EffectAsset::new(63000, SpawnerSettings::once(800.0.into()), module)
            .with_simulation_space(SimulationSpace::Local)
            .with_alpha_mode(AlphaMode::Blend) 
            .init(init_pos)
            .init(init_vel)
            .init(init_lifetime)
            .init(init_size)
            .init(init_color)
            .render(ColorOverLifetimeModifier {
                gradient: gradient_projectile_3,
                ..Default::default()
            })
            .render(SizeOverLifetimeModifier {
                gradient: size_projectile_3,
                screen_space_size: false,
                ..Default::default()
            })
            .update(update_accel)
    );
    let projectile_effect_3 = commands.spawn((
        Name::new("projectile Effect 3"),
        ParticleEffect::new(projectile_effect_3_base),
        EffectSpawner::default(),
        Transform::from_xyz(0.0, 2.0, -0.5), 
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        ProjectileFlashEffect(3),
    )).id();
}

fn camera_effects(mut dash_camera_query: Query<&mut Projection, With<Camera3d>>, mut dash_effect_camera: ResMut<CameraDashEffect>, aim_distance: Res<AimDistance>, mut camera_dof_query: Query<&mut DepthOfField>, mut camera: Query<&mut Transform, With<Camera>>, mut timer: Local<f32>, player_q: Query<&LinearVelocity, With<Player>>, time: Res<Time>, mut shake: ResMut<ScreenShake>) {
    let Ok(mut camera_transform) = camera.single_mut() else {
        return;
    };
    let Ok(velocity) = player_q.single() else {
        return;
    };
    if let Ok(mut dof) = camera_dof_query.single_mut() {
        dof.focal_distance = aim_distance.0;
    }
    if shake.strength > 0.01 {
        camera_transform.translation += Vec3::new(
            rand::rng().random_range(-shake.strength * 1.25..shake.strength * 1.25),
            rand::rng().random_range(-shake.strength * 1.25..shake.strength * 1.25),
            rand::rng().random_range(-shake.strength * 1.25..shake.strength * 1.25),
        );
        shake.strength *= 0.05_f32.powf(time.delta_secs());
    } else {
        shake.strength = 0.0;
    }
    let speed = Vec2::new(velocity.x, velocity.z).length();
    if speed > 0.1 {
        *timer += time.delta_secs() * 10.0;
        camera_transform.translation.y += (*timer).sin() * 0.035;
        camera_transform.translation.x += (*timer * 0.5).cos() * 0.018;
    } else {
        *timer *= 1.0 - time.delta_secs() * 8.0;
    }
    let Ok(mut projection) = dash_camera_query.single_mut() else {
        return;
    };
    dash_effect_camera.active_multiplier += (1.0 - dash_effect_camera.active_multiplier) * time.delta_secs() * 10.0;
    if let Projection::Perspective(ref mut perspective) = *projection {
        let base_fov = 90.0_f32.to_radians();
        perspective.fov = base_fov * dash_effect_camera.active_multiplier;
    }
}

fn hitmarker(mut query: Query<&mut BackgroundColor, With<Hitmarker>>, mut timer: ResMut<HitmarkerTimer>, time: Res<Time>) {
    if timer.0.tick(time.delta()).just_finished() {
        for mut color in &mut query {
            color.0 = Color::srgba(1.0, 1.0, 1.0, 0.0);
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

fn shooting(
    (impact_effects, timer, time, selected_weapon, mouse_button, mut aim_distance): (
        Res<ImpactEffects>,
        ResMut<HitmarkerTimer>,
        Res<Time>,
        Res<SelectedWeapon>,
        Res<ButtonInput<MouseButton>>,
        ResMut<AimDistance>,
    ),
    (mut shake, mut crosshair_spread, mut crosshair, window): (
        ResMut<ScreenShake>,
        ResMut<CrosshairSpread>,
        ResMut<FloatingCrosshair>,
        Single<&Window>,
    ),
    mut commands: Commands,
    mut shooting_effects: Query<(&mut EffectSpawner, &mut Transform, &ProjectileFlashEffect), (Without<Player>, Without<Camera3d>)>,
    mut fire_cooldown: Local<f32>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut gizmos: Gizmos,
    query: Query<&mut BotData>,
    mut ray_cast: MeshRayCast,
    parent: Query<&ChildOf>,
    q: Query<&mut BackgroundColor, With<Hitmarker>>,
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
) {
    if *fire_cooldown > 0.0 {
        *fire_cooldown -= time.delta_secs();
    }
    if !mouse_button.pressed(MouseButton::Left) { return; }
    if *fire_cooldown > 0.0 {
        return;
    }
    let Ok(player_transform) = player_query.single_mut() else {
        return;
    };
    let current_weapon = selected_weapon.id;
    let fire_rate = match current_weapon {
        1 => 0.5,
        2 => 1.0,
        3 => 5.0,
        _ => 0.5,
    };
    *fire_cooldown = fire_rate;
    let max_range = match current_weapon {
        1 => 50.0,
        2 => 100.0,
        3 => 175.0,
        _ => 50.0,
    };
    let damage = match current_weapon {
        1 => 5,
        2 => 10,
        3 => 50,
        _ => 1,
    };
    let flash_scale = match current_weapon {
        1 => 1.0,
        2 => 1.5,
        3 => 2.0,
        _ => 1.0,
    };
    let shake_strength = match current_weapon {
        1 => 0.2,
        2 => 0.5,
        3 => 1.0,
        _ => 0.2,
    };
    let base_spread = match current_weapon {
        1 => 0.4,
        2 => 0.72,
        3 => 0.20,
        _ => 0.4,
    };
    let sound = match current_weapon {
        1 => "Player/Pistol.ogg",
        2 => "Player/Rifle.ogg",
        3 => "Player/Rocket.ogg",
        _ => "Player/Pistol.ogg",
    };
    let shot_sound = audio.play(asset_server.load(sound)).handle();
    commands.spawn((
        Transform::from_translation(player_transform.translation),
        SpatialAudioEmitter {
            instances: vec![shot_sound]
        }
    ));
    let in_screen_pos = Vec2::new(window.width() / 2.0 + crosshair.x, window.height() / 2.0 + crosshair.y - 100.0);
    let (inner_camera, camera_transform) = camera.into_inner();
    let Ok(camera_ray) = inner_camera.viewport_to_world(camera_transform, in_screen_pos) else { return; };
    let target = if let Some((_, hit)) = ray_cast.cast_ray(camera_ray, &MeshRayCastSettings::default()).first() {
        hit.point
    } else {
        camera_ray.origin + *camera_ray.direction * max_range
    };
    aim_distance.0 = (target - camera_transform.translation()).length();
    let forward = -player_transform.forward();
    let ray_pos = player_transform.translation + Vec3::new(0.0, 1.25, 0.0) + *forward * 2.25;
    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.85, 0.4),
            intensity: 50_000.0,
            range: 8.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(ray_pos),
        DespawnTimer(Timer::from_seconds(0.05, TimerMode::Once)),
    ));
    crosshair.y -= 200.0;
    let total_spread = base_spread + (crosshair_spread.spread * 0.2);
    let mut dir_vec = (target - ray_pos).normalize_or_zero();
    if dir_vec == Vec3::ZERO { dir_vec = *camera_ray.direction; }
    if total_spread > 0.0 {
        dir_vec.x += rand::rng().random_range(-total_spread..total_spread);
        dir_vec.y += rand::rng().random_range(-total_spread..total_spread);
        dir_vec.z += rand::rng().random_range(-total_spread..total_spread);
    }
    let dir = Dir3::new(dir_vec).unwrap_or(camera_ray.direction);
    ray_handling(audio, asset_server, current_weapon, impact_effects, commands, timer, q, ray_pos, dir, damage, max_range, time, ray_cast, &mut gizmos, query, parent);
    for (mut spawner, mut transform, projectile) in shooting_effects.iter_mut() {
        if projectile.0 == current_weapon {
            transform.translation = ray_pos;
            transform.scale = Vec3::splat(flash_scale);
            transform.look_to(dir, Vec3::Y);
            spawner.active = true;
            spawner.reset();
        }
    }
    crosshair_spread.spread += 20.0 * flash_scale;
    shake.strength = shake_strength;
}

fn player_death(mut commands: Commands, query: Query<(Entity, &PlayerData), With<Player>>) {
    if let Ok((entity, player_data)) = query.single() {
        if player_data.health <= 0 {
            commands.entity(entity).despawn();
            println!("Player has died!");
        }
    }
}

fn setup_main_menu(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Node::default(),
        NodeStyleSheet::new(asset_server.load("menu/main_menu.css")),
        MainMenuUi,
        children![
            (Node::default(), Name::new("game_menu"), children![
                (Text::new("Mech Game".to_string()), Name::new("menu_title"), Node::default()),
                (Button, StartButton, Node::default(), children![(Text::new("Start"), Node::default())]),
                (Button, SettingsButton, Node::default(), children![(Text::new("Settings"), Node::default())]),
                (Node::default(), Name::new("floating_borders"))
            ]),
        ],
    ));
}

fn cycle_menu(
    keycode: Res<ButtonInput<KeyCode>>,
    mut menu: ResMut<CycleMenu>,
    mut query: Query<&mut Text, With<CycleTextTarget>>,
    q_prev: Query<&Interaction, (Changed<Interaction>, With<PrevOptionButton>)>,
    q_next: Query<&Interaction, (Changed<Interaction>, With<NextOptionButton>)>,
) {
    let mut changed = false;
    let mut right_pressed = keycode.just_pressed(KeyCode::ArrowRight);
    let mut left_pressed = keycode.just_pressed(KeyCode::ArrowLeft);

    for interaction in q_next.iter() {
        if *interaction == Interaction::Pressed {
            right_pressed = true;
        }
    }
    for interaction in q_prev.iter() {
        if *interaction == Interaction::Pressed {
            left_pressed = true;
        }
    }

    if right_pressed {
        menu.index = (menu.index + 1) % menu.options.len();
        changed = true;
    } else if left_pressed {
        if menu.index == 0 {
            menu.index = menu.options.len() - 1;
        } else {
            menu.index -= 1;
        }
        changed = true;
    }
    if changed {
        for mut text in &mut query {
            text.0 = menu.options[menu.index].to_string();
        }
    }
}

fn settings_menu(
    mut state: ResMut<NextState<AppState>>,
    q_start: Query<&Interaction, (Changed<Interaction>, With<StartButton>)>,
    q_settings: Query<&Interaction, (Changed<Interaction>, With<SettingsButton>)>,
    q_main_menu: Query<Entity, With<MainMenuUi>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    menu: Res<CycleMenu>,
) {
    for interaction in q_start.iter() {
        if *interaction == Interaction::Pressed {
            state.set(AppState::InGame);
        }
    }
    
    for interaction in q_settings.iter() {
        if *interaction == Interaction::Pressed {
            println!("Settings button clicked!");
            for entity in q_main_menu.iter() {
                commands.entity(entity).despawn();
            }
            commands.spawn((
                Node::default(),
                NodeStyleSheet::new(asset_server.load("menu/settings.css")),
                MainMenuUi,
                children![
                    (Node { flex_direction: FlexDirection::Row, ..default() }, Name::new("cycle_container"), children![
                        (Button, PrevOptionButton, Node { width: Val::Px(40.0), height: Val::Px(40.0), ..default() }, children![(Text::new("<"), Node::default())]),
                        (Text::new(menu.options[menu.index].clone()), Name::new("cycle_text"), CycleTextTarget, Node::default()),
                        (Button, NextOptionButton, Node { width: Val::Px(40.0), height: Val::Px(40.0), ..default() }, children![(Text::new(">"), Node::default())]),
                    ]),
                    (Button, Name::new("apply_button"), ApplySettingsButton, Node::default(), children![(Text::new("Apply"), Node::default())]),
                ],
            ));
        }
    }
}

fn settings_apply(mut state: ResMut<NextState<AppState>>, mut algorithm: ResMut<TerrainAlgorithm>, q_main_menu: Query<Entity, With<MainMenuUi>>, query: Query<&Interaction, (Changed<Interaction>, With<ApplySettingsButton>)>, menu: Res<CycleMenu>, mut player_model: ResMut<PlayerModel>, mut graphics: ResMut<SsaoEnabled>, mut commands: Commands, asset_server: Res<AssetServer>) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            println!("Settings applied!");
            match menu.index {
                0 => {
                    player_model.model_name = "Player/Player.glb".to_string();
                    graphics.enabled = false;
                }
                1 => {
                    player_model.model_name = "Player/Player_Highpoly.glb".to_string();
                    *algorithm = TerrainAlgorithm::Sobel;
                    graphics.enabled = false;
                }
                2 => {
                    player_model.model_name = "Player/Player_Highpoly.glb".to_string();
                    *algorithm = TerrainAlgorithm::AreaWeighted;
                    graphics.enabled = true;
                }
                _ => {
                    player_model.model_name = "Player/Player.glb".to_string();
                    *algorithm = TerrainAlgorithm::Sobel;
                    graphics.enabled = false;
                }
            }
            for entity in q_main_menu.iter() {
                commands.entity(entity).despawn();
            }
            state.set(AppState::InGame);
        }
    }
}

fn main_menu_handling(mut commands: Commands, query: Query<Entity, With<MainMenuUi>>, camera_query: Query<Entity, With<Camera2d>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in camera_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn main() {
    App::new()
        .add_plugins(EmbeddedAssetPlugin {
            mode: bevy_embedded_assets::PluginMode::ReplaceDefault,
        })
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Mech Game".into(),
                        resolution: WindowResolution::new(1920, 1080),
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        limits: WgpuLimits {
                            max_sampled_textures_per_shader_stage: 256,
                            max_samplers_per_shader_stage: 256,
                            ..default()
                        },
                        ..default()
                    }),
                    ..default()
                })
                .disable::<bevy::audio::AudioPlugin>(),
        )
        .add_plugins(bevy_kira_audio::AudioPlugin)
        .add_plugins(bevy_kira_audio::SpatialAudioPlugin)
        .add_plugins(UiMaterialPlugin::<JumpIndicator>::default())
        .add_plugins(UiMaterialPlugin::<HealthBarUI>::default())
        .add_plugins(UiMaterialPlugin::<SprintBarUI>::default())
        .add_plugins(UiMaterialPlugin::<WeaponSelectorUI>::default())
        .add_plugins(HanabiPlugin)
        .add_plugins(FlairPlugin)
        .init_asset::<WeaponData>()
        .init_asset::<ProjectileFlash>()
        .init_state::<AppState>()
        .init_resource::<SsaoEnabled>()
        .init_resource::<TerrainGen>()
        .init_resource::<FloatingCrosshair>()
        .init_resource::<SelectedWeapon>()
        .init_resource::<BotConfig>()
        .init_resource::<AimDistance>()
        .insert_resource(CrosshairSpread { spread: 0.0 })
        .insert_resource(ScreenShake { strength: 0.0 })
        .insert_resource(CameraDashEffect { active_multiplier: 1.0 })
        .insert_resource(Hitmarker)
        .insert_resource(TerrainAlgorithm::Sobel)
        .insert_resource(HitmarkerTimer(Timer::from_seconds(0.1, TimerMode::Once)))
        .insert_resource(CycleMenu {
            options: vec!["< Low Preset >".to_string(), "< Medium Preset >".to_string(), "< High Preset >".to_string()],
            index: 0,
        })
        .insert_resource(PlayerModel {
            model_name: "Player/Player.glb".to_string()
        })
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 10.0, // increase ambient light
            ..default()
        })
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity(Vec3::new(0.0, -14.0, 0.0))) 
        .add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
        .add_systems(Update, (settings_menu, cycle_menu, settings_apply).run_if(in_state(AppState::MainMenu)))
        .add_systems(OnExit(AppState::MainMenu), main_menu_handling)
        .add_systems(Startup, setup_impact_effects)
        .add_systems(OnEnter(AppState::InGame), (spawn_player, setup, bot_spawn, jump_indicator, health_bar, sprint_bar, weapon_selector_setup, procedural_terrain))
        .add_systems(
            Update,
            (
                player_movement,
                setup_scene_once_loaded,
                movement_animations,
                camera_positioning,
                bot_handling,
                cursor_handling,
                mesh_load_check,
                shooting,
                jump_indicator_handling,
                health_bar_handling,
                sprint_bar_handling,
                despawn_ray,
                targeting_disruptor,
                player_death,
                botdead,
                particle_effects,
                bevy_symbios_ground::sync_splat_texture,
            ).run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (
                weapon_selector,
                crosshair_spread,
                hitmarker,
                camera_effects.after(camera_positioning),
            ).run_if(in_state(AppState::InGame)),
        )
        .run();
}