//! Magic Trail + GPU Particles
//!
//! Two animated sources combining `saddle-rendering-trail` with `bevy_hanabi`:
//! - **Left**: magic orb — spark particles oriented along velocity, HDR colors
//!   with bloom, size/color gradients over lifetime
//! - **Right**: sword swing — ember shower with gravity, drag, and warm-to-cool
//!   color gradient
//!
//! Demonstrates that the trail system composes cleanly with GPU particle
//! effects attached as children of the same moving entity.

use saddle_rendering_trail_example_common as common;

use bevy::{
    core_pipeline::tonemapping::Tonemapping, post_process::bloom::Bloom, prelude::*,
    render::view::Hdr,
};
use bevy_hanabi::prelude::*;
use saddle_rendering_trail::{
    Trail, TrailColorKey, TrailDiagnostics, TrailEmitterMode, TrailFadeMode, TrailGradient,
    TrailMaterial, TrailOrientation, TrailPlugin, TrailScalarCurve, TrailScalarKey, TrailStyle,
};

#[derive(Component)]
struct MagicOrb;

#[derive(Component)]
struct SwordSpark;

#[derive(Component)]
struct OverlayText;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        common::default_plugins("trail magic + particles"),
        TrailPlugin::default(),
        HanabiPlugin,
    ));
    common::install_auto_exit(&mut app);
    app.add_systems(Startup, setup)
        .add_systems(Update, (animate, update_overlay));
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    common::spawn_stage_scene(&mut commands, &mut meshes, &mut materials);

    // HDR camera with bloom for particle glow.
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Camera {
            clear_color: Color::srgb(0.02, 0.02, 0.04).into(),
            ..default()
        },
        Hdr,
        Tonemapping::None,
        Bloom {
            intensity: 0.3,
            ..default()
        },
        Transform::from_xyz(0.0, 5.0, 14.0).looking_at(Vec3::new(0.0, 1.8, 0.0), Vec3::Y),
    ));

    common::spawn_overlay(
        &mut commands,
        "Magic Trail + GPU Particles",
        "Left: magic orb (trail + spark particles)\n\
         Right: sword swing (trail + ember particles)\n\
         HDR camera with bloom for particle glow.",
    );

    // --- Magic orb: trail + glowing spark particles ---
    let orb_spark_effect = effects.add(magic_orb_effect());
    commands
        .spawn((
            Name::new("Magic Orb"),
            MagicOrb,
            Trail::default()
                .with_emitter_mode(TrailEmitterMode::Always)
                .with_lifetime_secs(1.2)
                .with_style(TrailStyle {
                    base_width: 0.35,
                    fade_mode: TrailFadeMode::Both,
                    width_over_length: TrailScalarCurve::linear(0.3, 1.0),
                    color_over_length: TrailGradient::new([
                        TrailColorKey::new(0.0, Color::srgb(0.2, 0.1, 0.8)),
                        TrailColorKey::new(0.4, Color::srgb(0.5, 0.2, 1.0)),
                        TrailColorKey::new(1.0, Color::srgb(0.9, 0.8, 1.0)),
                    ]),
                    alpha_over_length: TrailScalarCurve::linear(0.0, 1.0),
                    material: TrailMaterial {
                        emissive: LinearRgba::rgb(0.8, 0.3, 1.8),
                        ..default()
                    },
                    ..default()
                }),
            Mesh3d(meshes.add(Sphere::new(0.2).mesh().uv(16, 10))),
            MeshMaterial3d(common::glow_material(
                &mut materials,
                Color::srgb(0.5, 0.3, 1.0),
            )),
            Transform::from_xyz(-3.5, 2.0, 0.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Orb Sparks"),
                ParticleEffect::new(orb_spark_effect),
                Transform::IDENTITY,
            ));
        });

    // --- Sword swing: trail + ember particles ---
    let ember_effect = effects.add(sword_ember_effect());
    commands
        .spawn((
            Name::new("Sword Spark"),
            SwordSpark,
            Trail::default()
                .with_lifetime_secs(0.4)
                .with_orientation(TrailOrientation::TransformLocked { axis: Vec3::Y })
                .with_style(TrailStyle {
                    base_width: 0.9,
                    fade_mode: TrailFadeMode::Width,
                    width_over_length: TrailScalarCurve::new([
                        TrailScalarKey::new(0.0, 0.0),
                        TrailScalarKey::new(0.2, 0.7),
                        TrailScalarKey::new(1.0, 1.1),
                    ]),
                    color_over_length: TrailGradient::new([
                        TrailColorKey::new(0.0, Color::srgb(1.0, 0.3, 0.05)),
                        TrailColorKey::new(0.5, Color::srgb(1.0, 0.65, 0.2)),
                        TrailColorKey::new(1.0, Color::srgb(1.0, 0.95, 0.85)),
                    ]),
                    alpha_over_length: TrailScalarCurve::linear(0.0, 1.0),
                    material: TrailMaterial {
                        emissive: LinearRgba::rgb(2.0, 0.6, 0.1),
                        ..default()
                    },
                    ..default()
                }),
            Mesh3d(meshes.add(Cuboid::new(0.06, 1.2, 0.12))),
            MeshMaterial3d(common::glow_material(
                &mut materials,
                Color::srgb(1.0, 0.5, 0.2),
            )),
            Transform::from_xyz(3.0, 1.5, 0.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Sword Embers"),
                ParticleEffect::new(ember_effect),
                Transform::IDENTITY,
            ));
        });

    // Diagnostics overlay
    commands.spawn((
        Name::new("Diagnostics Overlay"),
        OverlayText,
        Node {
            position_type: PositionType::Absolute,
            right: px(16.0),
            top: px(16.0),
            width: px(300.0),
            padding: UiRect::all(px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.05, 0.08, 0.75)),
        Text::new("..."),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

/// Sparkle particles for the magic orb — elongated along velocity, HDR colors
/// with bloom, shrinking over lifetime, brief random burst.
fn magic_orb_effect() -> EffectAsset {
    let writer = ExprWriter::new();

    let age = writer.lit(0.0).expr();
    let lifetime = writer.lit(0.4).uniform(writer.lit(0.8)).expr();
    let drag = writer.lit(3.0).expr();

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.2).expr(),
        dimension: ShapeDimension::Volume,
    };
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(2.0).uniform(writer.lit(4.0)).expr(),
    };

    // HDR color gradient: bright white core → vivid purple → fade out
    let mut color_gradient = bevy_hanabi::Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(3.0, 2.5, 4.0, 1.0));
    color_gradient.add_key(0.3, Vec4::new(1.5, 0.5, 3.0, 1.0));
    color_gradient.add_key(0.7, Vec4::new(0.6, 0.2, 1.5, 0.8));
    color_gradient.add_key(1.0, Vec4::new(0.3, 0.1, 0.8, 0.0));

    // Size gradient: elongated spark shape, shrinking to nothing
    let mut size_gradient = bevy_hanabi::Gradient::new();
    size_gradient.add_key(0.0, Vec3::new(0.12, 0.03, 1.0));
    size_gradient.add_key(0.5, Vec3::new(0.08, 0.02, 1.0));
    size_gradient.add_key(1.0, Vec3::splat(0.0));

    EffectAsset::new(1024, SpawnerSettings::rate(200.0.into()), writer.finish())
        .with_name("magic_orb_sparks")
        .init(init_pos)
        .init(init_vel)
        .init(SetAttributeModifier::new(Attribute::AGE, age))
        .init(SetAttributeModifier::new(Attribute::LIFETIME, lifetime))
        .update(LinearDragModifier::new(drag))
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient,
            screen_space_size: false,
        })
        .render(OrientModifier::new(OrientMode::AlongVelocity))
}

/// Ember shower for the sword — warm-to-cool gradient, gravity pull,
/// drag for deceleration, oriented sparks falling down.
fn sword_ember_effect() -> EffectAsset {
    let writer = ExprWriter::new();

    let age = writer.lit(0.0).expr();
    let lifetime = writer.lit(0.3).uniform(writer.lit(0.7)).expr();

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.35).expr(),
        dimension: ShapeDimension::Surface,
    };
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(2.0).uniform(writer.lit(5.0)).expr(),
    };
    let gravity = AccelModifier::new(writer.lit(Vec3::new(0.0, -6.0, 0.0)).expr());
    let drag = writer.lit(2.0).expr();

    // HDR color gradient: bright yellow-white core → orange → red → fade
    let mut color_gradient = bevy_hanabi::Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(4.0, 3.0, 1.5, 1.0));
    color_gradient.add_key(0.2, Vec4::new(3.0, 1.5, 0.3, 1.0));
    color_gradient.add_key(0.6, Vec4::new(2.0, 0.4, 0.05, 0.9));
    color_gradient.add_key(1.0, Vec4::new(0.5, 0.1, 0.02, 0.0));

    // Size gradient: elongated ember, shrinking
    let mut size_gradient = bevy_hanabi::Gradient::new();
    size_gradient.add_key(0.0, Vec3::new(0.10, 0.025, 1.0));
    size_gradient.add_key(0.4, Vec3::new(0.06, 0.015, 1.0));
    size_gradient.add_key(1.0, Vec3::splat(0.0));

    EffectAsset::new(2048, SpawnerSettings::rate(300.0.into()), writer.finish())
        .with_name("sword_embers")
        .init(init_pos)
        .init(init_vel)
        .init(SetAttributeModifier::new(Attribute::AGE, age))
        .init(SetAttributeModifier::new(Attribute::LIFETIME, lifetime))
        .update(gravity)
        .update(LinearDragModifier::new(drag))
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient,
            screen_space_size: false,
        })
        .render(OrientModifier::new(OrientMode::AlongVelocity))
}

fn animate(
    time: Res<Time>,
    mut orbs: Query<&mut Transform, (With<MagicOrb>, Without<SwordSpark>)>,
    mut swords: Query<&mut Transform, (With<SwordSpark>, Without<MagicOrb>)>,
) {
    let t = time.elapsed_secs();

    for mut transform in &mut orbs {
        let radius = 2.8;
        transform.translation = Vec3::new(
            -3.5 + (t * 0.9).sin() * radius,
            2.0 + (t * 1.6).cos() * 0.8,
            (t * 0.7).cos() * 1.5,
        );
    }

    for mut transform in &mut swords {
        let angle = (t * 2.8).sin() * 1.8;
        let radius = 2.0;
        transform.translation = Vec3::new(
            3.0 + angle.cos() * radius,
            1.5 + angle.sin() * 0.5,
            angle.sin() * radius * 0.3,
        );
        transform.rotation = Quat::from_rotation_z(angle * 2.0) * Quat::from_rotation_y(-angle);
    }
}

fn update_overlay(
    diagnostics: Res<TrailDiagnostics>,
    mut overlays: Query<&mut Text, With<OverlayText>>,
) {
    for mut text in &mut overlays {
        **text = format!(
            "Trail: sources={} points={}\nGPU particles running alongside",
            diagnostics.active_sources, diagnostics.active_points,
        );
    }
}
