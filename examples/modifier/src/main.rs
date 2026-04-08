use bevy::prelude::*;
use saddle_rendering_trail::{
    Trail, TrailColorKey, TrailEmitterMode, TrailGradient, TrailHistory, TrailOrientation,
    TrailPlugin, TrailScalarCurve, TrailScalarKey, TrailStyle, TrailSystems,
};
use saddle_rendering_trail_example_common as common;

fn main() {
    let mut app = App::new();
    app.add_plugins(common::default_plugins("Trail Modifier"));
    app.add_plugins(TrailPlugin::default());
    common::install_auto_exit(&mut app);
    app.add_systems(Startup, setup);
    app.add_systems(Update, animate);
    app.add_systems(
        Update,
        wave_modifier.in_set(TrailSystems::Modify),
    );
    app.run();
}

#[derive(Component)]
struct Mover;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    common::spawn_stage(&mut commands, &mut meshes, &mut materials);
    common::spawn_overlay(
        &mut commands,
        "Trail Modifier",
        "Sine-wave displacement applied in TrailSystems::Modify\nvia TrailHistory::points_mut()",
    );

    // Spawn trail source with locked orientation so wave is visible
    commands.spawn((
        Name::new("Wavy Trail"),
        Mover,
        Trail {
            emitter_mode: TrailEmitterMode::Always,
            orientation: TrailOrientation::TransformLocked { axis: Vec3::Y },
            lifetime_secs: 2.0,
            max_points: 64,
            min_sample_distance: 0.08,
            style: TrailStyle {
                base_width: 0.5,
                width_over_length: TrailScalarCurve::linear(0.3, 1.0),
                color_over_length: TrailGradient::new([
                    TrailColorKey::new(0.0, Color::srgb(0.3, 0.5, 1.0)),
                    TrailColorKey::new(0.5, Color::srgb(0.6, 0.9, 1.0)),
                    TrailColorKey::new(1.0, Color::WHITE),
                ]),
                alpha_over_length: TrailScalarCurve::new([
                    TrailScalarKey::new(0.0, 0.0),
                    TrailScalarKey::new(0.1, 0.6),
                    TrailScalarKey::new(1.0, 1.0),
                ]),
                ..default()
            },
            ..default()
        },
        Mesh3d(meshes.add(Sphere::new(0.12))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.8, 1.0),
            emissive: LinearRgba::new(0.3, 0.5, 1.0, 1.0),
            ..default()
        })),
        Transform::from_xyz(0.0, 1.5, 0.0),
    ));
}

fn animate(time: Res<Time>, mut movers: Query<&mut Transform, With<Mover>>) {
    let t = time.elapsed_secs();
    for mut transform in &mut movers {
        transform.translation.x = t.sin() * 3.0;
        transform.translation.z = (t * 0.7).cos() * 2.0;
        transform.translation.y = 1.5 + (t * 1.5).sin() * 0.3;
    }
}

/// User modifier system running in `TrailSystems::Modify`.
/// Displaces trail points with a sine wave perpendicular to the trail path.
fn wave_modifier(time: Res<Time>, mut histories: Query<&mut TrailHistory>) {
    let t = time.elapsed_secs();
    for mut history in &mut histories {
        if history.len() < 2 {
            continue;
        }
        let points = history.points_mut();
        let count = points.len();
        for (i, point) in points.iter_mut().enumerate() {
            let phase = i as f32 * 0.8 + t * 3.0;
            let amplitude = 0.15 * (1.0 - (i as f32 / count as f32));
            point.position.y += phase.sin() * amplitude;
        }
    }
}
