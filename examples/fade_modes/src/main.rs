use saddle_rendering_trail_example_common as common;

use bevy::prelude::*;
use saddle_rendering_trail::{
    Trail, TrailColorKey, TrailEmitterMode, TrailFadeMode, TrailGradient, TrailMaterial,
    TrailPlugin, TrailScalarCurve, TrailStyle,
};

#[derive(Component)]
struct Mover {
    offset: Vec3,
    speed: f32,
}

fn main() {
    let mut app = App::new();
    app.add_plugins((
        common::default_plugins("trail fade modes"),
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
        "Fade Modes Comparison",
        "Left: Alpha fade (default) — opacity decreases\n\
         Centre: Width fade — trail narrows to nothing\n\
         Right: Both fade — opacity + width shrink together",
    );

    let sphere_mesh = meshes.add(Sphere::new(0.22).mesh().uv(18, 10));

    let modes = [
        (
            "Alpha Fade",
            TrailFadeMode::Alpha,
            Vec3::new(-4.0, 1.5, 0.0),
            Color::srgb(1.0, 0.45, 0.25),
        ),
        (
            "Width Fade",
            TrailFadeMode::Width,
            Vec3::new(0.0, 1.5, 0.0),
            Color::srgb(0.3, 0.85, 0.5),
        ),
        (
            "Both Fade",
            TrailFadeMode::Both,
            Vec3::new(4.0, 1.5, 0.0),
            Color::srgb(0.4, 0.6, 1.0),
        ),
    ];

    for (label, fade_mode, offset, color) in modes {
        commands.spawn((
            Name::new(label),
            Mover { offset, speed: 1.2 },
            Trail::default()
                .with_emitter_mode(TrailEmitterMode::Always)
                .with_lifetime_secs(1.0)
                .with_style(TrailStyle {
                    base_width: 0.5,
                    fade_mode,
                    alpha_over_length: TrailScalarCurve::linear(0.0, 1.0),
                    color_over_length: TrailGradient::new([
                        TrailColorKey::new(0.0, color.with_alpha(0.3)),
                        TrailColorKey::new(1.0, color),
                    ]),
                    material: TrailMaterial {
                        emissive: color.to_linear() * 0.5,
                        ..default()
                    },
                    ..default()
                }),
            Mesh3d(sphere_mesh.clone()),
            MeshMaterial3d(common::glow_material(&mut materials, color)),
            Transform::from_translation(offset),
        ));
    }
}

fn animate(time: Res<Time>, mut movers: Query<(&Mover, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (mover, mut transform) in &mut movers {
        transform.translation = mover.offset
            + Vec3::new(
                (t * mover.speed).sin() * 2.5,
                (t * mover.speed * 1.8).cos() * 0.6,
                (t * mover.speed * 0.7).sin() * 1.2,
            );
    }
}
