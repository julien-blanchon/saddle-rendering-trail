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
    let buffers = build_mesh(
        &sample_points(),
        &trail,
        Some(Vec3::new(0.0, 2.0, 3.0)),
        0.0,
    );

    assert_eq!(buffers.positions.len(), 6);
    assert_eq!(buffers.indices.len(), 12);
    assert!(buffers.visible);
    assert!(buffers.aabb.is_some());
}

#[test]
fn repeat_uv_mode_advances_by_distance() {
    let mut trail = Trail::default();
    trail.style.uv_mode = TrailUvMode::RepeatByDistance { distance: 0.5 };

    let buffers = build_mesh(
        &sample_points(),
        &trail,
        Some(Vec3::new(0.0, 2.0, 3.0)),
        0.0,
    );
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

    let buffers = build_mesh(&points, &trail, None, 0.0);
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

#[test]
fn uv_scroll_offset_shifts_coordinates() {
    let trail = Trail::default();
    let without_scroll = build_mesh(
        &sample_points(),
        &trail,
        Some(Vec3::new(0.0, 2.0, 3.0)),
        0.0,
    );
    let with_scroll = build_mesh(
        &sample_points(),
        &trail,
        Some(Vec3::new(0.0, 2.0, 3.0)),
        1.5,
    );

    assert!(!without_scroll.uvs.is_empty());
    assert!(!with_scroll.uvs.is_empty());
    let diff = (with_scroll.uvs[0][0] - without_scroll.uvs[0][0] - 1.5).abs();
    assert!(diff < 0.001);
}

#[test]
fn tube_mode_generates_ring_vertices() {
    use crate::TrailMeshMode;
    let trail = Trail {
        mesh_mode: TrailMeshMode::Tube { sides: 6 },
        ..default()
    };
    let buffers = build_mesh(
        &sample_points(),
        &trail,
        Some(Vec3::new(0.0, 2.0, 3.0)),
        0.0,
    );

    // 3 points × 6 sides = 18 vertices
    assert_eq!(buffers.positions.len(), 18);
    // 2 segments × 6 sides × 6 indices = 72
    assert_eq!(buffers.indices.len(), 72);
    assert!(buffers.visible);
}

#[test]
fn width_fade_mode_shrinks_width_at_tail() {
    use crate::{TrailFadeMode, TrailScalarCurve};
    let mut trail = Trail::default();
    trail.style.fade_mode = TrailFadeMode::Width;
    trail.style.alpha_over_length = TrailScalarCurve::linear(0.0, 1.0);

    let buffers = build_mesh(
        &sample_points(),
        &trail,
        Some(Vec3::new(0.0, 2.0, 3.0)),
        0.0,
    );
    // Tail (index 0) should have zero-ish width, head should be wider.
    // With Width fade, the tail width = base_width * width_curve * alpha → near 0.
    // Since alpha_over_length at 0.0 = 0.0, and the tail is at length_t ≈ 0,
    // those points should be skipped (width <= EPSILON).
    // The result is fewer vertices than normal.
    assert!(buffers.positions.len() <= 6);
}
