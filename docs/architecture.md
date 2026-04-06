# Architecture

## Why A CPU-Built Ribbon In V1

`saddle-rendering-trail` deliberately starts with a CPU-built mesh path.

Reasons:

1. It keeps the public contract small: consumers only attach `Trail` and do not need to opt into a custom render pipeline.
2. It maps cleanly onto Bevy's documented runtime `Mesh` update path.
3. It keeps debugging straightforward because sampled points, built vertices, and culling bounds all live in ordinary ECS data.
4. It matches the most important production controls from prior-art trail systems: spacing thresholds, lifetime pruning, width curves, color ramps, UV modes, and discontinuity resets.

This tradeoff favors clarity and portability over maximum trail count. The crate can grow a custom material or shader path later without changing the sampling model.

## Prior Art That Shaped V1

- Bevy `Mesh` docs and runtime mesh examples reinforced the decision to rebuild a triangle-list ribbon with explicit attribute ownership.
- Godot's `RibbonTrailMesh` shaped the section-based ribbon mental model and the emphasis on width curves over opaque spline magic.
- Unity's Trail Renderer manual shaped the v1 control surface: lifetime, min distance, width, color gradient, and texture mode.
- PlayCanvas's mesh-API trail walkthrough reinforced the value of keeping the whole feature legible as "sample points, generate strip, upload mesh".

## Layer Split

The crate is intentionally split in two.

### Pure domain logic

`sampling.rs` owns:

- point insertion
- point aging
- lifetime pruning
- max-point trimming
- min-distance checks
- max-interval checks
- teleport reset detection
- normalized length evaluation

This logic is plain Rust and easy to unit-test.

### Bevy integration

`systems.rs`, `mesh_builder.rs`, and `debug.rs` own:

- ECS components and resources
- schedule wiring
- render-entity spawn and cleanup
- active-camera snapshotting and per-trail view-source resolution
- mesh rebuilds
- bounds updates
- diagnostics publication
- optional gizmo output

That separation keeps most correctness work testable without a renderer while still giving the runtime a clean ECS surface.

## Data Flow

```text
source entity with Trail
  -> ensure a render entity exists
  -> sample source transform into TrailBuffer
  -> prune old points and trim to point budget
  -> reset history on large discontinuities
  -> evaluate width/color/alpha/uv per point
  -> build a triangle-list ribbon mesh
  -> update mesh asset + AABB + Visibility
  -> publish TrailDiagnostics
```

The source entity and render entity are separate on purpose. The source stays focused on gameplay ownership while the render entity owns the mesh, material handle, and trail history bookkeeping.

## System Ordering

`TrailSystems` exposes the runtime phases explicitly:

1. `Sample`
2. `BuildMesh`
3. `Cleanup`
4. `Diagnostics`
5. `Debug`

Within the default plugin these sets are chained.

The `Sample` phase itself runs in this order:

1. spawn missing render entities
2. refresh the active `Camera3d` snapshot
3. resolve each trail's view source, sync source configuration, and append or prune points
4. tick orphaned render instances whose source despawned

This guarantees that when `BuildMesh` runs in the same frame, it sees a complete and stable history buffer.

## Orientation Modes

## View Source Resolution

Every trail resolves a `TrailViewSource` before LOD, billboard rebuilding, and debug-normal reconstruction run.

- `TrailViewSource::ActiveCamera3d` uses the current lowest-order active `Camera3d`
- `TrailViewSource::Entity(entity)` uses that entity's world transform directly

The resolved view is cached on the render instance so the runtime can detect per-trail view changes even when different trails follow different cameras or view anchors.

### `TrailOrientation::Billboard`

The ribbon width axis is computed from:

- the local tangent of the sampled polyline
- the resolved view position, transformed into trail space when needed

The side vector is `tangent x view_direction`. If that degenerates, the runtime falls back to the source's local up axis projected onto the plane orthogonal to the tangent.

This mode is the default because it reads cleanly for contrails, wakes, and speed-line style effects.

### `TrailOrientation::TransformLocked`

The consumer provides a source-local axis such as `Vec3::Y`.

The runtime rotates that axis by each sampled point's source rotation and projects it away from the tangent. This keeps the ribbon aligned to the source rather than the camera, which is a better fit for melee swipes and tether-like effects.

## Length, Curves, And UVs

Each sampled point receives a normalized length value from `0.0` to `1.0`.

- `0.0` is the oldest remaining point in the trail
- `1.0` is the newest point, closest to the emitting source

That normalized length drives:

- `width_over_length`
- `color_over_length`
- `alpha_over_length`

Point age is tracked separately and normalized against `Trail::lifetime_secs`. That age drives `alpha_over_age`.

UV generation has two policies:

- `Stretch`: the ribbon spans `u = 0..1` over the current visible length
- `RepeatByDistance`: `u` advances by traveled world or local distance divided by the repeat distance

## Space Handling

### `TrailSpace::World`

- sampled positions come from the source `GlobalTransform`
- the render entity stays at identity
- the trail remains where it was emitted even if the parent hierarchy later moves

### `TrailSpace::Local`

- sampled positions come from the source local `Transform`
- the render entity is anchored to the parent's current global transform
- the trail follows the parent hierarchy

This is the main choice for "leave residue in the world" versus "stick the ribbon to a moving rig or attachment frame".

## Pruning And Discontinuity Handling

Every update tick:

1. point ages increase by `delta_secs`
2. points older than `lifetime_secs` are removed
3. if the point count exceeds `max_points`, the oldest points are dropped

Discontinuity handling is separate. If a new sample is farther than `teleport_distance` from the previous point, the history is cleared before the new point is appended. This avoids giant stretched triangles after warps, respawns, or hard rewinds.

## Idle Behavior

The runtime does not rebuild meshes every frame by default.

A trail becomes dirty when:

- its history changed because of a new sample or pruning
- its configuration changed
- it uses a non-constant `alpha_over_age` curve and live points are still aging
- its source despawned and the decay tail is still shrinking
- its orientation is billboarded and the resolved view changed

If none of those conditions apply, the mesh is left alone.

## Bounds And Culling

Each rebuild computes an axis-aligned bounding box from the generated ribbon vertices and writes that AABB back to the render entity.

That solves the common runtime-mesh problem where geometry updates but stale bounds cause incorrect culling.

`TrailMaterial::disable_frustum_culling` is available for effects where the consumer prefers overdraw to culling risk, but the default path keeps culling enabled.

## Cleanup Rules

### Deactivate schedule

When the plugin's deactivate schedule runs:

- `clear_on_deactivate = true` despawns the render entity and removes the source link
- otherwise the history is cleared in place and the render entity remains alive

### Source despawn

When a source entity disappears:

- `keep_after_source_despawn = true` lets the existing trail decay naturally until its points expire
- otherwise cleanup removes the render entity as soon as the source link is detected as stale

## Fade Modes

`TrailFadeMode` controls how the trail visually tapers from head to tail:

### `Alpha` (default)

The existing alpha curves (`alpha_over_length`, `alpha_over_age`) drive vertex alpha as before. Width is unaffected by alpha.

### `Width`

The alpha curves are redirected to modulate the width instead of the vertex alpha. The mesh physically narrows at the tail rather than becoming transparent. Vertex alpha is set to `1.0` so the trail remains fully opaque. This is useful for solid weapon trails where transparency doesn't look right.

### `Both`

Both channels are active: the alpha curves drive both width modulation and vertex alpha simultaneously. The trail narrows and fades at the same time.

The fade mode is evaluated in `mesh_builder.rs` via two helper functions:
- `compute_width_with_fade()` — multiplies the base width × width curve by the alpha curve when the mode is `Width` or `Both`
- `compute_color_with_fade()` — skips alpha application when the mode is `Width` only

## Tube Mesh Mode

`TrailMeshMode::Tube { sides }` generates a cylindrical cross-section instead of a flat ribbon.

For each sample point, the builder generates `sides` vertices arranged in a circle perpendicular to the tangent direction. The circle plane is constructed from the same side vector and up vector used by the ribbon builder, then swept `sides` times around the tangent.

Index generation connects adjacent rings with triangle pairs, forming a closed tube surface. At the tail end, a fan of triangles closes the cap.

Vertex count scales as `sides × points` (compared with `2 × points` for ribbons), so tube mode is more expensive. Typical values of 4–8 sides work well for rope and energy beam effects.

## UV Scroll

When `TrailStyle::uv_scroll_speed` is non-zero, the runtime accumulates a UV offset each frame:

```
uv_scroll_offset += uv_scroll_speed × delta_secs
```

The offset is added to the base `u` coordinate during mesh generation, creating continuous texture animation along the trail. This is useful for flowing energy effects, magical streams, and pulsing patterns.

The scroll offset is tracked per render instance and continues to accumulate even for orphaned (decaying) trails.

**Performance note**: A non-zero scroll speed marks the trail dirty every frame, forcing a mesh rebuild regardless of whether the source is moving or the camera has changed.

## Custom Material Override

By default, the crate creates and manages a `StandardMaterial` for each trail's render entity, syncing it from the `TrailMaterial` configuration.

When a `TrailCustomMaterial(handle)` component is present on the source entity:

1. The crate assigns the user's material handle to the render entity instead of the auto-generated one
2. It skips all per-frame material synchronization from `TrailMaterial`
3. If the user removes the component, the crate reverts to managing its own material
4. If the user changes the handle inside the component, the crate hot-swaps the render entity's material

This escape hatch allows advanced users to use custom shaders, animated materials, or material instances shared across multiple trail sources.

## Level of Detail (LOD)

The optional `TrailLod` component enables distance-based point count reduction.

Each frame, the system computes the distance between the trail source and the resolved `TrailViewSource`. This distance is used to linearly interpolate the effective `max_points`:

```
effective_max_points = lerp(
    trail.max_points,
    trail.max_points * lod.min_points_fraction,
    (distance - lod.start_distance) / (lod.end_distance - lod.start_distance),
)
```

Clamped so that it never exceeds `trail.max_points` or drops below the configured fraction floor.

When the effective max is lower than the current point count, excess points are trimmed from the oldest end. This reduces vertex count for far-away trails where detail is not visible.

## Public Sample Points

`TrailSamplePoint` is exposed as a public type so that users can write custom modifier systems. A modifier system ordered between `TrailSystems::Sample` and `TrailSystems::BuildMesh` can read or mutate the sample points in a trail's buffer before the mesh is generated.

Example use cases:
- Procedural noise displacement along the trail
- Physics-based droop or wind deflection
- Snapping points to a surface

## Performance Tradeoffs

The current implementation uses a `Vec`-backed point buffer and rebuilds the entire mesh when dirty.

That is acceptable for the intended scale of:

- hero trails
- moderate counts of projectile contrails
- stylized scene dressing

The crate-local stress example exists to catch obviously bad scaling. If a future consumer needs hundreds of very long ribbons at once, the likely next steps are pooled point storage, partial uploads, or a shader-driven strip path.
