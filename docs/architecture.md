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
- camera extraction
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
2. refresh the active camera view state
3. sync source configuration and append or prune points
4. tick orphaned render instances whose source despawned

This guarantees that when `BuildMesh` runs in the same frame, it sees a complete and stable history buffer.

## Orientation Modes

### `TrailOrientation::Billboard`

The ribbon width axis is computed from:

- the local tangent of the sampled polyline
- the active camera position, transformed into trail space when needed

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
- its orientation is billboarded and the active camera changed

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

## Performance Tradeoffs

The current implementation uses a `Vec`-backed point buffer and rebuilds the entire mesh when dirty.

That is acceptable for the intended scale of:

- hero trails
- moderate counts of projectile contrails
- stylized scene dressing

The crate-local stress example exists to catch obviously bad scaling. If a future consumer needs hundreds of very long ribbons at once, the likely next steps are pooled point storage, partial uploads, or a shader-driven strip path.
