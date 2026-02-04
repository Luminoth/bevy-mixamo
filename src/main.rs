use bevy::prelude::*;
use bevy_ufbx::FbxPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // TODO: this isn't great but fbx's won't load without it
            unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
            ..default()
        }))
        .add_plugins(bevy::remote::RemotePlugin::default())
        .add_plugins(bevy::remote::http::RemoteHttpPlugin::default())
        .add_plugins(FbxPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Camera"),
    ));

    // Ambient light
    commands.spawn((
        AmbientLight {
            color: Color::WHITE,
            brightness: 200.0,
            affects_lightmapped_meshes: false,
        },
        Name::new("Ambient Light"),
    ));

    // Light
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

    // Load FBX scene
    let scene = asset_server.load::<Scene>("models/Mutant.fbx#Scene0");

    // Spawn the scene
    commands.spawn((SceneRoot(scene), Name::new("Mutant")));
}
