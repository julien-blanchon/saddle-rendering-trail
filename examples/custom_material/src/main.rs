//! Demonstrates custom material integration via `TrailMaterial3d<M>` and
//! `TrailMaterialPlugin<M>`.
//!
//! Three trails orbit side by side:
//! - **Left**: standard `TrailMaterial` (baseline — no custom material)
//! - **Center**: `ExtendedMaterial` with an energy-trail fragment shader
//!   (hot core, glowing edges, vertex-alpha-aware)
//! - **Right**: hot-swapped `TrailCustomMaterial` with a vivid emissive
//!
//! This shows the three material paths the trail crate supports.

use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
};
use saddle_rendering_trail::{
    Trail, TrailColorKey, TrailCustomMaterial, TrailEmitterMode, TrailGradient, TrailMaterial,
    TrailMaterial3d, TrailMaterialPlugin, TrailOrientation, TrailPlugin, TrailScalarCurve,
    TrailScalarKey, TrailStyle,
};
use saddle_rendering_trail_example_common as common;

// ---------------------------------------------------------------------------
// Plugin & main
// ---------------------------------------------------------------------------

fn main() {
    let mut app = App::new();
    app.add_plugins(common::default_plugins("Trail — Custom Materials"));
    app.add_plugins(TrailPlugin::default());

    // Register the extended material pipeline.
    app.add_plugins(
        MaterialPlugin::<ExtendedMaterial<StandardMaterial, EnergyTrailExtension>>::default(),
    );
    app.add_plugins(
        TrailMaterialPlugin::<ExtendedMaterial<StandardMaterial, EnergyTrailExtension>>::new(
            Update,
        ),
    );

    common::install_auto_exit(&mut app);
    app.add_systems(Startup, setup);
    app.add_systems(Update, animate);
    app.run();
}

// ---------------------------------------------------------------------------
// Custom shader material — energy trail with glowing core
// ---------------------------------------------------------------------------

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
struct EnergyTrailExtension {
    #[uniform(100)]
    params: EnergyTrailParams,
}

#[derive(Debug, Clone, ShaderType)]
struct EnergyTrailParams {
    edge_color: LinearRgba,
    core_color: LinearRgba,
    pulse_speed: f32,
    edge_sharpness: f32,
    _padding: Vec2,
}

impl MaterialExtension for EnergyTrailExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/energy_trail.wgsl".into()
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
struct Orbiter {
    radius: f32,
    speed: f32,
    phase: f32,
    center: Vec3,
}

// ---------------------------------------------------------------------------
// Setup — three trails, three material paths
// ---------------------------------------------------------------------------

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut energy_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, EnergyTrailExtension>>,
    >,
    _asset_server: Res<AssetServer>,
) {
    common::spawn_stage_scene(&mut commands, &mut meshes, &mut materials);
    common::spawn_overlay(
        &mut commands,
        "Trail — Custom Materials",
        "Left: TrailMaterial  |  Center: ExtendedMaterial (energy shader)  |  Right: TrailCustomMaterial (emissive)",
    );

    // Wide camera to frame all three trails.
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 6.0, 14.0).looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y),
    ));

    let orb_mesh = meshes.add(Sphere::new(0.12));

    // ---- LEFT: baseline TrailMaterial (no custom material) ----
    let left_center = Vec3::new(-4.0, 1.5, 0.0);
    commands.spawn((
        Name::new("Standard Trail"),
        Orbiter {
            radius: 1.8,
            speed: 1.2,
            phase: 0.0,
            center: left_center,
        },
        Trail {
            emitter_mode: TrailEmitterMode::Always,
            lifetime_secs: 1.2,
            max_points: 56,
            style: TrailStyle {
                base_width: 0.35,
                width_over_length: TrailScalarCurve::linear(0.3, 1.0),
                color_over_length: TrailGradient::new([
                    TrailColorKey::new(0.0, Color::srgb(0.4, 0.5, 0.8)),
                    TrailColorKey::new(1.0, Color::WHITE),
                ]),
                alpha_over_length: TrailScalarCurve::new([
                    TrailScalarKey::new(0.0, 0.0),
                    TrailScalarKey::new(0.15, 0.5),
                    TrailScalarKey::new(1.0, 1.0),
                ]),
                material: TrailMaterial {
                    emissive: LinearRgba::new(0.15, 0.2, 0.5, 1.0),
                    ..default()
                },
                ..default()
            },
            ..default()
        },
        Mesh3d(orb_mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.6, 1.0),
            emissive: LinearRgba::new(0.2, 0.3, 0.8, 1.0),
            ..default()
        })),
        Transform::from_translation(left_center),
    ));

    // ---- CENTER: ExtendedMaterial with energy shader ----
    let center = Vec3::new(0.0, 1.5, 0.0);
    let energy_mat = energy_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: Color::WHITE,
            emissive: LinearRgba::new(0.5, 0.8, 1.0, 1.0),
            unlit: true,
            double_sided: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        },
        extension: EnergyTrailExtension {
            params: EnergyTrailParams {
                edge_color: LinearRgba::new(0.05, 0.2, 1.0, 0.6),
                core_color: LinearRgba::new(0.7, 0.9, 1.0, 1.0),
                pulse_speed: 4.0,
                edge_sharpness: 3.0,
                _padding: Vec2::ZERO,
            },
        },
    });

    commands.spawn((
        Name::new("Energy Trail"),
        Orbiter {
            radius: 2.0,
            speed: 1.5,
            phase: 0.0,
            center,
        },
        Trail {
            emitter_mode: TrailEmitterMode::Always,
            lifetime_secs: 1.0,
            max_points: 64,
            min_sample_distance: 0.1,
            style: TrailStyle {
                base_width: 0.5,
                width_over_length: TrailScalarCurve::new([
                    TrailScalarKey::new(0.0, 0.0),
                    TrailScalarKey::new(0.1, 0.6),
                    TrailScalarKey::new(0.5, 1.0),
                    TrailScalarKey::new(1.0, 0.8),
                ]),
                alpha_over_length: TrailScalarCurve::new([
                    TrailScalarKey::new(0.0, 0.0),
                    TrailScalarKey::new(0.05, 0.8),
                    TrailScalarKey::new(1.0, 1.0),
                ]),
                ..default()
            },
            ..default()
        },
        TrailMaterial3d(energy_mat),
        Mesh3d(orb_mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.8, 1.0),
            emissive: LinearRgba::new(0.4, 0.7, 1.0, 1.0),
            ..default()
        })),
        Transform::from_translation(center),
    ));

    // ---- RIGHT: TrailCustomMaterial (vivid emissive StandardMaterial) ----
    let right_center = Vec3::new(4.0, 1.5, 0.0);
    let hot_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.3, 0.05),
        emissive: LinearRgba::new(2.0, 0.6, 0.1, 1.0),
        unlit: true,
        double_sided: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn((
        Name::new("Emissive Trail"),
        Orbiter {
            radius: 1.6,
            speed: 1.8,
            phase: 1.0,
            center: right_center,
        },
        Trail {
            emitter_mode: TrailEmitterMode::Always,
            orientation: TrailOrientation::TransformLocked { axis: Vec3::Y },
            lifetime_secs: 0.8,
            max_points: 48,
            style: TrailStyle {
                base_width: 0.6,
                width_over_length: TrailScalarCurve::new([
                    TrailScalarKey::new(0.0, 0.0),
                    TrailScalarKey::new(0.2, 1.2),
                    TrailScalarKey::new(1.0, 0.7),
                ]),
                color_over_length: TrailGradient::new([
                    TrailColorKey::new(0.0, Color::srgb(1.0, 0.1, 0.0)),
                    TrailColorKey::new(0.4, Color::srgb(1.0, 0.5, 0.0)),
                    TrailColorKey::new(1.0, Color::srgb(1.0, 0.9, 0.6)),
                ]),
                alpha_over_length: TrailScalarCurve::new([
                    TrailScalarKey::new(0.0, 0.0),
                    TrailScalarKey::new(0.1, 0.8),
                    TrailScalarKey::new(1.0, 1.0),
                ]),
                ..default()
            },
            ..default()
        },
        TrailCustomMaterial(hot_material),
        Mesh3d(orb_mesh),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.4, 0.1),
            emissive: LinearRgba::new(1.5, 0.4, 0.05, 1.0),
            ..default()
        })),
        Transform::from_translation(right_center),
    ));
}

// ---------------------------------------------------------------------------
// Animation — each orbiter follows its own circular path
// ---------------------------------------------------------------------------

fn animate(time: Res<Time>, mut orbiters: Query<(&Orbiter, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (orbiter, mut transform) in &mut orbiters {
        let angle = t * orbiter.speed + orbiter.phase;
        transform.translation.x = orbiter.center.x + angle.cos() * orbiter.radius;
        transform.translation.z = orbiter.center.z + angle.sin() * orbiter.radius;
        transform.translation.y = orbiter.center.y + (t * 2.0 + orbiter.phase).sin() * 0.4;
    }
}
