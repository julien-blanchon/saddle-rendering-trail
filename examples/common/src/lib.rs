use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub const AUTO_EXIT_ENV: &str = "TRAIL_AUTO_EXIT_SECONDS";

#[derive(Resource)]
struct AutoExitAfter(Timer);

pub fn default_plugins(title: &str) -> impl PluginGroup {
    DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: title.to_string(),
            resolution: (1400, 900).into(),
            ..default()
        }),
        ..default()
    })
}

pub fn install_auto_exit(app: &mut App) {
    if let Some(seconds) = std::env::var(AUTO_EXIT_ENV)
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|seconds| *seconds > 0.0)
    {
        app.insert_resource(AutoExitAfter(Timer::from_seconds(seconds, TimerMode::Once)))
            .add_systems(Update, auto_exit_after);
    }
}

pub fn spawn_stage(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    spawn_stage_scene(commands, meshes, materials);
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 12.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));
}

pub fn spawn_stage_scene(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 18_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Name::new("Fill Light"),
        PointLight {
            intensity: 8_000.0,
            range: 24.0,
            ..default()
        },
        Transform::from_xyz(-5.0, 4.0, 8.0),
    ));
    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(24.0, 24.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.08, 0.09, 0.11),
            perceptual_roughness: 0.92,
            ..default()
        })),
    ));
}

pub fn spawn_overlay(commands: &mut Commands, title: &str, subtitle: &str) {
    commands.spawn((
        Name::new("Overlay"),
        Node {
            position_type: PositionType::Absolute,
            left: px(16.0),
            top: px(16.0),
            width: px(420.0),
            padding: UiRect::all(px(12.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.05, 0.08, 0.8)),
        Text::new(format!("{title}\n{subtitle}")),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

pub fn stripe_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let mut bytes = Vec::with_capacity(64 * 4);
    for index in 0..64u8 {
        let alpha = if (index / 8) % 2 == 0 { 255 } else { 64 };
        bytes.extend_from_slice(&[255, 255, 255, alpha]);
    }
    let image = Image::new_fill(
        Extent3d {
            width: 64,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    images.add(image)
}

pub fn glow_material(
    materials: &mut Assets<StandardMaterial>,
    color: Color,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: color,
        emissive: color.to_linear() * 0.25,
        unlit: false,
        ..default()
    })
}

fn auto_exit_after(
    time: Res<Time>,
    mut timer: ResMut<AutoExitAfter>,
    mut exit: MessageWriter<AppExit>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        exit.write(AppExit::Success);
    }
}
