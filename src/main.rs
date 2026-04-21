use bevy::prelude::*;
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
    animations: Res<Animations>,
    mut current_animation: Local<usize>,
) {
    for (mut player, mut transitions) in &mut animation_players {
        let Some((&playing_animation_index, _)) = player.playing_animations().next() else {
            continue;
        };

        if keyboard_input.pressed(KeyCode::KeyW) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            if playing_animation.is_paused() {
                let _ = playing_animation.speed() == 0.5;
                playing_animation.resume();
            } else {
                playing_animation.pause();
            }
        }
    }
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

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .insert_resource(Gravity(Vec3::new(0.0, -35.0, 0.0)))
        .add_systems(Startup, (spawn_player, setup))
        .add_systems(Update, (player_movement, setup_scene_once_loaded, movement_animations))
        .run();
}
