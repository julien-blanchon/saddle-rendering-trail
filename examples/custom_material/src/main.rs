use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
};
use saddle_rendering_trail::{
    Trail, TrailEmitterMode, TrailMaterial3d, TrailMaterialPlugin, TrailPlugin, TrailScalarCurve,
    TrailStyle,
};
use saddle_rendering_trail_example_common as common;

fn main() {
    let mut app = App::new();
    app.add_plugins(common::default_plugins("Trail Custom Material"));
    app.add_plugins(TrailPlugin::default());
    app.add_plugins(
        MaterialPlugin::<ExtendedMaterial<StandardMaterial, GlowExtension>>::default(),
    );
    app.add_plugins(
        TrailMaterialPlugin::<ExtendedMaterial<StandardMaterial, GlowExtension>>::new(Update),
    );
    common::install_auto_exit(&mut app);
    app.add_systems(Startup, setup);
    app.add_systems(Update, animate);
    app.run();
}

/// Minimal material extension — uses the default PBR shaders but demonstrates
/// the `TrailMaterial3d<M>` + `TrailMaterialPlugin<M>` integration path.
/// A real extension would add custom uniforms and a fragment shader.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone, Default)]
struct GlowExtension {}

impl MaterialExtension for GlowExtension {}

#[derive(Component)]
struct Mover;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ext_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, GlowExtension>>>,
) {
    common::spawn_stage(&mut commands, &mut meshes, &mut materials);
    common::spawn_overlay(
        &mut commands,
        "Trail Custom Material",
        "ExtendedMaterial via TrailMaterial3d + TrailMaterialPlugin",
    );

    let glow_material = ext_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: Color::srgb(0.1, 0.4, 1.0),
            emissive: LinearRgba::new(0.2, 0.5, 1.0, 1.0),
            unlit: true,
            double_sided: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        },
        extension: GlowExtension {},
    });

    commands.spawn((
        Name::new("Custom Material Trail"),
        Mover,
        Trail {
            emitter_mode: TrailEmitterMode::Always,
            lifetime_secs: 1.5,
            max_points: 48,
            style: TrailStyle {
                base_width: 0.4,
                alpha_over_length: TrailScalarCurve::linear(0.0, 1.0),
                ..default()
            },
            ..default()
        },
        TrailMaterial3d(glow_material),
        Mesh3d(meshes.add(Sphere::new(0.15))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.6, 1.0),
            emissive: LinearRgba::new(0.2, 0.4, 1.0, 1.0),
            ..default()
        })),
        Transform::from_xyz(0.0, 1.5, 0.0),
    ));
}

fn animate(time: Res<Time>, mut movers: Query<&mut Transform, With<Mover>>) {
    let t = time.elapsed_secs();
    for mut transform in &mut movers {
        transform.translation.x = (t * 0.8).sin() * 3.5;
        transform.translation.z = (t * 0.6).cos() * 2.5;
    }
}
