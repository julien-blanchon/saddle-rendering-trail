use bevy::{
    camera::visibility::NoFrustumCulling,
    color::LinearRgba,
    pbr::StandardMaterial,
    prelude::*,
    reflect::Reflect,
    render::render_resource::Face,
};

#[derive(Component, Clone, Debug, Reflect, PartialEq)]
#[reflect(Component, Default)]
#[require(Transform)]
pub struct Trail {
    pub emitter_mode: TrailEmitterMode,
    pub space: TrailSpace,
    pub orientation: TrailOrientation,
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
}

#[derive(Resource, Clone, Debug, Reflect, PartialEq)]
#[reflect(Resource, Default)]
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
            draw_points: true,
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
    pub color_over_length: TrailGradient,
    pub alpha_over_length: TrailScalarCurve,
    pub alpha_over_age: TrailScalarCurve,
    pub uv_mode: TrailUvMode,
    pub material: TrailMaterial,
}

impl Default for TrailStyle {
    fn default() -> Self {
        Self {
            base_width: 0.35,
            width_over_length: TrailScalarCurve::linear(0.45, 1.0),
            color_over_length: TrailGradient::new([
                TrailColorKey::new(0.0, Color::srgb(0.65, 0.72, 1.0)),
                TrailColorKey::new(1.0, Color::srgb(1.0, 1.0, 1.0)),
            ]),
            alpha_over_length: TrailScalarCurve::new([
                TrailScalarKey::new(0.0, 0.0),
                TrailScalarKey::new(0.15, 0.35),
                TrailScalarKey::new(1.0, 1.0),
            ]),
            alpha_over_age: TrailScalarCurve::constant(1.0),
            uv_mode: TrailUvMode::Stretch,
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
        let mut color = self.color_over_length.evaluate(length_t);
        let alpha = self.alpha_over_length.evaluate(length_t) * self.alpha_over_age.evaluate(age_t);
        color.alpha *= alpha.clamp(0.0, 1.0);
        color
    }

    #[must_use]
    pub fn evaluate_width(&self, length_t: f32) -> f32 {
        (self.base_width * self.width_over_length.evaluate(length_t)).max(0.0)
    }

    #[must_use]
    pub(crate) fn animates_alpha_over_age(&self) -> bool {
        !self.alpha_over_age.is_constant()
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

#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq)]
pub enum TrailOrientation {
    #[default]
    Billboard,
    TransformLocked { axis: Vec3 },
}

#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq)]
pub enum TrailUvMode {
    #[default]
    Stretch,
    RepeatByDistance { distance: f32 },
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
        Self::new([TrailColorKey::new(0.0, color), TrailColorKey::new(1.0, color)])
    }

    #[must_use]
    pub fn evaluate(&self, t: f32) -> LinearRgba {
        evaluate_color_curve(&self.keys, t)
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

#[derive(Component)]
pub(crate) struct TrailSourceLink {
    pub render_entity: Entity,
}

#[derive(Component)]
pub(crate) struct TrailRenderInstance {
    pub source: Entity,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub config: Trail,
    pub history: crate::sampling::TrailBuffer,
    pub source_missing: bool,
    pub dirty: bool,
}

#[derive(Component)]
pub(crate) struct TrailRenderTag;

pub(crate) fn maybe_disable_frustum_culling(material: &TrailMaterial) -> Option<NoFrustumCulling> {
    material.disable_frustum_culling.then(NoFrustumCulling::default)
}
