# Saddle Rendering Trail

Reusable runtime ribbon and motion trail renderer for Bevy. Attach `Trail` to any moving entity and the crate will sample its motion, build a ribbon mesh, and keep that mesh updated as the source moves, pauses, deactivates, or despawns.

`saddle-rendering-trail` is designed as a small rendering primitive rather than a combat-specific effect. It covers projectile contrails, melee swipes, tether residue, hover wakes, and stylized speed lines without importing any project-specific types.

## Quick Start

```rust
use bevy::prelude::*;
use saddle_rendering_trail::{
    Trail, TrailEmitterMode, TrailGradient, TrailPlugin, TrailScalarCurve, TrailStyle,
    TrailUvMode,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TrailPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Trail Source"),
        Trail::default()
            .with_emitter_mode(TrailEmitterMode::Always)
            .with_style(TrailStyle {
                base_width: 0.22,
                width_over_length: TrailScalarCurve::linear(0.2, 1.0),
                color_over_length: TrailGradient::constant(Color::srgb(0.9, 0.75, 1.0)),
                uv_mode: TrailUvMode::RepeatByDistance { distance: 0.35 },
                ..default()
            }),
        Mesh3d(meshes.add(Capsule3d::new(0.12, 0.4))),
        MeshMaterial3d(materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(0.4, 0.2, 0.6),
            ..default()
        })),
        Transform::from_xyz(0.0, 1.2, 0.0),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 3.5, 8.0).looking_at(Vec3::new(0.0, 1.2, 0.0), Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 18_000.0,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
```

For state-scoped integration, construct the plugin with explicit schedules:

```rust
app.add_plugins(TrailPlugin::new(
    OnEnter(MyState::Active),
    OnExit(MyState::Active),
    Update,
));
```

`TrailPlugin::default()` is the always-on form and internally maps to `PostStartup`, a no-op deactivate schedule, and `Update`.

## Public API

| Type | Purpose |
|------|---------|
| `TrailPlugin` | Registers the runtime with injectable activate, deactivate, and update schedules |
| `TrailSystems` | Public ordering hooks for sampling, mesh rebuilds, cleanup, diagnostics, and optional debug drawing |
| `Trail` | Per-source sampling, lifetime, reset, cleanup, and orientation configuration |
| `TrailStyle` | Width, color, alpha, UV, and material configuration for the generated ribbon |
| `TrailMaterial` | StandardMaterial-backed appearance settings for the spawned render entity |
| `TrailEmitterMode` | `Always`, `WhenMoving`, or `Disabled` sampling behavior |
| `TrailSpace` | `World` or `Local` point storage and mesh-space behavior |
| `TrailOrientation` | `Billboard` or transform-locked axis mode |
| `TrailUvMode` | Stretch once over the full ribbon or repeat by traveled distance |
| `TrailScalarCurve` / `TrailScalarKey` | Width and alpha curves over normalized length or normalized age |
| `TrailGradient` / `TrailColorKey` | Color ramp over normalized ribbon length |
| `TrailDebugSettings` | Optional gizmo drawing for points, segments, normals, and bounds |
| `TrailDiagnostics` | Runtime counters for active sources, active points, resets, and mesh rebuilds |

## Supported

- CPU-built ribbon meshes with rebuilds only when sampling, styling, camera state, or age-driven alpha requires new geometry colors
- World-space and local-space trails
- Camera-facing billboard ribbons
- Transform-locked ribbons using a source-local axis
- Lifetime pruning, point-budget trimming, and teleport reset handling
- Width curves over normalized trail length
- Color ramps over normalized trail length
- Alpha over normalized trail length and normalized point age
- Stretch and repeat-by-distance UV modes
- Source-despawn decay tails and deactivate-time clear behavior
- Diagnostics publication and optional gizmo debug drawing
- Crate-local examples, stress checks, E2E scenarios, and BRP inspection

## Intentionally Deferred In V1

- Dual-edge sword-strip authoring
- Cross-ribbon volumetric shapes
- Spline smoothing or interpolation passes
- Custom material or shader ownership by the consumer
- Pooling or ring-buffer reuse beyond the current `Vec`-backed history

The v1 runtime deliberately keeps the rendering path small and debuggable: the crate owns a `StandardMaterial` plus a generated mesh with vertex colors and UVs.

## Examples

| Example | What it demonstrates | Run |
|---------|----------------------|-----|
| `basic` | Minimal always-on billboard trail with width and alpha shaping | `cargo run -p saddle-rendering-trail-example-basic` |
| `projectile_contrail` | Narrow fast mover with repeat-by-distance UVs | `cargo run -p saddle-rendering-trail-example-projectile-contrail` |
| `melee_swipe` | Short-lived wide transform-locked ribbon | `cargo run -p saddle-rendering-trail-example-melee-swipe` |
| `orientation_modes` | Billboard and transform-locked trails side by side | `cargo run -p saddle-rendering-trail-example-orientation-modes` |
| `space_modes` | World-space residue versus parent-following local-space trails | `cargo run -p saddle-rendering-trail-example-space-modes` |
| `stress` | Many simultaneous trails for rough scaling checks | `cargo run -p saddle-rendering-trail-example-stress` |

## Workspace Lab

The workspace also contains a crate-local verification app at
`shared/rendering/saddle-rendering-trail/examples/lab`:

```bash
cargo run -p saddle-rendering-trail-lab
```

## Lab Verification

```bash
cargo run -p saddle-rendering-trail-lab --features e2e -- trail_smoke
cargo run -p saddle-rendering-trail-lab --features e2e -- trail_billboard
cargo run -p saddle-rendering-trail-lab --features e2e -- trail_locked
cargo run -p saddle-rendering-trail-lab --features e2e -- trail_reset
```

For BRP inspection:

```bash
TRAIL_LAB_BRP_PORT=15752 cargo run -p saddle-rendering-trail-lab
uv run --project .codex/skills/bevy-brp/script brp world query -p 15752 bevy_ecs::name::Name
uv run --project .codex/skills/bevy-brp/script brp resource get -p 15752 saddle_rendering_trail::resources::TrailDiagnostics
uv run --project .codex/skills/bevy-brp/script brp extras screenshot -p 15752 /tmp/trail_lab.png
uv run --project .codex/skills/bevy-brp/script brp extras shutdown -p 15752
```

## More Detail

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
