use bevy::prelude::*;

use crate::{Trail, TrailDebugSettings, TrailStyle, TrailViewSource};

#[test]
fn default_style_is_neutral() {
    let style = TrailStyle::default();

    assert_eq!(style.width_over_length.evaluate(0.0), 1.0);
    assert_eq!(style.width_over_length.evaluate(1.0), 1.0);
    assert_eq!(style.alpha_over_length.evaluate(0.0), 1.0);
    assert_eq!(style.alpha_over_length.evaluate(1.0), 1.0);
    assert_eq!(style.evaluate_color(0.0, 0.0), Color::WHITE.to_linear());
    assert_eq!(style.evaluate_color(1.0, 1.0), Color::WHITE.to_linear());
}

#[test]
fn debug_defaults_skip_anchor_points() {
    let debug = TrailDebugSettings::default();

    assert!(!debug.draw_points);
    assert!(debug.draw_segments);
}

#[test]
fn with_view_entity_sets_explicit_view_source() {
    let entity = Entity::from_bits(7);
    let trail = Trail::default().with_view_entity(entity);

    assert_eq!(trail.view_source, TrailViewSource::Entity(entity));
}
