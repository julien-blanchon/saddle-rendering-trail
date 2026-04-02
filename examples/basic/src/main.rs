use saddle_rendering_trail_example_common as common;

use bevy::prelude::*;
use saddle_rendering_trail::{Trail, TrailEmitterMode, TrailPlugin, TrailStyle};

#[derive(Component)]
struct Mover;

fn main() {
    let mut app = App::new();
    app.add_plugins((common::default_plugins("trail basic"), TrailPlugin::default()));
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
        "Basic Trail",
        "A single emitter sweeps a world-space billboard ribbon.",
    );

    commands.spawn((
        Name::new("Basic Mover"),
        Mover,
        Trail::default().with_emitter_mode(TrailEmitterMode::Always).with_style(TrailStyle {
            base_width: 0.45,
            ..default()
        }),
        Mesh3d(meshes.add(Sphere::new(0.28).mesh().uv(20, 12))),
        MeshMaterial3d(common::glow_material(
            &mut materials,
            Color::srgb(0.62, 0.82, 1.0),
        )),
        Transform::from_xyz(-3.0, 1.2, 0.0),
    ));
}

fn animate(time: Res<Time>, mut movers: Query<&mut Transform, With<Mover>>) {
    for mut transform in &mut movers {
        let t = time.elapsed_secs();
        transform.translation = Vec3::new(t.sin() * 3.4, 1.2 + (t * 2.2).sin() * 0.6, t.cos() * 1.2);
    }
}
