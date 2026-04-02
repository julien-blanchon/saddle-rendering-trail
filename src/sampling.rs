use bevy::prelude::*;

use crate::{Trail, TrailEmitterMode};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SamplePoint {
    pub position: Vec3,
    pub rotation: Quat,
    pub age_secs: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TrailBuffer {
    pub points: Vec<SamplePoint>,
    pub time_since_emit_secs: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EmitResult {
    Ignored,
    Appended,
    ResetAndAppended,
}

impl TrailBuffer {
    pub fn clear(&mut self) {
        self.points.clear();
        self.time_since_emit_secs = 0.0;
    }

    pub fn advance(&mut self, delta_secs: f32, lifetime_secs: f32, max_points: usize) -> bool {
        self.time_since_emit_secs += delta_secs.max(0.0);
        if self.points.is_empty() {
            return false;
        }

        for point in &mut self.points {
            point.age_secs += delta_secs;
        }

        let before = self.points.len();
        self.points
            .retain(|point| point.age_secs <= lifetime_secs.max(0.0));
        trim_to_max_points(&mut self.points, max_points);
        before != self.points.len()
    }

    pub fn maybe_emit(&mut self, trail: &Trail, position: Vec3, rotation: Quat) -> EmitResult {
        if trail.emitter_mode == TrailEmitterMode::Disabled {
            return EmitResult::Ignored;
        }

        let sample = SamplePoint {
            position,
            rotation,
            age_secs: 0.0,
        };

        let Some(last) = self.points.last().copied() else {
            self.points.push(sample);
            self.time_since_emit_secs = 0.0;
            trim_to_max_points(&mut self.points, trail.max_points);
            return EmitResult::Appended;
        };

        let distance = last.position.distance(position);
        if should_reset(last.position, position, trail.teleport_distance) {
            self.points.clear();
            self.points.push(sample);
            self.time_since_emit_secs = 0.0;
            return EmitResult::ResetAndAppended;
        }

        if !should_emit_sample(
            trail.emitter_mode,
            distance,
            trail.min_sample_distance,
            self.time_since_emit_secs,
            trail.max_sample_interval_secs,
        ) {
            return EmitResult::Ignored;
        }

        self.points.push(sample);
        self.time_since_emit_secs = 0.0;
        trim_to_max_points(&mut self.points, trail.max_points);
        EmitResult::Appended
    }
}

pub(crate) fn should_reset(
    last_position: Vec3,
    new_position: Vec3,
    teleport_distance: f32,
) -> bool {
    teleport_distance > 0.0 && last_position.distance(new_position) >= teleport_distance
}

pub(crate) fn should_emit_sample(
    emitter_mode: TrailEmitterMode,
    distance_from_last: f32,
    min_sample_distance: f32,
    time_since_emit_secs: f32,
    max_sample_interval_secs: f32,
) -> bool {
    match emitter_mode {
        TrailEmitterMode::Disabled => false,
        TrailEmitterMode::Always => {
            distance_from_last >= min_sample_distance
                || time_since_emit_secs >= max_sample_interval_secs.max(0.0)
        }
        TrailEmitterMode::WhenMoving => {
            distance_from_last >= min_sample_distance
                || (distance_from_last > 0.0001
                    && time_since_emit_secs >= max_sample_interval_secs.max(0.0))
        }
    }
}

pub(crate) fn total_length(points: &[SamplePoint]) -> f32 {
    points
        .windows(2)
        .map(|pair| pair[0].position.distance(pair[1].position))
        .sum()
}

pub(crate) fn normalized_lengths(points: &[SamplePoint]) -> Vec<f32> {
    if points.is_empty() {
        return Vec::new();
    }
    if points.len() == 1 {
        return vec![1.0];
    }

    let mut lengths = Vec::with_capacity(points.len());
    lengths.push(0.0);
    for pair in points.windows(2) {
        let next = lengths.last().copied().unwrap_or_default()
            + pair[0].position.distance(pair[1].position);
        lengths.push(next);
    }
    let total = lengths.last().copied().unwrap_or_default();
    if total <= f32::EPSILON {
        return vec![0.0; points.len() - 1]
            .into_iter()
            .chain([1.0])
            .collect();
    }
    lengths.into_iter().map(|length| length / total).collect()
}

fn trim_to_max_points(points: &mut Vec<SamplePoint>, max_points: usize) {
    if max_points == 0 || points.len() <= max_points {
        return;
    }
    let drop_count = points.len() - max_points;
    points.drain(0..drop_count);
}
