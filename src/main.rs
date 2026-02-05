use bevy::{color::palettes::css::*, prelude::*};
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
        .add_systems(Update, apply_fallback_material)
        .run();
}

#[derive(Component)]
struct FallbackMaterialApplied;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        AmbientLight {
            color: Color::WHITE,
            brightness: 200.0,
            affects_lightmapped_meshes: false,
        },
        Name::new("Camera"),
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

    // Test cube
    /*commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::from(ORANGE))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));*/

    // Load FBX scene
    let scene = asset_server.load::<Scene>("models/Mutant.fbx#Scene0");

    // Spawn the scene
    commands.spawn((
        SceneRoot(scene),
        Transform::from_xyz(0.0, 0.5, 0.0).with_scale(Vec3::splat(100.0)),
        Name::new("Mutant"),
    ));
}

fn apply_fallback_material(
    mut commands: Commands,
    query: Query<(Entity, &MeshMaterial3d<StandardMaterial>), (With<Mesh3d>, Without<FallbackMaterialApplied>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, material_handle) in &query {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.base_color = Color::WHITE;
            material.base_color_texture = None;
            material.emissive = LinearRgba::BLACK;
            material.alpha_mode = AlphaMode::Opaque;
            material.double_sided = true;
            material.cull_mode = None;
            material.unlit = true;
        }
        commands.entity(entity).insert(FallbackMaterialApplied);
    }
}
