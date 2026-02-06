use std::collections::HashMap;

use bevy::{prelude::*, scene::SceneInstanceReady};
use bevy_common_assets::json::JsonAssetPlugin;
use serde::Deserialize;

#[derive(Event)]
struct AssetLoaded<A>(AssetId<A>)
where
    A: Asset;

// bridge method because we can't observe asset events yet
// https://github.com/bevyengine/bevy/issues/16041
fn bridge_asset_events<A>(mut events: MessageReader<AssetEvent<A>>, mut commands: Commands)
where
    A: Asset,
{
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            commands.trigger(AssetLoaded(*id));
        }
    }
}

#[derive(Deserialize, Asset, TypePath)]
struct CharacterData {
    model_path: String,
    animation_paths: HashMap<String, String>,
}

impl CharacterData {
    pub fn model_scene_path(&self) -> String {
        format!("{}#Scene0", self.model_path)
    }

    pub fn animation_path(&self, name: impl AsRef<str>) -> String {
        format!("{}#Animation0", self.animation_paths[name.as_ref()])
    }
}

#[derive(Resource)]
struct Characters {
    // we have to hold the data handle until everything is loaded
    // or the asset system will free it before we get a chance to use it
    mutant: Handle<CharacterData>,
    mutant_animations: HashMap<String, (Handle<AnimationGraph>, AnimationNodeIndex)>,
}

#[derive(Component)]
struct Rotator;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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

    // load characters
    let mutant = asset_server.load::<CharacterData>("characters/mutant.json");
    commands.insert_resource(Characters {
        mutant,
        mutant_animations: HashMap::new(),
    });
}

fn on_character_loaded(
    event: On<AssetLoaded<CharacterData>>,
    mut commands: Commands,
    character_datum: Res<Assets<CharacterData>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    mut characters: ResMut<Characters>,
    asset_server: Res<AssetServer>,
) {
    if let Some(character_data) = character_datum.get(event.0) {
        // load model
        let model = asset_server.load::<Scene>(character_data.model_scene_path());

        // spawn the scene
        commands
            .spawn((
                SceneRoot(model),
                Transform::from_xyz(0.0, 0.0, 0.0),
                Name::new("Mutant"),
                //Rotator,
            ))
            .observe(start_idle);

        // load Idle animation
        let idle_animation =
            asset_server.load::<AnimationClip>(character_data.animation_path("idle"));
        let (idle_animation_graph, idle_animation_index) =
            AnimationGraph::from_clip(idle_animation);
        let idle_animation_graph = animation_graphs.add(idle_animation_graph);

        characters.mutant_animations.insert(
            "idle".to_string(),
            (idle_animation_graph, idle_animation_index),
        );
    }
}

fn start_idle(
    scene_ready: On<SceneInstanceReady>,
    mut commands: Commands,
    characters: Res<Characters>,
    children: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
) {
    // TODO: currently only works for the mutant
    for child in children.iter_descendants(scene_ready.entity) {
        if let Ok(mut player) = players.get_mut(child) {
            let (idle_animation_graph, idle_animation_index) =
                characters.mutant_animations.get("idle").unwrap();
            player.play(*idle_animation_index).repeat();

            // NOTE: only do this once (probably a separate system?)
            // connect the animation player to the mesh
            commands
                .entity(child)
                .insert(AnimationGraphHandle(idle_animation_graph.clone()));
        }
    }
}

fn rotate_model(time: Res<Time>, mut query: Query<&mut Transform, With<Rotator>>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() * 0.5);
    }
}

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);

    app.add_plugins(bevy::remote::RemotePlugin::default())
        .add_plugins(bevy::remote::http::RemoteHttpPlugin::default());

    app.add_plugins(JsonAssetPlugin::<CharacterData>::new(&[".json"]))
        .add_systems(Update, bridge_asset_events::<CharacterData>)
        .add_observer(on_character_loaded);

    app.add_systems(Startup, setup)
        .add_systems(Update, rotate_model);

    app.run();
}
