#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;

use saddle_rendering_trail_example_common as common;

use std::fmt::Write as _;

use bevy::prelude::*;
#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
use bevy_brp_extras::BrpExtrasPlugin;
use saddle_rendering_trail::{
    Trail, TrailColorKey, TrailDiagnostics, TrailEmitterMode, TrailGradient, TrailMaterial,
    TrailOrientation, TrailPlugin, TrailScalarCurve, TrailScalarKey, TrailSpace, TrailStyle,
    TrailSystems, TrailUvMode,
};

#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
const DEFAULT_BRP_PORT: u16 = 15_752;
const LAB_EXIT_ENV: &str = "TRAIL_LAB_EXIT_AFTER_SECONDS";

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum LabSystems {
    Animate,
    Overlay,
}

#[derive(Component)]
pub(crate) struct BillboardSource;

#[derive(Component)]
pub(crate) struct LockedSource;

#[derive(Component)]
pub(crate) struct HoverSource;

#[derive(Component)]
struct HoverCarrier;

#[derive(Component)]
pub(crate) struct TeleportSource;

#[derive(Component)]
pub(crate) struct LabCamera;

#[derive(Component)]
pub(crate) struct OverlayMarker;

#[derive(Component)]
struct ProjectileMotion {
    radius: Vec3,
    speed: f32,
}

#[derive(Component)]
struct LockedMotion {
    radius: f32,
    speed: f32,
}

#[derive(Component)]
struct HoverMotion {
    scale: Vec3,
    speed: f32,
}

#[derive(Component)]
struct HoverCarrierMotion {
    radius: Vec3,
    speed: f32,
}

#[derive(Component)]
struct TeleportMotion {
    anchors: [Vec3; 2],
    active_anchor: usize,
    bob_phase: f32,
    bob_speed: f32,
    timer: Timer,
}

#[derive(Resource, Clone, Copy)]
pub(crate) struct LabEntities {
    #[cfg_attr(not(feature = "e2e"), allow(dead_code))]
    pub billboard: Entity,
    #[cfg_attr(not(feature = "e2e"), allow(dead_code))]
    pub locked: Entity,
    #[cfg_attr(not(feature = "e2e"), allow(dead_code))]
    pub hover: Entity,
    #[cfg_attr(not(feature = "e2e"), allow(dead_code))]
    pub teleporter: Entity,
    #[cfg_attr(not(feature = "e2e"), allow(dead_code))]
    pub camera: Entity,
    pub overlay: Entity,
}

#[derive(Resource)]
struct AutoExitAfter(Timer);

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.02, 0.03, 0.05)))
        .add_plugins((
            common::default_plugins("trail crate-local lab"),
            TrailPlugin::default(),
        ))
        .configure_sets(
            Update,
            (
                LabSystems::Animate,
                TrailSystems::Sample,
                TrailSystems::BuildMesh,
                TrailSystems::Cleanup,
                TrailSystems::Diagnostics,
                LabSystems::Overlay,
            )
                .chain(),
        )
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                animate_billboard,
                animate_locked,
                animate_hover_carrier,
                animate_hover,
                animate_teleporter,
            )
                .in_set(LabSystems::Animate),
        )
        .add_systems(Update, update_overlay.in_set(LabSystems::Overlay));

    #[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
    app.add_plugins(BrpExtrasPlugin::with_port(lab_brp_port()));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::TrailLabE2EPlugin);

    install_lab_auto_exit(&mut app);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    common::spawn_stage_scene(&mut commands, &mut meshes, &mut materials);

    let stripe = common::stripe_texture(&mut images);

    let camera = commands
        .spawn((
            Name::new("Lab Camera"),
            LabCamera,
            Camera3d::default(),
            Transform::from_xyz(0.0, 5.4, 14.0).looking_at(Vec3::new(0.0, 1.6, 0.0), Vec3::Y),
        ))
        .id();

    let billboard = commands
        .spawn((
            Name::new("Billboard Contrail Source"),
            BillboardSource,
            ProjectileMotion {
                radius: Vec3::new(3.6, 0.8, 2.0),
                speed: 1.25,
            },
            Trail::default()
                .with_emitter_mode(TrailEmitterMode::Always)
                .with_style(TrailStyle {
                    base_width: 0.2,
                    uv_mode: TrailUvMode::RepeatByDistance { distance: 0.4 },
                    material: TrailMaterial {
                        texture: Some(stripe.clone()),
                        base_color: Color::srgb(1.0, 0.7, 0.25),
                        emissive: LinearRgba::rgb(0.8, 0.35, 0.12),
                        ..default()
                    },
                    ..default()
                }),
            Mesh3d(meshes.add(Capsule3d::new(0.12, 0.4))),
            MeshMaterial3d(common::glow_material(
                &mut materials,
                Color::srgb(1.0, 0.72, 0.3),
            )),
            Transform::from_xyz(-4.5, 1.6, 0.0),
        ))
        .id();

    let locked = commands
        .spawn((
            Name::new("Locked Swipe Source"),
            LockedSource,
            LockedMotion {
                radius: 2.4,
                speed: 1.45,
            },
            Trail::default()
                .with_lifetime_secs(0.35)
                .with_orientation(TrailOrientation::TransformLocked { axis: Vec3::Y })
                .with_style(TrailStyle {
                    base_width: 0.95,
                    width_over_length: TrailScalarCurve::new([
                        TrailScalarKey::new(0.0, 0.0),
                        TrailScalarKey::new(0.15, 0.55),
                        TrailScalarKey::new(1.0, 1.2),
                    ]),
                    color_over_length: TrailGradient::new([
                        TrailColorKey::new(0.0, Color::srgba(0.15, 0.7, 1.0, 0.0)),
                        TrailColorKey::new(0.4, Color::srgb(0.35, 0.85, 1.0)),
                        TrailColorKey::new(1.0, Color::srgb(1.0, 1.0, 1.0)),
                    ]),
                    ..default()
                }),
            Mesh3d(meshes.add(Cuboid::new(0.12, 1.4, 0.18))),
            MeshMaterial3d(common::glow_material(
                &mut materials,
                Color::srgb(0.55, 0.95, 1.0),
            )),
            Transform::from_xyz(0.0, 1.6, 0.0),
        ))
        .id();

    let mut hover = Entity::PLACEHOLDER;
    commands
        .spawn((
            Name::new("Hover Carrier"),
            HoverCarrier,
            HoverCarrierMotion {
                radius: Vec3::new(4.3, 0.35, 1.4),
                speed: 0.55,
            },
            Mesh3d(meshes.add(Cuboid::new(0.9, 0.12, 1.2))),
            MeshMaterial3d(common::glow_material(
                &mut materials,
                Color::srgb(0.22, 0.55, 0.7),
            )),
            Transform::from_xyz(4.3, 1.2, 0.0),
        ))
        .with_children(|parent| {
            hover = parent
                .spawn((
                    Name::new("Hover Wake Source"),
                    HoverSource,
                    HoverMotion {
                        scale: Vec3::new(1.2, 0.45, 1.4),
                        speed: 0.9,
                    },
                    Trail::default()
                        .with_emitter_mode(TrailEmitterMode::Always)
                        .with_lifetime_secs(1.35)
                        .with_space(TrailSpace::Local)
                        .with_style(TrailStyle {
                            base_width: 0.28,
                            material: TrailMaterial {
                                texture: Some(stripe.clone()),
                                base_color: Color::srgb(0.45, 0.9, 1.0),
                                ..default()
                            },
                            ..default()
                        }),
                    Mesh3d(meshes.add(Torus::new(0.35, 0.11))),
                    MeshMaterial3d(common::glow_material(
                        &mut materials,
                        Color::srgb(0.4, 0.88, 1.0),
                    )),
                    Transform::from_xyz(0.0, 0.0, 0.0),
                ))
                .id();
        });

    let teleporter = commands
        .spawn((
            Name::new("Teleport Reset Source"),
            TeleportSource,
            TeleportMotion {
                anchors: [Vec3::new(-2.8, 1.0, -4.0), Vec3::new(2.8, 1.0, -4.0)],
                active_anchor: 0,
                bob_phase: 0.0,
                bob_speed: 4.0,
                timer: Timer::from_seconds(0.95, TimerMode::Repeating),
            },
            Trail::default()
                .with_emitter_mode(TrailEmitterMode::Always)
                .with_lifetime_secs(0.9)
                .with_style(TrailStyle {
                    base_width: 0.24,
                    material: TrailMaterial {
                        texture: Some(stripe),
                        base_color: Color::srgb(1.0, 0.35, 0.72),
                        emissive: LinearRgba::rgb(0.9, 0.22, 0.55),
                        ..default()
                    },
                    ..default()
                }),
            Mesh3d(meshes.add(Sphere::new(0.18).mesh().uv(18, 10))),
            MeshMaterial3d(common::glow_material(
                &mut materials,
                Color::srgb(1.0, 0.38, 0.76),
            )),
            Transform::from_translation(Vec3::new(-2.8, 1.0, -4.0)),
        ))
        .id();

    let overlay = commands
        .spawn((
            Name::new("Trail Lab Overlay"),
            OverlayMarker,
            Node {
                position_type: PositionType::Absolute,
                left: px(18.0),
                top: px(18.0),
                width: px(480.0),
                padding: UiRect::all(px(14.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.05, 0.08, 0.82)),
            Text::new("Trail Lab"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .id();

    commands.insert_resource(LabEntities {
        billboard,
        locked,
        hover,
        teleporter,
        camera,
        overlay,
    });
}

fn animate_billboard(
    time: Res<Time>,
    mut movers: Query<(&ProjectileMotion, &mut Transform), With<BillboardSource>>,
) {
    for (motion, mut transform) in &mut movers {
        let t = time.elapsed_secs() * motion.speed;
        transform.translation = Vec3::new(
            -4.2 + t.cos() * motion.radius.x,
            1.6 + (t * 2.2).sin() * motion.radius.y,
            t.sin() * motion.radius.z,
        );
        transform.rotation = Quat::from_rotation_y(-t);
    }
}

fn animate_locked(
    time: Res<Time>,
    mut movers: Query<(&LockedMotion, &mut Transform), With<LockedSource>>,
) {
    for (motion, mut transform) in &mut movers {
        let t = time.elapsed_secs() * motion.speed;
        transform.translation = Vec3::new(
            t.cos() * motion.radius,
            1.6 + (t * 1.7).sin() * 0.4,
            t.sin() * motion.radius,
        );
        transform.rotation = Quat::from_rotation_z(t * 2.4) * Quat::from_rotation_y(-t * 1.2);
    }
}

fn animate_hover(
    time: Res<Time>,
    mut movers: Query<(&HoverMotion, &mut Transform), With<HoverSource>>,
) {
    for (motion, mut transform) in &mut movers {
        let t = time.elapsed_secs() * motion.speed;
        transform.translation = Vec3::new(
            t.sin() * motion.scale.x,
            (t * 2.4).cos() * motion.scale.y,
            (t * 0.5).sin() * (t).cos() * motion.scale.z,
        );
        transform.rotation = Quat::from_rotation_y(t * 1.2);
    }
}

fn animate_hover_carrier(
    time: Res<Time>,
    mut carriers: Query<(&HoverCarrierMotion, &mut Transform), With<HoverCarrier>>,
) {
    for (motion, mut transform) in &mut carriers {
        let t = time.elapsed_secs() * motion.speed;
        transform.translation = Vec3::new(
            motion.radius.x + (t * 0.8).sin() * 0.6,
            1.2 + t.sin() * motion.radius.y,
            t.cos() * motion.radius.z,
        );
        transform.rotation = Quat::from_rotation_y(t * 0.9);
    }
}

fn animate_teleporter(
    time: Res<Time>,
    mut movers: Query<(&mut TeleportMotion, &mut Transform), With<TeleportSource>>,
) {
    for (mut motion, mut transform) in &mut movers {
        if motion.timer.tick(time.delta()).just_finished() {
            motion.active_anchor = 1 - motion.active_anchor;
        }
        motion.bob_phase += time.delta_secs() * motion.bob_speed;
        let anchor = motion.anchors[motion.active_anchor];
        transform.translation = anchor + Vec3::new(0.0, motion.bob_phase.sin() * 0.25, 0.0);
    }
}

fn update_overlay(
    diagnostics: Res<TrailDiagnostics>,
    lab: Res<LabEntities>,
    mut overlays: Query<&mut Text, With<OverlayMarker>>,
) {
    let Ok(mut text) = overlays.get_mut(lab.overlay) else {
        return;
    };

    let mut body = String::from("Trail Lab\n");
    let _ = writeln!(
        &mut body,
        "sources={} renders={} points={} resets={} rebuilds={}",
        diagnostics.active_sources,
        diagnostics.active_render_entities,
        diagnostics.active_points,
        diagnostics.total_resets,
        diagnostics.total_mesh_rebuilds,
    );
    let _ = writeln!(
        &mut body,
        "billboard, locked, local-space hover, and teleport-reset modes run side by side."
    );
    **text = body;
}

fn install_lab_auto_exit(app: &mut App) {
    if let Some(seconds) = std::env::var(LAB_EXIT_ENV)
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|seconds| *seconds > 0.0)
    {
        app.insert_resource(AutoExitAfter(Timer::from_seconds(seconds, TimerMode::Once)))
            .add_systems(Update, auto_exit_after);
    }
}

fn auto_exit_after(
    time: Res<Time>,
    mut timer: ResMut<AutoExitAfter>,
    mut exit: MessageWriter<AppExit>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        exit.write(AppExit::Success);
    }
}

#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
fn lab_brp_port() -> u16 {
    std::env::var("TRAIL_LAB_BRP_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(DEFAULT_BRP_PORT)
}
