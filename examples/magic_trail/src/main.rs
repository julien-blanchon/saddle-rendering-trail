use saddle_rendering_trail_example_common as common;

use bevy::prelude::*;
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
    common::spawn_stage(&mut commands, &mut meshes, &mut materials);
    common::spawn_overlay(
        &mut commands,
        "Magic Trail + GPU Particles",
        "Left: magic orb (trail + spark particles)\n\
         Right: sword swing (trail + ember particles)\n\
         Combines saddle-rendering-trail with bevy_hanabi.",
    );

    // --- Magic orb: trail + glowing particles ---
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
                        emissive: LinearRgba::rgb(0.4, 0.15, 0.9),
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
                        emissive: LinearRgba::rgb(1.0, 0.3, 0.05),
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

    // Overlay for diagnostics
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

fn magic_orb_effect() -> EffectAsset {
    let writer = ExprWriter::new();
    let age = writer.lit(0.0).expr();
    let lifetime = writer.lit(0.6).expr();
    let drag = writer.lit(2.0).expr();
    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.15).expr(),
        dimension: ShapeDimension::Volume,
    };
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(1.5).expr(),
    };

    EffectAsset::new(256, SpawnerSettings::rate(60.0.into()), writer.finish())
        .with_name("magic_orb_sparks")
        .init(init_pos)
        .init(init_vel)
        .init(SetAttributeModifier::new(Attribute::AGE, age))
        .init(SetAttributeModifier::new(Attribute::LIFETIME, lifetime))
        .update(LinearDragModifier::new(drag))
        .render(SetColorModifier::new(CpuValue::Uniform((
            Vec4::new(0.4, 0.15, 0.9, 1.0),
            Vec4::new(0.7, 0.4, 1.0, 1.0),
        ))))
        .render(SetSizeModifier {
            size: CpuValue::Uniform((Vec3::splat(0.02), Vec3::splat(0.06))),
        })
}

fn sword_ember_effect() -> EffectAsset {
    let writer = ExprWriter::new();
    let age = writer.lit(0.0).expr();
    let lifetime = writer.lit(0.45).expr();
    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.3).expr(),
        dimension: ShapeDimension::Surface,
    };
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(2.5).expr(),
    };
    let gravity = AccelModifier::new(writer.lit(Vec3::new(0.0, -4.0, 0.0)).expr());

    EffectAsset::new(512, SpawnerSettings::rate(120.0.into()), writer.finish())
        .with_name("sword_embers")
        .init(init_pos)
        .init(init_vel)
        .init(SetAttributeModifier::new(Attribute::AGE, age))
        .init(SetAttributeModifier::new(Attribute::LIFETIME, lifetime))
        .update(gravity)
        .render(SetColorModifier::new(CpuValue::Uniform((
            Vec4::new(1.0, 0.3, 0.05, 1.0),
            Vec4::new(1.0, 0.7, 0.2, 1.0),
        ))))
        .render(SetSizeModifier {
            size: CpuValue::Uniform((Vec3::splat(0.01), Vec3::splat(0.04))),
        })
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
