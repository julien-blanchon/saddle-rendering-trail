use bevy::{
    asset::RenderAssetUsages,
    camera::primitives::Aabb,
    mesh::{Indices, Mesh},
    prelude::*,
    render::render_resource::PrimitiveTopology,
};

use crate::{
    Trail, TrailFadeMode, TrailMeshMode, TrailOrientation, TrailSpace, TrailUvMode,
    sampling::{SamplePoint, normalized_lengths},
};

#[derive(Debug, Default)]
pub(crate) struct TrailMeshBuffers {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
    pub aabb: Option<Aabb>,
    pub visible: bool,
}

pub(crate) fn build_mesh(
    points: &[SamplePoint],
    trail: &Trail,
    camera_position_in_trail_space: Option<Vec3>,
    uv_scroll_offset: f32,
) -> TrailMeshBuffers {
    match trail.mesh_mode {
        TrailMeshMode::Ribbon => build_ribbon_mesh(
            points,
            trail,
            camera_position_in_trail_space,
            uv_scroll_offset,
        ),
        TrailMeshMode::Tube { sides } => build_tube_mesh(
            points,
            trail,
            camera_position_in_trail_space,
            sides.max(3),
            uv_scroll_offset,
        ),
    }
}

fn build_ribbon_mesh(
    points: &[SamplePoint],
    trail: &Trail,
    camera_position_in_trail_space: Option<Vec3>,
    uv_scroll_offset: f32,
) -> TrailMeshBuffers {
    if points.len() < 2 {
        return TrailMeshBuffers::default();
    }

    let mut buffers = TrailMeshBuffers::default();
    let normalized = normalized_lengths(points);
    let total_length = crate::sampling::total_length(points);
    if total_length <= f32::EPSILON {
        return buffers;
    }

    let mut accumulated_length = 0.0;
    let mut mins = Vec3::splat(f32::INFINITY);
    let mut maxs = Vec3::splat(f32::NEG_INFINITY);

    for (index, point) in points.iter().enumerate() {
        if index > 0 {
            accumulated_length += points[index - 1].position.distance(point.position);
        }

        let tangent = tangent_at(points, index);
        let length_t = normalized[index];
        let age_t = (point.age_secs / trail.lifetime_secs.max(f32::EPSILON)).clamp(0.0, 1.0);
        let half_width = compute_width_with_fade(trail, length_t, age_t) * 0.5;
        if half_width <= f32::EPSILON {
            continue;
        }
        let side = side_vector(trail, *point, tangent, camera_position_in_trail_space);
        let normal = side.cross(tangent).normalize_or_zero();
        let left = point.position - side * half_width;
        let right = point.position + side * half_width;
        let color = compute_color_with_fade(trail, length_t, age_t);
        let u = compute_u(trail, length_t, accumulated_length, uv_scroll_offset);

        buffers.positions.push(left.to_array());
        buffers.positions.push(right.to_array());
        buffers.normals.push(normal.to_array());
        buffers.normals.push(normal.to_array());
        buffers.uvs.push([u, 0.0]);
        buffers.uvs.push([u, 1.0]);
        buffers
            .colors
            .push([color.red, color.green, color.blue, color.alpha]);
        buffers
            .colors
            .push([color.red, color.green, color.blue, color.alpha]);

        mins = mins.min(left).min(right);
        maxs = maxs.max(left).max(right);
    }

    if buffers.positions.len() < 4 {
        return TrailMeshBuffers::default();
    }

    let point_count = buffers.positions.len() / 2;
    for index in 0..(point_count - 1) as u32 {
        let base = index * 2;
        buffers.indices.extend_from_slice(&[
            base,
            base + 2,
            base + 1,
            base + 1,
            base + 2,
            base + 3,
        ]);
    }

    buffers.aabb = Some(Aabb::from_min_max(mins, maxs));
    buffers.visible = true;
    buffers
}

fn build_tube_mesh(
    points: &[SamplePoint],
    trail: &Trail,
    camera_position_in_trail_space: Option<Vec3>,
    sides: u32,
    uv_scroll_offset: f32,
) -> TrailMeshBuffers {
    if points.len() < 2 {
        return TrailMeshBuffers::default();
    }

    let mut buffers = TrailMeshBuffers::default();
    let normalized = normalized_lengths(points);
    let total_length = crate::sampling::total_length(points);
    if total_length <= f32::EPSILON {
        return buffers;
    }

    let mut accumulated_length = 0.0;
    let mut mins = Vec3::splat(f32::INFINITY);
    let mut maxs = Vec3::splat(f32::NEG_INFINITY);
    let mut ring_count: u32 = 0;

    for (index, point) in points.iter().enumerate() {
        if index > 0 {
            accumulated_length += points[index - 1].position.distance(point.position);
        }

        let tangent = tangent_at(points, index);
        let length_t = normalized[index];
        let age_t = (point.age_secs / trail.lifetime_secs.max(f32::EPSILON)).clamp(0.0, 1.0);
        let radius = compute_width_with_fade(trail, length_t, age_t) * 0.5;
        if radius <= f32::EPSILON {
            continue;
        }
        let color = compute_color_with_fade(trail, length_t, age_t);
        let u = compute_u(trail, length_t, accumulated_length, uv_scroll_offset);

        let side = side_vector(trail, *point, tangent, camera_position_in_trail_space);
        let up = side.cross(tangent).normalize_or_zero();

        for i in 0..sides {
            let angle = (i as f32 / sides as f32) * std::f32::consts::TAU;
            let local = side * angle.cos() + up * angle.sin();
            let pos = point.position + local * radius;
            let normal = local.normalize_or_zero();
            let v = i as f32 / sides as f32;

            buffers.positions.push(pos.to_array());
            buffers.normals.push(normal.to_array());
            buffers.uvs.push([u, v]);
            buffers
                .colors
                .push([color.red, color.green, color.blue, color.alpha]);

            mins = mins.min(pos);
            maxs = maxs.max(pos);
        }
        ring_count += 1;
    }

    if ring_count < 2 {
        return TrailMeshBuffers::default();
    }

    for ring in 0..(ring_count - 1) {
        for i in 0..sides {
            let next_i = (i + 1) % sides;
            let current_base = ring * sides;
            let next_base = (ring + 1) * sides;

            buffers.indices.extend_from_slice(&[
                current_base + i,
                next_base + i,
                current_base + next_i,
                current_base + next_i,
                next_base + i,
                next_base + next_i,
            ]);
        }
    }

    buffers.aabb = Some(Aabb::from_min_max(mins, maxs));
    buffers.visible = true;
    buffers
}

fn compute_width_with_fade(trail: &Trail, length_t: f32, age_t: f32) -> f32 {
    let base = trail.style.evaluate_width(length_t);
    match trail.style.fade_mode {
        TrailFadeMode::Alpha => base,
        TrailFadeMode::Width | TrailFadeMode::Both => {
            let alpha = trail.style.alpha_over_length.evaluate(length_t)
                * trail.style.alpha_over_age.evaluate(age_t);
            base * alpha.clamp(0.0, 1.0)
        }
    }
}

fn compute_color_with_fade(trail: &Trail, length_t: f32, age_t: f32) -> LinearRgba {
    match trail.style.fade_mode {
        TrailFadeMode::Alpha | TrailFadeMode::Both => trail.style.evaluate_color(length_t, age_t),
        TrailFadeMode::Width => {
            // Don't apply alpha fade to color, only the color gradient.
            trail.style.color_over_length.evaluate(length_t)
        }
    }
}

fn compute_u(trail: &Trail, length_t: f32, accumulated_length: f32, uv_scroll_offset: f32) -> f32 {
    let base_u = match trail.style.uv_mode {
        TrailUvMode::Stretch => length_t,
        TrailUvMode::RepeatByDistance { distance } => accumulated_length / distance.max(0.001),
    };
    base_u + uv_scroll_offset
}

pub(crate) fn apply_buffers(mesh: &mut Mesh, buffers: TrailMeshBuffers) {
    *mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, buffers.positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, buffers.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, buffers.uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, buffers.colors)
    .with_inserted_indices(Indices::U32(buffers.indices));
}

fn tangent_at(points: &[SamplePoint], index: usize) -> Vec3 {
    let previous = index.saturating_sub(1);
    let next = (index + 1).min(points.len() - 1);
    let tangent = points[next].position - points[previous].position;
    if tangent.length_squared() <= f32::EPSILON {
        Vec3::Y
    } else {
        tangent.normalize()
    }
}

fn side_vector(
    trail: &Trail,
    point: SamplePoint,
    tangent: Vec3,
    camera_position_in_trail_space: Option<Vec3>,
) -> Vec3 {
    match trail.orientation {
        TrailOrientation::Billboard => {
            let fallback = point.rotation * Vec3::Y;
            if let Some(camera_position) = camera_position_in_trail_space {
                let view = (camera_position - point.position).normalize_or_zero();
                let side = tangent.cross(view).normalize_or_zero();
                if side.length_squared() > f32::EPSILON {
                    return side;
                }
            }
            projected_axis(fallback, tangent)
        }
        TrailOrientation::TransformLocked { axis } => {
            projected_axis(point.rotation * axis, tangent)
        }
    }
}

fn projected_axis(axis: Vec3, tangent: Vec3) -> Vec3 {
    let projected = axis - tangent * axis.dot(tangent);
    if projected.length_squared() > f32::EPSILON {
        projected.normalize()
    } else {
        tangent.any_orthonormal_vector()
    }
}

pub(crate) fn camera_position_for_space(
    trail_space: TrailSpace,
    render_transform: &Transform,
    camera_world_position: Vec3,
) -> Vec3 {
    match trail_space {
        TrailSpace::World => camera_world_position,
        TrailSpace::Local => render_transform
            .to_matrix()
            .inverse()
            .transform_point3(camera_world_position),
    }
}
