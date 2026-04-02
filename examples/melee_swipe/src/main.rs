use saddle_rendering_trail_example_common as common;

use bevy::prelude::*;
use saddle_rendering_trail::{Trail, TrailOrientation, TrailPlugin, TrailStyle};

#[derive(Component)]
struct Blade;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        common::default_plugins("trail melee swipe"),
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
        "Melee Swipe",
        "Transform-locked orientation with a wide width curve and short lifetime.",
    );

    commands.spawn((
        Name::new("Blade"),
        Blade,
        Trail::default()
            .with_lifetime_secs(0.35)
            .with_orientation(TrailOrientation::TransformLocked { axis: Vec3::Y })
            .with_style(TrailStyle {
                base_width: 0.8,
                width_over_length: saddle_rendering_trail::TrailScalarCurve::new([
                    saddle_rendering_trail::TrailScalarKey::new(0.0, 0.0),
                    saddle_rendering_trail::TrailScalarKey::new(0.2, 0.8),
                    saddle_rendering_trail::TrailScalarKey::new(1.0, 1.15),
                ]),
                color_over_length: saddle_rendering_trail::TrailGradient::new([
                    saddle_rendering_trail::TrailColorKey::new(
                        0.0,
                        Color::srgba(0.1, 0.6, 1.0, 0.0),
                    ),
                    saddle_rendering_trail::TrailColorKey::new(0.45, Color::srgb(0.4, 0.9, 1.0)),
                    saddle_rendering_trail::TrailColorKey::new(1.0, Color::srgb(1.0, 1.0, 1.0)),
                ]),
                ..default()
            }),
        Mesh3d(meshes.add(Cuboid::new(0.1, 1.2, 0.16))),
        MeshMaterial3d(common::glow_material(
            &mut materials,
            Color::srgb(0.6, 0.9, 1.0),
        )),
        Transform::from_xyz(0.0, 1.2, 0.0),
    ));
}

fn animate(time: Res<Time>, mut movers: Query<&mut Transform, With<Blade>>) {
    for mut transform in &mut movers {
        let t = time.elapsed_secs();
        let radius = 2.4;
        let angle = (t * 2.5).sin() * 1.2;
        transform.translation = Vec3::new(
            angle.cos() * radius,
            1.2 + angle.sin() * 0.5,
            angle.sin() * radius,
        );
        transform.rotation = Quat::from_rotation_z(angle * 1.6) * Quat::from_rotation_y(-angle);
    }
}
