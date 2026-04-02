use saddle_rendering_trail_example_common as common;

use bevy::prelude::*;
use saddle_rendering_trail::{Trail, TrailDiagnostics, TrailEmitterMode, TrailPlugin, TrailStyle};

#[derive(Component)]
struct StressMover {
    radius: f32,
    speed: f32,
    height_phase: f32,
}

fn main() {
    let mut app = App::new();
    app.add_plugins((
        common::default_plugins("trail stress"),
        TrailPlugin::default(),
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, (animate, log_diagnostics));
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
        "Stress",
        "Many simultaneous world-space trails for rough scaling checks.",
    );

    for index in 0..96 {
        let x = (index % 12) as f32 - 5.5;
        let z = (index / 12) as f32 - 3.5;
        commands.spawn((
            Name::new(format!("Stress Mover {index}")),
            StressMover {
                radius: 0.35 + (index % 6) as f32 * 0.08,
                speed: 0.8 + (index % 9) as f32 * 0.12,
                height_phase: index as f32 * 0.3,
            },
            Trail::default()
                .with_emitter_mode(TrailEmitterMode::Always)
                .with_lifetime_secs(0.8)
                .with_style(TrailStyle {
                    base_width: 0.14,
                    ..TrailStyle::default()
                }),
            Mesh3d(meshes.add(Sphere::new(0.08).mesh().uv(10, 6))),
            MeshMaterial3d(common::glow_material(
                &mut materials,
                Color::srgb(0.7, 0.9, 1.0),
            )),
            Transform::from_xyz(x * 1.2, 0.8, z * 1.2),
        ));
    }
}

fn animate(time: Res<Time>, mut movers: Query<(&StressMover, &mut Transform)>) {
    for (mover, mut transform) in &mut movers {
        let t = time.elapsed_secs() * mover.speed;
        transform.translation.x += t.cos() * mover.radius * 0.01;
        transform.translation.z += t.sin() * mover.radius * 0.01;
        transform.translation.y = 0.9 + (t + mover.height_phase).sin() * 0.35;
    }
}

fn log_diagnostics(time: Res<Time>, diagnostics: Res<TrailDiagnostics>, mut once: Local<bool>) {
    if !*once && time.elapsed_secs() > 1.0 {
        *once = true;
        info!(
            "stress: sources={}, renders={}, points={}, rebuilds={}",
            diagnostics.active_sources,
            diagnostics.active_render_entities,
            diagnostics.active_points,
            diagnostics.total_mesh_rebuilds
        );
    }
}
