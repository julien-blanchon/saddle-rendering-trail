use bevy::prelude::*;

use crate::{
    Trail, TrailEmitterMode,
    sampling::{
        EmitResult, TrailBuffer, normalized_lengths, should_emit_sample, should_reset, total_length,
    },
};

#[test]
fn min_distance_suppresses_redundant_samples() {
    let trail = Trail {
        min_sample_distance: 0.5,
        max_sample_interval_secs: 10.0,
        ..default()
    };

    let mut buffer = TrailBuffer::default();
    assert_eq!(
        buffer.maybe_emit(&trail, Vec3::ZERO, Quat::IDENTITY),
        EmitResult::Appended
    );
    assert_eq!(
        buffer.maybe_emit(&trail, Vec3::new(0.1, 0.0, 0.0), Quat::IDENTITY),
        EmitResult::Ignored
    );
    assert_eq!(buffer.points.len(), 1);
}

#[test]
fn lifetime_expiration_prunes_old_points() {
    let mut buffer = TrailBuffer::default();
    buffer.points.push(crate::sampling::SamplePoint {
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        age_secs: 0.95,
    });
    buffer.points.push(crate::sampling::SamplePoint {
        position: Vec3::X,
        rotation: Quat::IDENTITY,
        age_secs: 0.2,
    });

    let changed = buffer.advance(0.1, 1.0, 8);
    assert!(changed);
    assert_eq!(buffer.points.len(), 1);
    assert_eq!(buffer.points[0].position, Vec3::X);
}

#[test]
fn teleport_reset_clears_previous_history() {
    let trail = Trail {
        teleport_distance: 2.0,
        ..default()
    };
    let mut buffer = TrailBuffer::default();
    assert_eq!(
        buffer.maybe_emit(&trail, Vec3::ZERO, Quat::IDENTITY),
        EmitResult::Appended
    );
    assert_eq!(
        buffer.maybe_emit(&trail, Vec3::new(3.0, 0.0, 0.0), Quat::IDENTITY),
        EmitResult::ResetAndAppended
    );
    assert_eq!(buffer.points.len(), 1);
    assert_eq!(buffer.points[0].position, Vec3::new(3.0, 0.0, 0.0));
}

#[test]
fn normalized_length_evaluation_tracks_path_progress() {
    let points = vec![
        crate::sampling::SamplePoint {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            age_secs: 0.0,
        },
        crate::sampling::SamplePoint {
            position: Vec3::X,
            rotation: Quat::IDENTITY,
            age_secs: 0.0,
        },
        crate::sampling::SamplePoint {
            position: Vec3::new(3.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            age_secs: 0.0,
        },
    ];

    assert_eq!(total_length(&points), 3.0);
    assert_eq!(normalized_lengths(&points), vec![0.0, 1.0 / 3.0, 1.0]);
}

#[test]
fn slow_motion_emits_on_interval_when_requested() {
    assert!(should_emit_sample(
        TrailEmitterMode::Always,
        0.0,
        1.0,
        0.05,
        0.04,
    ));
    assert!(should_emit_sample(
        TrailEmitterMode::WhenMoving,
        0.01,
        1.0,
        0.05,
        0.04,
    ));
    assert!(!should_emit_sample(
        TrailEmitterMode::WhenMoving,
        0.0,
        1.0,
        0.05,
        0.04,
    ));
}

#[test]
fn reset_helper_uses_teleport_threshold() {
    assert!(should_reset(Vec3::ZERO, Vec3::new(5.0, 0.0, 0.0), 3.0));
    assert!(!should_reset(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), 3.0));
}
