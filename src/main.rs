use std::collections::HashMap;

use bevy::{prelude::*, scene::SceneInstanceReady};
use bevy_common_assets::json::JsonAssetPlugin;
use serde::Deserialize;

#[derive(Event)]
struct AssetLoadedEvent<A>(AssetId<A>)
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
            debug!("bridging asset load for {}", id);
            commands.trigger(AssetLoadedEvent(*id));
        }
    }
}

#[derive(Deserialize, Asset, TypePath)]
struct CharacterData {
    id: String,
    model_path: String,
    animation_paths: HashMap<String, String>,
}

impl CharacterData {
    pub fn model_scene_path(&self) -> String {
        format!("{}#Scene0", self.model_path)
    }

    pub fn animations(&self) -> impl Iterator<Item = &String> {
        self.animation_paths.keys()
    }

    pub fn animation_path(&self, name: impl AsRef<str>) -> String {
        format!("{}#Animation0", self.animation_paths[name.as_ref()])
    }
}

struct Character {
    data: Handle<CharacterData>,
    animations: HashMap<String, (Handle<AnimationGraph>, AnimationNodeIndex)>,
}

#[derive(Resource)]
struct Characters(HashMap<String, Character>);

#[derive(Component)]
struct CharacterModel(Handle<CharacterData>);

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
    let mut characters = HashMap::new();

    info!("Loading character 'mutant' from 'characters/mutant.json' ...");
    let data = asset_server.load::<CharacterData>("characters/mutant.json");

    // we have to hold the data handle until the asset is loaded
    // or the asset system will free it before we get a chance to use it
    characters.insert(
        "mutant".to_owned(),
        Character {
            data,
            animations: HashMap::new(),
        },
    );

    commands.insert_resource(Characters(characters));
}

fn on_character_data_loaded(
    event: On<AssetLoadedEvent<CharacterData>>,
    mut commands: Commands,
    character_datum: Res<Assets<CharacterData>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    mut characters: ResMut<Characters>,
    asset_server: Res<AssetServer>,
) {
    let character_data = character_datum.get(event.0).unwrap();
    info!(
        "Loaded character data for '{}', loading assets ...",
        character_data.id
    );

    let character = characters.0.get_mut(&character_data.id).unwrap();

    // load model
    let model_path = character_data.model_scene_path();
    info!("Loading character model from '{}' ...", model_path);
    let model = asset_server.load::<Scene>(model_path);

    // spawn the scene
    commands
        .spawn((
            SceneRoot(model),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Name::new(character_data.id.clone()),
            CharacterModel(character.data.clone()),
            //Rotator,
        ))
        .observe(start_idle);

    // load animations
    for animation_name in character_data.animations() {
        let animation_path = character_data.animation_path(animation_name);
        info!(
            "Loading character animation '{}' from '{}' ...",
            animation_name, animation_path
        );
        let animation_clip =
            asset_server.load::<AnimationClip>(character_data.animation_path(animation_name));

        let (animation_graph, animation_index) = AnimationGraph::from_clip(animation_clip);
        let animation_graph = animation_graphs.add(animation_graph);

        character
            .animations
            .insert(animation_name.clone(), (animation_graph, animation_index));
    }
}

fn start_idle(
    scene_ready: On<SceneInstanceReady>,
    mut commands: Commands,
    character_datum: Res<Assets<CharacterData>>,
    characters: Res<Characters>,
    character_models: Query<&CharacterModel>,
    children: Query<&Children>,
    mut animation_players: Query<&mut AnimationPlayer>,
) {
    let character_model = character_models.get(scene_ready.entity).unwrap();
    let character_data = character_datum.get(&character_model.0).unwrap();

    for child in children.iter_descendants(scene_ready.entity) {
        if let Ok(mut player) = animation_players.get_mut(child) {
            info!(
                "Running idle animation for character '{}' ...",
                character_data.id
            );

            let (animation_graph, animation_index) = characters
                .0
                .get(&character_data.id)
                .unwrap()
                .animations
                .get("idle")
                .unwrap();
            player.play(*animation_index).repeat();

            // TODO: only do this once (probably a separate system?)
            // connect the animation player to the mesh
            commands
                .entity(child)
                .insert(AnimationGraphHandle(animation_graph.clone()));
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
        .add_observer(on_character_data_loaded);

    app.add_systems(Startup, setup)
        .add_systems(Update, rotate_model);

    app.run();
}
