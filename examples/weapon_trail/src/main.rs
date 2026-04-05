use saddle_rendering_trail_example_common as common;

use bevy::prelude::*;
use saddle_rendering_trail::{
    Trail, TrailColorKey, TrailFadeMode, TrailGradient, TrailMaterial, TrailOrientation,
    TrailPlugin, TrailScalarCurve, TrailScalarKey, TrailStyle,
};

#[derive(Component)]
struct SwordBlade;

#[derive(Component)]
struct AxeBlade;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        common::default_plugins("trail weapon trail"),
        TrailPlugin::default(),
    ));
    common::install_auto_exit(&mut app);
    app.add_systems(Startup, setup).add_systems(Update, animate);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    common::spawn_stage(&mut commands, &mut meshes, &mut materials);
    common::spawn_overlay(
        &mut commands,
        "Weapon Trails",
        "Left: sword swipe (width fade, fire gradient)\n\
         Right: axe swing (both fade, cyan glow)\n\
         Both use TransformLocked orientation.",
    );

    // Sword trail — width fade mode with fire gradient
    commands.spawn((
        Name::new("Sword Blade"),
        SwordBlade,
        Trail::default()
            .with_lifetime_secs(0.4)
            .with_orientation(TrailOrientation::TransformLocked { axis: Vec3::Y })
            .with_style(TrailStyle {
                base_width: 1.0,
                fade_mode: TrailFadeMode::Width,
                width_over_length: TrailScalarCurve::new([
                    TrailScalarKey::new(0.0, 0.0),
                    TrailScalarKey::new(0.15, 0.6),
                    TrailScalarKey::new(1.0, 1.2),
                ]),
                color_over_length: TrailGradient::new([
                    TrailColorKey::new(0.0, Color::srgb(1.0, 0.2, 0.05)),
                    TrailColorKey::new(0.4, Color::srgb(1.0, 0.6, 0.1)),
                    TrailColorKey::new(1.0, Color::srgb(1.0, 0.95, 0.8)),
                ]),
                alpha_over_length: TrailScalarCurve::linear(0.0, 1.0),
                material: TrailMaterial {
                    emissive: LinearRgba::rgb(1.2, 0.3, 0.05),
                    ..default()
                },
                ..default()
            }),
        Mesh3d(meshes.add(Cuboid::new(0.08, 1.4, 0.14))),
        MeshMaterial3d(common::glow_material(
            &mut materials,
            Color::srgb(1.0, 0.55, 0.15),
        )),
        Transform::from_xyz(-2.5, 1.5, 0.0),
    ));

    // Axe trail — both fade mode with cyan glow
    commands.spawn((
        Name::new("Axe Blade"),
        AxeBlade,
        Trail::default()
            .with_lifetime_secs(0.5)
            .with_orientation(TrailOrientation::TransformLocked { axis: Vec3::Y })
            .with_style(TrailStyle {
                base_width: 1.3,
                fade_mode: TrailFadeMode::Both,
                width_over_length: TrailScalarCurve::new([
                    TrailScalarKey::new(0.0, 0.0),
                    TrailScalarKey::new(0.1, 0.4),
                    TrailScalarKey::new(0.5, 0.9),
                    TrailScalarKey::new(1.0, 1.15),
                ]),
                color_over_length: TrailGradient::new([
                    TrailColorKey::new(0.0, Color::srgba(0.1, 0.4, 1.0, 0.0)),
                    TrailColorKey::new(0.3, Color::srgb(0.2, 0.7, 1.0)),
                    TrailColorKey::new(1.0, Color::srgb(0.85, 0.95, 1.0)),
                ]),
                alpha_over_length: TrailScalarCurve::linear(0.0, 1.0),
                material: TrailMaterial {
                    emissive: LinearRgba::rgb(0.1, 0.4, 1.0),
                    ..default()
                },
                ..default()
            }),
        Mesh3d(meshes.add(Cuboid::new(0.16, 0.9, 0.22))),
        MeshMaterial3d(common::glow_material(
            &mut materials,
            Color::srgb(0.3, 0.7, 1.0),
        )),
        Transform::from_xyz(2.5, 1.5, 0.0),
    ));
}

fn animate(
    time: Res<Time>,
    mut swords: Query<&mut Transform, (With<SwordBlade>, Without<AxeBlade>)>,
    mut axes: Query<&mut Transform, (With<AxeBlade>, Without<SwordBlade>)>,
) {
    let t = time.elapsed_secs();

    // Sword: fast circular swings
    for mut transform in &mut swords {
        let angle = (t * 3.0).sin() * 1.8;
        let radius = 2.0;
        transform.translation = Vec3::new(
            -2.5 + angle.cos() * radius,
            1.5 + angle.sin() * 0.6,
            angle.sin() * radius * 0.4,
        );
        transform.rotation =
            Quat::from_rotation_z(angle * 2.0) * Quat::from_rotation_y(-angle * 0.8);
    }

    // Axe: slower heavy swings
    for mut transform in &mut axes {
        let angle = (t * 2.0).sin() * 2.2;
        let radius = 1.8;
        transform.translation = Vec3::new(
            2.5 + angle.sin() * radius,
            1.5 + (angle * 0.6).cos() * 0.4,
            angle.cos() * radius * 0.3,
        );
        transform.rotation =
            Quat::from_rotation_z(angle * 1.5) * Quat::from_rotation_x(angle * 0.5);
    }
}
