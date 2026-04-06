use saddle_rendering_trail_example_common as common;

use bevy::prelude::*;
use saddle_rendering_trail::{Trail, TrailOrientation, TrailPlugin, TrailStyle};

#[derive(Component)]
struct BillboardTrail;

#[derive(Component)]
struct LockedTrail;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        common::default_plugins("trail orientation modes"),
        TrailPlugin::default(),
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, animate);
    common::install_auto_exit(&mut app);
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
        "Orientation Modes",
        "Left: camera-facing billboard. Right: transform-locked local axis.",
    );

    commands.spawn((
        Name::new("Billboard Example"),
        BillboardTrail,
        Trail::default().with_style(TrailStyle {
            base_width: 0.42,
            material: saddle_rendering_trail::TrailMaterial {
                base_color: Color::srgb(0.9, 0.4, 0.2),
                ..default()
            },
            ..common::showcase_trail_style()
        }),
        Mesh3d(meshes.add(Sphere::new(0.24).mesh().uv(16, 10))),
        MeshMaterial3d(common::glow_material(
            &mut materials,
            Color::srgb(0.92, 0.42, 0.22),
        )),
        Transform::from_xyz(-3.0, 1.3, 0.0),
    ));

    commands.spawn((
        Name::new("Locked Example"),
        LockedTrail,
        Trail::default()
            .with_orientation(TrailOrientation::TransformLocked { axis: Vec3::Y })
            .with_style(TrailStyle {
                base_width: 0.42,
                material: saddle_rendering_trail::TrailMaterial {
                    base_color: Color::srgb(0.3, 0.85, 1.0),
                    ..default()
                },
                ..common::showcase_trail_style()
            }),
        Mesh3d(meshes.add(Cuboid::new(0.22, 0.8, 0.22))),
        MeshMaterial3d(common::glow_material(
            &mut materials,
            Color::srgb(0.34, 0.86, 1.0),
        )),
        Transform::from_xyz(3.0, 1.3, 0.0),
    ));
}

fn animate(
    time: Res<Time>,
    mut billboard: Query<&mut Transform, (With<BillboardTrail>, Without<LockedTrail>)>,
    mut locked: Query<&mut Transform, (With<LockedTrail>, Without<BillboardTrail>)>,
) {
    let t = time.elapsed_secs();
    for mut transform in &mut billboard {
        transform.translation = Vec3::new(
            -3.0 + (t * 1.4).sin() * 1.7,
            1.3 + (t * 2.0).cos() * 0.7,
            (t * 1.1).cos(),
        );
    }
    for mut transform in &mut locked {
        transform.translation = Vec3::new(
            3.0 + (t * 1.4).sin() * 1.7,
            1.3 + (t * 1.7).sin() * 0.7,
            (t * 1.2).cos(),
        );
        transform.rotation = Quat::from_rotation_x(t * 1.7) * Quat::from_rotation_z(t * 0.9);
    }
}
