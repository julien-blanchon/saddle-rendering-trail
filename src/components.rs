use bevy::{
    camera::visibility::NoFrustumCulling, color::LinearRgba, pbr::StandardMaterial, prelude::*,
    reflect::Reflect, render::render_resource::Face,
};

use crate::sampling::SamplePoint;

#[derive(Component, Clone, Debug, Reflect, PartialEq)]
#[reflect(Component, Default)]
#[require(Transform)]
pub struct Trail {
    pub emitter_mode: TrailEmitterMode,
    pub space: TrailSpace,
    pub orientation: TrailOrientation,
    pub view_source: TrailViewSource,
    pub mesh_mode: TrailMeshMode,
    pub lifetime_secs: f32,
    pub min_sample_distance: f32,
    pub max_sample_interval_secs: f32,
    pub max_points: usize,
    pub teleport_distance: f32,
    pub keep_after_source_despawn: bool,
    pub clear_on_deactivate: bool,
    pub style: TrailStyle,
}

impl Default for Trail {
    fn default() -> Self {
        Self {
            emitter_mode: TrailEmitterMode::WhenMoving,
            space: TrailSpace::World,
            orientation: TrailOrientation::Billboard,
            view_source: TrailViewSource::ActiveCamera3d,
            mesh_mode: TrailMeshMode::Ribbon,
            lifetime_secs: 0.9,
            min_sample_distance: 0.18,
            max_sample_interval_secs: 0.05,
            max_points: 48,
            teleport_distance: 4.0,
            keep_after_source_despawn: true,
            clear_on_deactivate: true,
            style: TrailStyle::default(),
        }
    }
}

impl Trail {
    #[must_use]
    pub fn with_style(mut self, style: TrailStyle) -> Self {
        self.style = style;
        self
    }

    #[must_use]
    pub fn with_orientation(mut self, orientation: TrailOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    #[must_use]
    pub fn with_view_source(mut self, view_source: TrailViewSource) -> Self {
        self.view_source = view_source;
        self
    }

    #[must_use]
    pub fn with_view_entity(mut self, entity: Entity) -> Self {
        self.view_source = TrailViewSource::Entity(entity);
        self
    }

    #[must_use]
    pub fn with_space(mut self, space: TrailSpace) -> Self {
        self.space = space;
        self
    }

    #[must_use]
    pub fn with_emitter_mode(mut self, emitter_mode: TrailEmitterMode) -> Self {
        self.emitter_mode = emitter_mode;
        self
    }

    #[must_use]
    pub fn with_lifetime_secs(mut self, lifetime_secs: f32) -> Self {
        self.lifetime_secs = lifetime_secs.max(0.01);
        self
    }

    #[must_use]
    pub fn with_mesh_mode(mut self, mesh_mode: TrailMeshMode) -> Self {
        self.mesh_mode = mesh_mode;
        self
    }
}

#[derive(Component, Resource, Clone, Debug, Reflect, PartialEq)]
#[reflect(Component, Resource, Default)]
pub struct TrailDebugSettings {
    pub enabled: bool,
    pub draw_points: bool,
    pub draw_segments: bool,
    pub draw_normals: bool,
    pub draw_bounds: bool,
    pub point_radius: f32,
    pub normal_length: f32,
}

impl Default for TrailDebugSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            draw_points: false,
            draw_segments: true,
            draw_normals: false,
            draw_bounds: false,
            point_radius: 0.05,
            normal_length: 0.35,
        }
    }
}

#[derive(Clone, Debug, Reflect, PartialEq)]
#[reflect(Default)]
pub struct TrailStyle {
    pub base_width: f32,
    pub width_over_length: TrailScalarCurve,
    pub width_over_age: TrailScalarCurve,
    pub color_over_length: TrailGradient,
    pub color_over_age: TrailGradient,
    pub alpha_over_length: TrailScalarCurve,
    pub alpha_over_age: TrailScalarCurve,
    pub fade_mode: TrailFadeMode,
    pub uv_mode: TrailUvMode,
    pub uv_scroll_speed: f32,
    pub material: TrailMaterial,
}

impl Default for TrailStyle {
    fn default() -> Self {
        Self {
            base_width: 0.35,
            width_over_length: TrailScalarCurve::constant(1.0),
            width_over_age: TrailScalarCurve::constant(1.0),
            color_over_length: TrailGradient::constant(Color::WHITE),
            color_over_age: TrailGradient::constant(Color::WHITE),
            alpha_over_length: TrailScalarCurve::constant(1.0),
            alpha_over_age: TrailScalarCurve::constant(1.0),
            fade_mode: TrailFadeMode::Alpha,
            uv_mode: TrailUvMode::Stretch,
            uv_scroll_speed: 0.0,
            material: TrailMaterial::default(),
        }
    }
}

impl TrailStyle {
    #[must_use]
    pub fn with_texture(mut self, texture: Handle<Image>) -> Self {
        self.material.texture = Some(texture);
        self
    }

    #[must_use]
    pub fn evaluate_color(&self, length_t: f32, age_t: f32) -> LinearRgba {
        let length_color = self.color_over_length.evaluate(length_t);
        let age_color = self.color_over_age.evaluate(age_t);
        let mut color = LinearRgba {
            red: length_color.red * age_color.red,
            green: length_color.green * age_color.green,
            blue: length_color.blue * age_color.blue,
            alpha: length_color.alpha * age_color.alpha,
        };
        let alpha = self.alpha_over_length.evaluate(length_t) * self.alpha_over_age.evaluate(age_t);
        color.alpha *= alpha.clamp(0.0, 1.0);
        color
    }

    #[must_use]
    pub fn evaluate_width(&self, length_t: f32, age_t: f32) -> f32 {
        (self.base_width
            * self.width_over_length.evaluate(length_t)
            * self.width_over_age.evaluate(age_t))
        .max(0.0)
    }

    #[must_use]
    pub(crate) fn animates_over_age(&self) -> bool {
        !self.alpha_over_age.is_constant()
            || !self.width_over_age.is_constant()
            || !self.color_over_age.is_constant_gradient()
    }
}

#[derive(Clone, Debug, Reflect, PartialEq)]
#[reflect(Default)]
pub struct TrailMaterial {
    pub base_color: Color,
    pub texture: Option<Handle<Image>>,
    pub emissive: LinearRgba,
    pub unlit: bool,
    pub double_sided: bool,
    pub alpha_mode: AlphaMode,
    pub disable_frustum_culling: bool,
}

impl Default for TrailMaterial {
    fn default() -> Self {
        Self {
            base_color: Color::WHITE,
            texture: None,
            emissive: LinearRgba::BLACK,
            unlit: true,
            double_sided: true,
            alpha_mode: AlphaMode::Blend,
            disable_frustum_culling: false,
        }
    }
}

impl TrailMaterial {
    #[must_use]
    pub fn to_standard_material(&self) -> StandardMaterial {
        StandardMaterial {
            base_color: self.base_color,
            base_color_texture: self.texture.clone(),
            emissive: self.emissive,
            unlit: self.unlit,
            alpha_mode: self.alpha_mode,
            cull_mode: if self.double_sided {
                None
            } else {
                Some(Face::Back)
            },
            ..default()
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq, Eq)]
pub enum TrailEmitterMode {
    Always,
    #[default]
    WhenMoving,
    Disabled,
}

#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq, Eq)]
pub enum TrailSpace {
    #[default]
    World,
    Local,
}

#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq, Eq)]
pub enum TrailViewSource {
    #[default]
    ActiveCamera3d,
    Entity(Entity),
}

/// Controls how the trail visually fades out over its length and age.
#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq, Eq)]
pub enum TrailFadeMode {
    /// Fade by reducing opacity (default behaviour).
    #[default]
    Alpha,
    /// Fade by shrinking width to zero while keeping full opacity.
    Width,
    /// Apply both alpha and width fading simultaneously.
    Both,
}

/// Controls the cross-section geometry of the trail mesh.
#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq)]
pub enum TrailMeshMode {
    /// Flat two-vertex-per-point ribbon strip (default).
    #[default]
    Ribbon,
    /// Cylindrical tube with `sides` vertices per cross-section ring.
    Tube { sides: u32 },
}

/// Attach to a trail source entity to override the auto-generated material
/// with a user-provided `StandardMaterial` handle.
#[derive(Component, Clone, Debug)]
pub struct TrailCustomMaterial(pub Handle<StandardMaterial>);

/// Attach to a trail source entity to use a custom material type.
///
/// The material type must implement [`Material`]. You are responsible for
/// ensuring `MaterialPlugin::<M>` is added to your app.
///
/// When this component is present, the trail system will insert
/// `MeshMaterial3d::<M>` on the render entity instead of
/// `MeshMaterial3d::<StandardMaterial>`.
///
/// Vertex colors (`ATTRIBUTE_COLOR`) and UVs (`ATTRIBUTE_UV_0`) are still
/// generated by the trail system. Your material should sample them.
///
/// Add [`TrailMaterialPlugin::<M>`](crate::TrailMaterialPlugin) to sync
/// this component to the render entity.
#[derive(Component, Clone, Debug)]
pub struct TrailMaterial3d<M: Material>(pub Handle<M>);

/// Optional LOD configuration. Attach to a trail source entity to reduce
/// detail for trails far from the camera.
#[derive(Component, Clone, Debug, Reflect, PartialEq)]
#[reflect(Component, Default)]
pub struct TrailLod {
    /// Camera distance at which LOD reduction begins.
    pub start_distance: f32,
    /// Camera distance at which the trail reaches minimum detail.
    pub end_distance: f32,
    /// Fraction of `max_points` used at (or beyond) `end_distance` (e.g. 0.25).
    pub min_points_fraction: f32,
}

impl Default for TrailLod {
    fn default() -> Self {
        Self {
            start_distance: 20.0,
            end_distance: 60.0,
            min_points_fraction: 0.25,
        }
    }
}

impl TrailLod {
    /// Returns the effective `max_points` for the given camera distance.
    #[must_use]
    pub fn effective_max_points(&self, distance: f32, base_max_points: usize) -> usize {
        if distance <= self.start_distance {
            base_max_points
        } else if distance >= self.end_distance {
            ((base_max_points as f32) * self.min_points_fraction).max(2.0) as usize
        } else {
            let t = (distance - self.start_distance)
                / (self.end_distance - self.start_distance).max(f32::EPSILON);
            let fraction = 1.0 - t * (1.0 - self.min_points_fraction);
            ((base_max_points as f32) * fraction).max(2.0) as usize
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq)]
pub enum TrailOrientation {
    #[default]
    Billboard,
    TransformLocked {
        axis: Vec3,
    },
}

#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq)]
pub enum TrailUvMode {
    #[default]
    Stretch,
    RepeatByDistance {
        distance: f32,
    },
}

#[derive(Clone, Debug, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct TrailScalarCurve {
    pub keys: Vec<TrailScalarKey>,
}

impl TrailScalarCurve {
    #[must_use]
    pub fn new(keys: impl IntoIterator<Item = TrailScalarKey>) -> Self {
        let mut keys = keys.into_iter().collect::<Vec<_>>();
        keys.sort_by(|a, b| a.position.total_cmp(&b.position));
        if keys.is_empty() {
            keys.push(TrailScalarKey::new(0.0, 1.0));
            keys.push(TrailScalarKey::new(1.0, 1.0));
        }
        Self { keys }
    }

    #[must_use]
    pub fn constant(value: f32) -> Self {
        Self::new([
            TrailScalarKey::new(0.0, value),
            TrailScalarKey::new(1.0, value),
        ])
    }

    #[must_use]
    pub fn linear(start: f32, end: f32) -> Self {
        Self::new([
            TrailScalarKey::new(0.0, start),
            TrailScalarKey::new(1.0, end),
        ])
    }

    #[must_use]
    pub fn evaluate(&self, t: f32) -> f32 {
        evaluate_scalar_curve(&self.keys, t)
    }

    #[must_use]
    pub(crate) fn is_constant(&self) -> bool {
        self.keys
            .windows(2)
            .all(|pair| (pair[0].value - pair[1].value).abs() <= 0.000_1)
    }
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq)]
pub struct TrailScalarKey {
    pub position: f32,
    pub value: f32,
}

impl TrailScalarKey {
    #[must_use]
    pub const fn new(position: f32, value: f32) -> Self {
        Self { position, value }
    }
}

#[derive(Clone, Debug, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct TrailGradient {
    pub keys: Vec<TrailColorKey>,
}

impl TrailGradient {
    #[must_use]
    pub fn new(keys: impl IntoIterator<Item = TrailColorKey>) -> Self {
        let mut keys = keys.into_iter().collect::<Vec<_>>();
        keys.sort_by(|a, b| a.position.total_cmp(&b.position));
        if keys.is_empty() {
            keys.push(TrailColorKey::new(0.0, Color::WHITE));
            keys.push(TrailColorKey::new(1.0, Color::WHITE));
        }
        Self { keys }
    }

    #[must_use]
    pub fn constant(color: Color) -> Self {
        Self::new([
            TrailColorKey::new(0.0, color),
            TrailColorKey::new(1.0, color),
        ])
    }

    #[must_use]
    pub fn evaluate(&self, t: f32) -> LinearRgba {
        evaluate_color_curve(&self.keys, t)
    }

    #[must_use]
    pub(crate) fn is_constant_gradient(&self) -> bool {
        self.keys.windows(2).all(|pair| {
            let a = pair[0].color.to_linear();
            let b = pair[1].color.to_linear();
            (a.red - b.red).abs() <= 0.0001
                && (a.green - b.green).abs() <= 0.0001
                && (a.blue - b.blue).abs() <= 0.0001
                && (a.alpha - b.alpha).abs() <= 0.0001
        })
    }
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq)]
pub struct TrailColorKey {
    pub position: f32,
    pub color: Color,
}

impl TrailColorKey {
    #[must_use]
    pub const fn new(position: f32, color: Color) -> Self {
        Self { position, color }
    }
}

fn evaluate_scalar_curve(keys: &[TrailScalarKey], t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let Some(first) = keys.first() else {
        return 1.0;
    };
    let Some(last) = keys.last() else {
        return 1.0;
    };
    if t <= first.position {
        return first.value;
    }
    if t >= last.position {
        return last.value;
    }

    for pair in keys.windows(2) {
        let [a, b] = [pair[0], pair[1]];
        if (a.position..=b.position).contains(&t) {
            let span = (b.position - a.position).max(f32::EPSILON);
            let lerp_t = (t - a.position) / span;
            return a.value + (b.value - a.value) * lerp_t;
        }
    }

    last.value
}

fn evaluate_color_curve(keys: &[TrailColorKey], t: f32) -> LinearRgba {
    let t = t.clamp(0.0, 1.0);
    let Some(first) = keys.first() else {
        return Color::WHITE.to_linear();
    };
    let Some(last) = keys.last() else {
        return Color::WHITE.to_linear();
    };
    if t <= first.position {
        return first.color.to_linear();
    }
    if t >= last.position {
        return last.color.to_linear();
    }

    for pair in keys.windows(2) {
        let [a, b] = [pair[0], pair[1]];
        if (a.position..=b.position).contains(&t) {
            let span = (b.position - a.position).max(f32::EPSILON);
            let lerp_t = (t - a.position) / span;
            let a = a.color.to_linear();
            let b = b.color.to_linear();
            return LinearRgba {
                red: a.red + (b.red - a.red) * lerp_t,
                green: a.green + (b.green - a.green) * lerp_t,
                blue: a.blue + (b.blue - a.blue) * lerp_t,
                alpha: a.alpha + (b.alpha - a.alpha) * lerp_t,
            };
        }
    }

    last.color.to_linear()
}

/// Live access to the trail's point history.
///
/// Attached to the **source** entity each frame after sampling. Systems
/// ordered between [`TrailSystems::Sample`](crate::TrailSystems::Sample) and
/// [`TrailSystems::BuildMesh`](crate::TrailSystems::BuildMesh) — in the
/// [`TrailSystems::Modify`](crate::TrailSystems::Modify) set — can read or
/// mutate the points.
///
/// Calling [`points_mut`](Self::points_mut) marks the history as dirty,
/// which triggers a mesh rebuild on the next frame.
#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct TrailHistory {
    points: Vec<SamplePoint>,
    total_length: f32,
    #[reflect(ignore)]
    dirty: bool,
}

impl TrailHistory {
    /// The current sample points, oldest first.
    #[must_use]
    pub fn points(&self) -> &[SamplePoint] {
        &self.points
    }

    /// Mutable access to the sample points.
    /// Accessing this marks the history as dirty, triggering a mesh rebuild.
    pub fn points_mut(&mut self) -> &mut Vec<SamplePoint> {
        self.dirty = true;
        &mut self.points
    }

    /// Total polyline length of the trail.
    #[must_use]
    pub fn total_length(&self) -> f32 {
        self.total_length
    }

    /// Number of active points.
    #[must_use]
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Whether the trail has no active points.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Normalized lengths (0.0 at oldest, 1.0 at newest).
    /// Returns a `Vec` parallel to [`points()`](Self::points).
    #[must_use]
    pub fn normalized_lengths(&self) -> Vec<f32> {
        crate::sampling::normalized_lengths(&self.points)
    }

    pub(crate) fn sync_from_buffer(&mut self, buffer: &crate::sampling::TrailBuffer) {
        self.points.clone_from(&buffer.points);
        self.total_length = crate::sampling::total_length(&buffer.points);
    }

    pub(crate) fn sync_to_buffer(&self, buffer: &mut crate::sampling::TrailBuffer) {
        buffer.points.clone_from(&self.points);
    }

    pub(crate) fn take_dirty(&mut self) -> bool {
        let was = self.dirty;
        self.dirty = false;
        was
    }
}

/// Attach to a trail source entity to override [`Trail::style`] with a
/// shared style. Useful when many trails should share the same visual
/// configuration without duplicating the data in every [`Trail`].
///
/// When present, `Trail::style` is ignored in favor of this value.
/// Removing the component reverts to `Trail::style`.
#[derive(Component, Clone, Debug, Reflect, PartialEq)]
#[reflect(Component)]
pub struct TrailStyleOverride(pub TrailStyle);

/// Links a trail source entity to its render entity.
/// Present on every trail source after the first tick.
#[derive(Component)]
pub struct TrailSourceLink {
    pub(crate) render_entity: Entity,
}

impl TrailSourceLink {
    /// The entity that holds the trail's mesh and material.
    #[must_use]
    pub fn render_entity(&self) -> Entity {
        self.render_entity
    }
}

#[derive(Component)]
pub(crate) struct TrailRenderInstance {
    pub source: Entity,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub config: Trail,
    pub view_state: crate::TrailViewState,
    pub history: crate::sampling::TrailBuffer,
    pub source_missing: bool,
    pub dirty: bool,
    pub uv_scroll_offset: f32,
    pub using_custom_material: bool,
    pub scratch_lengths: Vec<f32>,
}

#[derive(Component)]
pub(crate) struct TrailRenderTag;

pub(crate) fn maybe_disable_frustum_culling(material: &TrailMaterial) -> Option<NoFrustumCulling> {
    material
        .disable_frustum_culling
        .then(NoFrustumCulling::default)
}
