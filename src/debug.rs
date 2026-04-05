use bevy::{camera::primitives::Aabb, prelude::*};

use crate::{
    TrailDebugSettings, components::TrailRenderInstance, mesh_builder::build_mesh,
    resources::TrailDiagnostics,
};

pub(crate) fn draw_debug(
    debug: Res<TrailDebugSettings>,
    diagnostics: Res<TrailDiagnostics>,
    instances: Query<(&TrailRenderInstance, &Transform, Option<&Aabb>)>,
    mut gizmos: Gizmos,
) {
    if !debug.enabled || diagnostics.active_render_entities == 0 {
        return;
    }

    for (instance, render_transform, aabb) in &instances {
        if debug.draw_points {
            for point in &instance.history.points {
                let world = render_transform.transform_point(point.position);
                gizmos.sphere(world, debug.point_radius, Color::srgb(0.86, 0.92, 1.0));
            }
        }

        if debug.draw_segments {
            for pair in instance.history.points.windows(2) {
                let a = render_transform.transform_point(pair[0].position);
                let b = render_transform.transform_point(pair[1].position);
                gizmos.line(a, b, Color::srgb(0.35, 0.78, 1.0));
            }
        }

        if debug.draw_normals {
            let camera_position = render_transform.translation + Vec3::new(0.0, 1.0, 4.0);
            let buffers = build_mesh(
                &instance.history.points,
                &instance.config,
                Some(camera_position),
                0.0,
            );
            for segment in buffers.positions.chunks_exact(2) {
                let left = render_transform.transform_point(Vec3::from_array(segment[0]));
                let right = render_transform.transform_point(Vec3::from_array(segment[1]));
                let center = left.lerp(right, 0.5);
                let normal =
                    (right - left).cross(Vec3::Y).normalize_or_zero() * debug.normal_length;
                gizmos.line(center, center + normal, Color::srgb(1.0, 0.65, 0.28));
            }
        }

        if debug.draw_bounds {
            let Some(aabb) = aabb else {
                continue;
            };
            let center = render_transform.transform_point(aabb.center.into());
            gizmos.cube(
                Transform::from_translation(center).with_scale((aabb.half_extents * 2.0).into()),
                Color::srgb(1.0, 0.24, 0.32),
            );
        }
    }
}
