use bevy::prelude::*;

use crate::{
    Trail, TrailOrientation, TrailStyle, TrailUvMode,
    mesh_builder::{build_mesh, camera_position_for_space},
    sampling::SamplePoint,
};

fn sample_points() -> Vec<SamplePoint> {
    vec![
        SamplePoint {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            age_secs: 0.9,
        },
        SamplePoint {
            position: Vec3::new(1.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            age_secs: 0.4,
        },
        SamplePoint {
            position: Vec3::new(2.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            age_secs: 0.1,
        },
    ]
}

#[test]
fn mesh_builder_generates_expected_vertex_and_index_counts() {
    let trail = Trail::default();
    let buffers = build_mesh(&sample_points(), &trail, Some(Vec3::new(0.0, 2.0, 3.0)));

    assert_eq!(buffers.positions.len(), 6);
    assert_eq!(buffers.indices.len(), 12);
    assert!(buffers.visible);
    assert!(buffers.aabb.is_some());
}

#[test]
fn repeat_uv_mode_advances_by_distance() {
    let mut trail = Trail::default();
    trail.style.uv_mode = TrailUvMode::RepeatByDistance { distance: 0.5 };

    let buffers = build_mesh(&sample_points(), &trail, Some(Vec3::new(0.0, 2.0, 3.0)));
    assert_eq!(buffers.uvs[0][0], 0.0);
    assert_eq!(buffers.uvs[2][0], 2.0);
    assert_eq!(buffers.uvs[4][0], 4.0);
}

#[test]
fn transform_locked_mode_uses_sampled_rotation_axis() {
    let mut points = sample_points();
    points[1].rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
    let trail = Trail {
        orientation: TrailOrientation::TransformLocked { axis: Vec3::Y },
        style: TrailStyle::default(),
        ..default()
    };

    let buffers = build_mesh(&points, &trail, None);
    let left = Vec3::from_array(buffers.positions[2]);
    let right = Vec3::from_array(buffers.positions[3]);
    assert!(left.y.abs() > 0.1 || right.y.abs() > 0.1);
}

#[test]
fn camera_conversion_respects_local_space_transform() {
    let transform = Transform::from_xyz(2.0, 0.0, 0.0);
    let local = camera_position_for_space(
        crate::TrailSpace::Local,
        &transform,
        Vec3::new(5.0, 0.0, 0.0),
    );
    assert_eq!(local, Vec3::new(3.0, 0.0, 0.0));
}
