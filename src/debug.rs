use bevy::{camera::primitives::Aabb, prelude::*};

use crate::{
    TrailDebugSettings,
    components::TrailRenderInstance,
    mesh_builder::{build_mesh, camera_position_for_space},
    resources::TrailDiagnostics,
};

pub(crate) fn draw_debug(
    global_debug: Res<TrailDebugSettings>,
    diagnostics: Res<TrailDiagnostics>,
    instances: Query<(&TrailRenderInstance, &Transform, Option<&Aabb>)>,
    per_entity_debug: Query<&TrailDebugSettings, Without<TrailRenderInstance>>,
    mut gizmos: Gizmos,
) {
    if diagnostics.active_render_entities == 0 {
        return;
    }

    if !global_debug.enabled && per_entity_debug.is_empty() {
        return;
    }

    for (instance, render_transform, aabb) in &instances {
        let debug = per_entity_debug
            .get(instance.source)
            .unwrap_or(&global_debug);

        if !debug.enabled {
            continue;
        }

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
            let camera_position = instance.view_state.camera_position.map(|position| {
                camera_position_for_space(instance.config.space, render_transform, position)
            });
            let mut scratch = Vec::new();
            let buffers = build_mesh(
                &instance.history.points,
                &instance.config,
                camera_position,
                0.0,
                &mut scratch,
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
