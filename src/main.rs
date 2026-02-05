use bevy::{prelude::*, scene::SceneInstanceReady};

#[derive(Resource)]
struct Animations {
    idle_animation_graph: Handle<AnimationGraph>,
    idle_animation_index: AnimationNodeIndex,
}

#[derive(Component)]
struct Rotator;

fn setup(
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
) {
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.0, 5.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        AmbientLight {
            color: Color::WHITE,
            brightness: 200.0,
            affects_lightmapped_meshes: false,
        },
        Name::new("Camera"),
    ));

    // light
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -45f32.to_radians(),
            45f32.to_radians(),
            0.0,
        )),
        Name::new("Directional Light"),
    ));

    // load GLB scene
    let scene = asset_server.load::<Scene>("models/Mutant.glb#Scene0");

    // spawn the scene
    commands
        .spawn((
            SceneRoot(scene),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Name::new("Mutant"),
            //Rotator,
        ))
        .observe(start_idle);

    // load Idle animation
    let idle_animation =
        asset_server.load::<AnimationClip>("animations/Breathing Idle.glb#Animation0");
    let (idle_animation_graph, idle_animation_index) = AnimationGraph::from_clip(idle_animation);
    let idle_animation_graph = graphs.add(idle_animation_graph);

    commands.insert_resource(Animations {
        idle_animation_graph,
        idle_animation_index,
    });
}

fn start_idle(
    scene_ready: On<SceneInstanceReady>,
    mut commands: Commands,
    animations: Res<Animations>,
    children: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
) {
    for child in children.iter_descendants(scene_ready.entity) {
        if let Ok(mut player) = players.get_mut(child) {
            player.play(animations.idle_animation_index).repeat();

            // NOTE: only do this once (probalby a separate system?)
            // connect the animation player to the mesh
            commands.entity(child).insert(AnimationGraphHandle(
                animations.idle_animation_graph.clone(),
            ));
        }
    }
}

fn rotate_model(time: Res<Time>, mut query: Query<&mut Transform, With<Rotator>>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() * 0.5);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy::remote::RemotePlugin::default())
        .add_plugins(bevy::remote::http::RemoteHttpPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_model)
        .run();
}
