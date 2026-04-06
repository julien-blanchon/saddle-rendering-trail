# Configuration

This document covers every public tuning surface in `saddle-rendering-trail`.

Normalized length always runs from the oldest visible point (`0.0`) to the newest point near the source (`1.0`).
Normalized age runs from newly emitted (`0.0`) to expired (`1.0`).

## `Trail`

| Field | Type | Default | Expected range | Effect | Performance notes |
|------|------|---------|----------------|--------|-------------------|
| `emitter_mode` | `TrailEmitterMode` | `WhenMoving` | enum | Chooses whether the source emits always, only while moving, or never | `Always` can append more points for slow movers because the max-interval path stays active |
| `space` | `TrailSpace` | `World` | enum | Leaves residue in world space or keeps the ribbon in the source parent space | `Local` needs an extra camera-to-local conversion in billboard mode, but the cost is negligible compared with rebuilds |
| `orientation` | `TrailOrientation` | `Billboard` | enum | Faces the ribbon to the camera or locks width orientation to a source-local axis | Billboard ribbons rebuild when the camera moves; transform-locked ribbons do not |
| `view_source` | `TrailViewSource` | `ActiveCamera3d` | enum | Chooses which view transform drives billboarding, LOD, and debug normal reconstruction | Explicit entities avoid hidden dependence on whichever `Camera3d` currently wins active-camera selection |
| `lifetime_secs` | `f32` | `0.9` | `> 0.0` | How long sampled points survive before pruning | Longer lifetimes keep more visible geometry and increase rebuild cost |
| `min_sample_distance` | `f32` | `0.18` | `>= 0.0` | Minimum travel distance before a new point may be inserted | Smaller values produce smoother curves but more points |
| `max_sample_interval_secs` | `f32` | `0.05` | `>= 0.0` | Forces new points for slow motion when enough time passes | Lower values increase sampling frequency on slow movers |
| `max_points` | `usize` | `48` | `>= 2` recommended | Hard cap on retained points | Primary safety valve for long-lived or fast-moving trails |
| `teleport_distance` | `f32` | `4.0` | `>= 0.0` | Clears history when a new sample jumps too far from the previous point | Cheap and worth keeping on for almost every gameplay-style trail |
| `keep_after_source_despawn` | `bool` | `true` | boolean | Lets the trail decay after the source entity disappears | No extra steady-state cost; useful for projectiles and short-lived swipes |
| `clear_on_deactivate` | `bool` | `true` | boolean | Clears or despawns render state when the plugin deactivate schedule runs | Clearing aggressively can reduce stale entities in state-driven apps |

## `TrailEmitterMode`

| Variant | Choose it when | Notes |
|---------|----------------|-------|
| `Always` | You want a continuous ribbon even through slow movement | Uses both distance and max-interval sampling |
| `WhenMoving` | You want the default "emit only while motion is happening" behavior | Still uses the max-interval path when the source is moving slightly |
| `Disabled` | You want to pause a trail without removing the component | Existing history continues to age out |

## `TrailSpace`

| Variant | Choose it when | Notes |
|---------|----------------|-------|
| `World` | Residue should stay where it was emitted | Best for projectile contrails, dust wakes, speed streaks |
| `Local` | The ribbon should follow the source's parent frame | Best for anchored weapon trails or rig-relative effects |

## `TrailOrientation`

| Variant | Choose it when | Notes |
|---------|----------------|-------|
| `Billboard` | Readability from the camera matters most | Default for contrails and speed lines |
| `TransformLocked { axis }` | The source should control ribbon roll and facing | Use an axis like `Vec3::Y` or `Vec3::Z` depending on how the source is authored |

## `TrailViewSource`

| Variant | Choose it when | Notes |
|---------|----------------|-------|
| `ActiveCamera3d` | You want the default shared-camera behavior | Resolves to the lowest-order active `Camera3d` each frame |
| `Entity(Entity)` | You need billboarding and LOD to follow a specific camera or authored view anchor | Uses that entity's world transform even if it is not the active `Camera3d` |

## `TrailStyle`

`TrailStyle::default()` is intentionally neutral: solid white, full alpha, and constant width.
Showcase gradients, tail fades, and other authored looks now live in examples or user-authored presets.

| Field | Type | Default | Expected range | Effect | Performance notes |
|------|------|---------|----------------|--------|-------------------|
| `base_width` | `f32` | `0.35` | `>= 0.0` | Base ribbon width before width-curve modulation | Wider trails increase the size of the generated bounds but not vertex count |
| `fade_mode` | `TrailFadeMode` | `Alpha` | enum | How the trail fades from head to tail — opacity, width, or both | Width/Both fade modes apply alpha curves to the width computation, so the mesh itself narrows |
| `width_over_length` | `TrailScalarCurve` | constant `1.0` | keys in `0..1` | Multiplies width from tail to head | Cheap curve evaluation per point |
| `color_over_length` | `TrailGradient` | solid white | keys in `0..1` | Interpolates vertex color from tail to head | Cheap per-point linear interpolation |
| `alpha_over_length` | `TrailScalarCurve` | constant `1.0` | keys in `0..1` | Fades by position along the ribbon | Author a non-constant curve when you want a soft tail |
| `alpha_over_age` | `TrailScalarCurve` | constant `1.0` | keys in `0..1` | Fades individual points as they age toward expiration | Non-constant curves rebuild while live points age, even if the source is otherwise stationary |
| `uv_mode` | `TrailUvMode` | `Stretch` | enum | Controls how `u` advances along the ribbon | Repeat mode uses traveled distance accumulation but is still cheap |
| `uv_scroll_speed` | `f32` | `0.0` | any | Continuous UV scroll along the trail in units per second | When non-zero the trail marks itself dirty every frame, increasing rebuild frequency |
| `material` | `TrailMaterial` | see below | struct | StandardMaterial-backed shading configuration | Material changes are copied onto the owned render material handle |

## `TrailUvMode`

| Variant | Choose it when | Notes |
|---------|----------------|-------|
| `Stretch` | You want a single gradient or texture span across the whole ribbon | Most stable for soft additive-style looks |
| `RepeatByDistance { distance }` | You want repeated streaks, stripes, or dashed textures | Keep `distance` comfortably above zero; very small values will tile aggressively |

## `TrailMaterial`

| Field | Type | Default | Expected range | Effect | Performance notes |
|------|------|---------|----------------|--------|-------------------|
| `base_color` | `Color` | `Color::WHITE` | any color | Base tint multiplied with vertex color | Pure data copy into `StandardMaterial` |
| `texture` | `Option<Handle<Image>>` | `None` | texture handle | Optional albedo texture for stripes, soft alpha, or stylized patterns | Texture sampling cost depends on the material path, not the trail logic |
| `emissive` | `LinearRgba` | black | any color | Adds glow when using lit materials | Useful for VFX readability without changing geometry |
| `unlit` | `bool` | `true` | boolean | Chooses between unlit and lit shading | Unlit is usually the safest trail default |
| `double_sided` | `bool` | `true` | boolean | Disables back-face culling for ribbons viewed from either side | Double-sided is usually correct for thin ribbons |
| `alpha_mode` | `AlphaMode` | `Blend` | enum | StandardMaterial transparency mode | `Blend` is the normal trail choice |
| `disable_frustum_culling` | `bool` | `false` | boolean | Adds `NoFrustumCulling` to the render entity | Use only when you suspect bounds-related popping and can accept extra draw cost |

## `TrailScalarCurve`

`TrailScalarCurve` is a sorted list of `TrailScalarKey { position, value }`.

- Positions are expected in `0.0..=1.0`.
- The runtime clamps evaluation outside the authored range.
- Empty curves are repaired to a constant `1.0`.

Practical guidance:

- Use a rising tail-to-head width curve for contrails.
- Use a bell-ish curve for melee swipes by adding a few interior keys.
- Use `alpha_over_age` instead of shrinking `lifetime_secs` when you want a soft dissolve without shortening the trail too aggressively.

## `TrailGradient`

`TrailGradient` is a sorted list of `TrailColorKey { position, color }`.

- Positions are expected in `0.0..=1.0`.
- Colors interpolate in linear space.
- Empty gradients are repaired to solid white.

Practical guidance:

- Tail-dark, head-bright ramps read well for speed lines.
- Tail-transparent, head-bright ramps are often better handled by keeping the color bright and shaping opacity with `alpha_over_length`.

## `TrailFadeMode`

| Variant | Choose it when | Notes |
|---------|----------------|-------|
| `Alpha` | Default — the trail fades via opacity from head to tail | Standard approach, works best with `AlphaMode::Blend` |
| `Width` | You want the trail to narrow to nothing at the tail | Alpha curves are redirected to modulate the width instead of vertex alpha; useful for melee swipes and weapon trails |
| `Both` | You want simultaneous width narrowing and opacity fade | Combines both approaches — the trail shrinks and fades at the same time |

## `TrailMeshMode`

| Variant | Choose it when | Notes |
|---------|----------------|-------|
| `Ribbon` | Default — a flat two-vertex strip that faces the camera or follows a locked axis | Cheapest option, best for most trail effects |
| `Tube { sides }` | You need a cylindrical cross-section (rope, wire, energy beam) | Generates `sides` vertices per sample point in a ring; minimum 3 sides. Roughly `sides × points` vertices per frame rebuild |

## `TrailCustomMaterial`

Attach `TrailCustomMaterial(handle)` to a trail source entity to override the auto-generated `StandardMaterial`. When present, the crate skips creating and syncing its own material and instead assigns your handle to the render entity.

- Adding the component at runtime swaps the material immediately
- Removing the component reverts to the auto-generated material from `TrailMaterial`
- Changing the handle inside the component hot-swaps the material on the render entity

## `TrailLod`

Attach `TrailLod` to a trail source entity for distance-based level of detail.

| Field | Type | Default | Expected range | Effect |
|------|------|---------|----------------|--------|
| `start_distance` | `f32` | `20.0` | `>= 0.0` | Distance at which point-count reduction begins |
| `end_distance` | `f32` | `60.0` | `> start_distance` recommended | Distance at which the trail reaches minimum detail |
| `min_points_fraction` | `f32` | `0.25` | `0.0..=1.0` recommended | Fraction of `Trail::max_points` retained at or beyond `end_distance` |

The effective max-points is linearly interpolated between `Trail::max_points` (at or below `start_distance`) and `Trail::max_points × min_points_fraction` (at or beyond `end_distance`). The distance is measured against the trail's resolved `TrailViewSource`, not always the shared active camera.

## `TrailSamplePoint`

`TrailSamplePoint` is the public type for individual trail sample points. It exposes position, age, and normalized length for user-defined modifier systems that run between `TrailSystems::Sample` and `TrailSystems::BuildMesh`.

## `TrailDebugSettings`

Debug drawing is only active when the app includes Bevy gizmos and `enabled = true`.

| Field | Type | Default | Effect |
|------|------|---------|--------|
| `enabled` | `bool` | `false` | Master toggle for all debug drawing |
| `draw_points` | `bool` | `false` | Draws sampled points |
| `draw_segments` | `bool` | `true` | Draws the sampled centerline |
| `draw_normals` | `bool` | `false` | Draws the width-direction helper at each point |
| `draw_bounds` | `bool` | `false` | Draws the generated AABB extents |
| `point_radius` | `f32` | `0.05` | Radius used for point markers |
| `normal_length` | `f32` | `0.35` | Length used when drawing normals |

## `TrailDiagnostics`

`TrailDiagnostics` is read-only runtime output.

| Field | Meaning |
|------|---------|
| `runtime_active` | Whether the runtime is currently considered active |
| `active_sources` | Number of source entities that still carry `Trail` |
| `active_render_entities` | Number of spawned render entities |
| `orphaned_render_entities` | Render entities whose source despawned but whose history is still decaying |
| `active_points` | Total sampled points across all render instances |
| `visible_trails` | Render instances whose mesh currently contains drawable geometry |
| `dirty_trails` | Render instances that still need a rebuild this frame |
| `total_mesh_rebuilds` | Accumulated rebuild count since startup |
| `total_resets` | Accumulated teleport/discontinuity reset count since startup |

These counters are useful both for HUD overlays in a lab app and for rough stress validation.
