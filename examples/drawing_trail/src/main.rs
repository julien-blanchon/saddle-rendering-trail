use saddle_rendering_trail_example_common as common;

use bevy::{prelude::*, window::PrimaryWindow};
use saddle_rendering_trail::{
    Trail, TrailColorKey, TrailDiagnostics, TrailEmitterMode, TrailGradient, TrailMaterial,
    TrailPlugin, TrailScalarCurve, TrailScalarKey, TrailStyle,
};

#[derive(Component)]
struct DrawCursor;

#[derive(Component)]
struct OverlayText;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        common::default_plugins("trail drawing"),
        TrailPlugin::default(),
    ));
    common::install_auto_exit(&mut app);
    app.add_systems(Startup, setup)
        .add_systems(Update, (follow_mouse, update_overlay));
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Top-down camera for drawing on the XZ plane
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 14.0, 0.001).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 12_000.0,
            ..default()
        },
        Transform::from_xyz(4.0, 10.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(30.0, 30.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.06, 0.07, 0.09),
            perceptual_roughness: 0.95,
            ..default()
        })),
    ));

    // Trail-emitting cursor (moves on the XZ plane)
    commands.spawn((
        Name::new("Draw Cursor"),
        DrawCursor,
        Trail::default()
            .with_emitter_mode(TrailEmitterMode::Always)
            .with_lifetime_secs(4.0)
            .with_style(TrailStyle {
                base_width: 0.3,
                width_over_length: TrailScalarCurve::new([
                    TrailScalarKey::new(0.0, 0.4),
                    TrailScalarKey::new(0.5, 0.9),
                    TrailScalarKey::new(1.0, 1.0),
                ]),
                color_over_length: TrailGradient::new([
                    TrailColorKey::new(0.0, Color::srgb(0.9, 0.2, 0.6)),
                    TrailColorKey::new(0.35, Color::srgb(0.3, 0.5, 1.0)),
                    TrailColorKey::new(0.7, Color::srgb(0.2, 0.9, 0.7)),
                    TrailColorKey::new(1.0, Color::srgb(1.0, 1.0, 0.6)),
                ]),
                alpha_over_length: TrailScalarCurve::new([
                    TrailScalarKey::new(0.0, 0.0),
                    TrailScalarKey::new(0.08, 0.6),
                    TrailScalarKey::new(1.0, 1.0),
                ]),
                alpha_over_age: TrailScalarCurve::linear(1.0, 0.0),
                material: TrailMaterial {
                    emissive: LinearRgba::rgb(0.3, 0.15, 0.5),
                    ..default()
                },
                ..default()
            }),
        Mesh3d(meshes.add(Sphere::new(0.15).mesh().uv(12, 8))),
        MeshMaterial3d(common::glow_material(
            &mut materials,
            Color::srgb(0.7, 0.3, 1.0),
        )),
        Transform::from_xyz(0.0, 0.05, 0.0),
    ));

    commands.spawn((
        Name::new("Overlay"),
        OverlayText,
        Node {
            position_type: PositionType::Absolute,
            left: px(16.0),
            top: px(16.0),
            width: px(400.0),
            padding: UiRect::all(px(12.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.05, 0.08, 0.8)),
        Text::new("Drawing Trail\nMove mouse to paint. Trail fades over 4 seconds."),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

fn follow_mouse(
    window_query: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut cursor: Query<&mut Transform, With<DrawCursor>>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };

    // Raycast from cursor into the XZ plane at y=0.05
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };
    let plane = InfinitePlane3d::new(Vec3::Y);
    let Some(distance) = ray.intersect_plane(Vec3::new(0.0, 0.05, 0.0), plane) else {
        return;
    };
    let world_pos = ray.get_point(distance);

    for mut transform in &mut cursor {
        transform.translation = world_pos;
    }
}

fn update_overlay(
    diagnostics: Res<TrailDiagnostics>,
    mut overlays: Query<&mut Text, With<OverlayText>>,
) {
    for mut text in &mut overlays {
        **text = format!(
            "Drawing Trail\nMove mouse to paint. Trail fades over 4 seconds.\npoints={} rebuilds={}",
            diagnostics.active_points, diagnostics.total_mesh_rebuilds,
        );
    }
}
