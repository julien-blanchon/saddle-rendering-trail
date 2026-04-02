use saddle_rendering_trail_example_common as common;

use bevy::prelude::*;
use saddle_rendering_saddle_rendering_trail::{Trail, TrailEmitterMode, TrailPlugin, TrailScalarCurve, TrailStyle, TrailUvMode};

#[derive(Component)]
struct Projectile;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        common::default_plugins("trail projectile contrail"),
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
    mut images: ResMut<Assets<Image>>,
) {
    common::spawn_stage(&mut commands, &mut meshes, &mut materials);
    common::spawn_overlay(
        &mut commands,
        "Projectile Contrail",
        "Repeat-by-distance UVs with a tight narrow ribbon.",
    );

    let stripe = common::stripe_texture(&mut images);
    commands.spawn((
        Name::new("Projectile"),
        Projectile,
        Trail::default()
            .with_emitter_mode(TrailEmitterMode::Always)
            .with_style(TrailStyle {
                base_width: 0.18,
                alpha_over_age: TrailScalarCurve::linear(1.0, 0.0),
                uv_mode: TrailUvMode::RepeatByDistance { distance: 0.35 },
                material: saddle_rendering_trail::TrailMaterial {
                    texture: Some(stripe),
                    base_color: Color::srgb(1.0, 0.74, 0.34),
                    emissive: LinearRgba::rgb(0.8, 0.4, 0.15),
                    ..default()
                },
                ..default()
            }),
        Mesh3d(meshes.add(Capsule3d::new(0.12, 0.4))),
        MeshMaterial3d(common::glow_material(
            &mut materials,
            Color::srgb(1.0, 0.66, 0.2),
        )),
        Transform::from_xyz(-5.0, 1.5, 0.0),
    ));
}

fn animate(time: Res<Time>, mut movers: Query<&mut Transform, With<Projectile>>) {
    for mut transform in &mut movers {
        let t = time.elapsed_secs();
        transform.translation = Vec3::new(-5.0 + (t * 4.0).rem_euclid(10.0), 1.5 + (t * 5.0).sin() * 0.3, 0.0);
    }
}
