use saddle_rendering_trail_example_common as common;

use bevy::prelude::*;
use saddle_rendering_saddle_rendering_trail::{
    Trail, TrailEmitterMode, TrailPlugin, TrailScalarCurve, TrailSpace, TrailStyle,
};

#[derive(Component)]
struct Carrier;

#[derive(Component)]
struct WorldSpaceEmitter;

#[derive(Component)]
struct LocalSpaceEmitter;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        common::default_plugins("trail space modes"),
        TrailPlugin::default(),
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, (animate_carrier, animate_emitters));
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
        "Space Modes",
        "Orange stays in world space. Cyan follows the moving parent in local space.",
    );

    let carrier = commands
        .spawn((
            Name::new("Carrier"),
            Carrier,
            Mesh3d(meshes.add(Cuboid::new(1.4, 0.18, 1.8))),
            MeshMaterial3d(common::glow_material(
                &mut materials,
                Color::srgb(0.85, 0.88, 0.94),
            )),
            Transform::from_xyz(0.0, 1.4, 0.0),
        ))
        .id();

    commands.entity(carrier).with_children(|parent| {
        parent.spawn((
            Name::new("World Space Emitter"),
            WorldSpaceEmitter,
            Trail::default()
                .with_emitter_mode(TrailEmitterMode::Always)
                .with_space(TrailSpace::World)
                .with_style(TrailStyle {
                    base_width: 0.22,
                    alpha_over_age: TrailScalarCurve::linear(1.0, 0.0),
                    material: saddle_rendering_trail::TrailMaterial {
                        base_color: Color::srgb(1.0, 0.6, 0.22),
                        emissive: LinearRgba::rgb(0.75, 0.28, 0.08),
                        ..default()
                    },
                    ..default()
                }),
            Mesh3d(meshes.add(Sphere::new(0.16).mesh().uv(18, 12))),
            MeshMaterial3d(common::glow_material(
                &mut materials,
                Color::srgb(1.0, 0.64, 0.24),
            )),
            Transform::from_xyz(-0.9, 0.0, 0.0),
        ));

        parent.spawn((
            Name::new("Local Space Emitter"),
            LocalSpaceEmitter,
            Trail::default()
                .with_emitter_mode(TrailEmitterMode::Always)
                .with_space(TrailSpace::Local)
                .with_style(TrailStyle {
                    base_width: 0.22,
                    alpha_over_age: TrailScalarCurve::linear(1.0, 0.0),
                    material: saddle_rendering_trail::TrailMaterial {
                        base_color: Color::srgb(0.32, 0.88, 1.0),
                        emissive: LinearRgba::rgb(0.18, 0.45, 0.7),
                        ..default()
                    },
                    ..default()
                }),
            Mesh3d(meshes.add(Sphere::new(0.16).mesh().uv(18, 12))),
            MeshMaterial3d(common::glow_material(
                &mut materials,
                Color::srgb(0.34, 0.86, 1.0),
            )),
            Transform::from_xyz(0.9, 0.0, 0.0),
        ));
    });
}

fn animate_carrier(time: Res<Time>, mut carriers: Query<&mut Transform, With<Carrier>>) {
    for mut transform in &mut carriers {
        let t = time.elapsed_secs() * 0.65;
        transform.translation =
            Vec3::new(t.cos() * 3.2, 1.4 + (t * 1.8).sin() * 0.25, t.sin() * 1.6);
        transform.rotation = Quat::from_rotation_y(-t * 0.9);
    }
}

fn animate_emitters(
    time: Res<Time>,
    mut world_emitters: Query<
        &mut Transform,
        (With<WorldSpaceEmitter>, Without<LocalSpaceEmitter>),
    >,
    mut local_emitters: Query<
        &mut Transform,
        (With<LocalSpaceEmitter>, Without<WorldSpaceEmitter>),
    >,
) {
    let t = time.elapsed_secs() * 2.4;

    for mut transform in &mut world_emitters {
        transform.translation = Vec3::new(
            -0.9 + t.sin() * 0.45,
            (t * 1.6).cos() * 0.28,
            (t * 0.8).sin() * 0.5,
        );
    }

    for mut transform in &mut local_emitters {
        transform.translation = Vec3::new(
            0.9 + (t + 1.2).sin() * 0.45,
            ((t + 0.7) * 1.7).cos() * 0.28,
            ((t + 0.4) * 0.85).sin() * 0.5,
        );
    }
}
