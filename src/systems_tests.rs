use std::time::Duration;

use bevy::{
    asset::AssetPlugin,
    ecs::schedule::ScheduleLabel,
    prelude::*,
    time::TimeUpdateStrategy,
    transform::TransformPlugin,
};

use crate::{
    Trail, TrailDiagnostics, TrailEmitterMode, TrailOrientation, TrailPlugin,
    TrailScalarCurve,
    components::TrailSourceLink,
};

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Activate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Deactivate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Tick;

fn init_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), TransformPlugin));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(16)));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_schedule(Activate);
    app.init_schedule(Deactivate);
    app.init_schedule(Tick);
    app.add_plugins(TrailPlugin::new(Activate, Deactivate, Tick));
    app
}

fn spawn_source(app: &mut App, trail: Trail) -> Entity {
    app.world_mut()
        .spawn((
            Name::new("Trail Source"),
            trail,
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
        ))
        .id()
}

fn run_tick(app: &mut App) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_millis(16));
    app.world_mut().run_schedule(Tick);
}

fn set_source_x(app: &mut App, source: Entity, x: f32) {
    if let Some(mut transform) = app.world_mut().get_mut::<Transform>(source) {
        transform.translation.x = x;
    }
}

fn render_entity_for(app: &App, source: Entity) -> Entity {
    app.world()
        .get::<TrailSourceLink>(source)
        .expect("trail source link should exist")
        .render_entity
}

fn visibility_for(app: &App, entity: Entity) -> Option<Visibility> {
    app.world().get::<Visibility>(entity).cloned()
}

#[test]
fn activation_and_tick_spawn_render_entity() {
    let mut app = init_app();
    let source = spawn_source(&mut app, Trail::default());

    app.world_mut().run_schedule(Activate);
    run_tick(&mut app);

    let link = app
        .world()
        .get::<TrailSourceLink>(source)
        .expect("trail source link should be inserted");
    assert!(app.world().get_entity(link.render_entity).is_ok());
}

#[test]
fn deactivation_clears_instances_when_configured() {
    let mut app = init_app();
    let source = spawn_source(&mut app, Trail::default());

    app.world_mut().run_schedule(Activate);
    run_tick(&mut app);
    let render_entity = render_entity_for(&app, source);

    app.world_mut().run_schedule(Deactivate);
    run_tick(&mut app);

    assert!(app.world().get_entity(render_entity).is_err());
    assert!(app.world().get::<TrailSourceLink>(source).is_none());
}

#[test]
fn deactivate_without_clearing_hides_until_reactivated() {
    let mut trail = Trail::default()
        .with_orientation(TrailOrientation::TransformLocked { axis: Vec3::Y })
        .with_lifetime_secs(10.0);
    trail.clear_on_deactivate = false;

    let mut app = init_app();
    let source = spawn_source(&mut app, trail);

    app.world_mut().run_schedule(Activate);
    run_tick(&mut app);
    set_source_x(&mut app, source, 0.6);
    run_tick(&mut app);

    let render_entity = render_entity_for(&app, source);
    assert!(matches!(
        visibility_for(&app, render_entity),
        Some(Visibility::Visible | Visibility::Inherited)
    ));

    app.world_mut().run_schedule(Deactivate);
    run_tick(&mut app);

    assert!(app.world().get_entity(render_entity).is_ok());
    assert!(matches!(
        visibility_for(&app, render_entity),
        Some(Visibility::Hidden)
    ));

    let rebuilds_while_inactive = app.world().resource::<TrailDiagnostics>().total_mesh_rebuilds;
    set_source_x(&mut app, source, 1.2);
    run_tick(&mut app);
    assert_eq!(
        app.world().resource::<TrailDiagnostics>().total_mesh_rebuilds,
        rebuilds_while_inactive
    );

    app.world_mut().run_schedule(Activate);
    set_source_x(&mut app, source, 1.8);
    run_tick(&mut app);
    set_source_x(&mut app, source, 2.4);
    run_tick(&mut app);

    assert!(matches!(
        visibility_for(&app, render_entity),
        Some(Visibility::Visible | Visibility::Inherited)
    ));
}

#[test]
fn source_despawn_can_leave_decay_history_alive() {
    let mut app = init_app();
    let source = spawn_source(
        &mut app,
        Trail::default()
            .with_emitter_mode(TrailEmitterMode::Always)
            .with_lifetime_secs(0.2),
    );

    app.world_mut().run_schedule(Activate);
    for frame in 0..4 {
        if let Some(mut transform) = app.world_mut().get_mut::<Transform>(source) {
            transform.translation.x = frame as f32 * 0.4;
        }
        run_tick(&mut app);
    }

    let render_entity = app
        .world()
        .get::<TrailSourceLink>(source)
        .expect("trail source link should exist")
        .render_entity;
    app.world_mut().despawn(source);
    run_tick(&mut app);

    assert!(app.world().get_entity(render_entity).is_ok());

    for _ in 0..30 {
        run_tick(&mut app);
    }

    assert!(app.world().get_entity(render_entity).is_err());
}

#[test]
fn stationary_trails_do_not_rebuild_when_age_fade_is_constant() {
    let mut app = init_app();
    let source = spawn_source(
        &mut app,
        Trail::default()
            .with_orientation(TrailOrientation::TransformLocked { axis: Vec3::Y })
            .with_lifetime_secs(10.0),
    );

    app.world_mut().run_schedule(Activate);
    run_tick(&mut app);
    set_source_x(&mut app, source, 0.6);
    run_tick(&mut app);

    let rebuilds_before_idle = app.world().resource::<TrailDiagnostics>().total_mesh_rebuilds;
    run_tick(&mut app);

    assert_eq!(
        app.world().resource::<TrailDiagnostics>().total_mesh_rebuilds,
        rebuilds_before_idle
    );
}

#[test]
fn stationary_trails_rebuild_when_age_fade_animates() {
    let mut trail = Trail::default()
        .with_orientation(TrailOrientation::TransformLocked { axis: Vec3::Y })
        .with_lifetime_secs(10.0);
    trail.style.alpha_over_age = TrailScalarCurve::linear(1.0, 0.0);

    let mut app = init_app();
    let source = spawn_source(&mut app, trail);

    app.world_mut().run_schedule(Activate);
    run_tick(&mut app);
    set_source_x(&mut app, source, 0.6);
    run_tick(&mut app);

    let rebuilds_before_idle = app.world().resource::<TrailDiagnostics>().total_mesh_rebuilds;
    run_tick(&mut app);

    assert!(
        app.world().resource::<TrailDiagnostics>().total_mesh_rebuilds > rebuilds_before_idle
    );
}

#[test]
fn config_changes_rebuild_even_when_stationary() {
    let mut app = init_app();
    let source = spawn_source(
        &mut app,
        Trail::default()
            .with_orientation(TrailOrientation::TransformLocked { axis: Vec3::Y })
            .with_lifetime_secs(10.0),
    );

    app.world_mut().run_schedule(Activate);
    run_tick(&mut app);
    set_source_x(&mut app, source, 0.6);
    run_tick(&mut app);

    let rebuilds_before_config_change = app.world().resource::<TrailDiagnostics>().total_mesh_rebuilds;
    if let Some(mut trail) = app.world_mut().get_mut::<Trail>(source) {
        trail.style.base_width = 0.9;
    }
    run_tick(&mut app);

    assert!(
        app.world().resource::<TrailDiagnostics>().total_mesh_rebuilds
            > rebuilds_before_config_change
    );
}

#[test]
fn billboard_trails_rebuild_when_camera_changes() {
    let mut app = init_app();
    spawn_source(&mut app, Trail::default().with_orientation(TrailOrientation::Billboard));
    app.world_mut().spawn((
        Camera3d::default(),
        Camera::default(),
        Transform::from_xyz(0.0, 2.0, 5.0),
        GlobalTransform::default(),
    ));

    app.world_mut().run_schedule(Activate);
    run_tick(&mut app);
    let initial = app.world().resource::<TrailDiagnostics>().total_mesh_rebuilds;

    let camera = {
        let mut query = app.world_mut().query_filtered::<Entity, With<Camera3d>>();
        query
            .single(app.world())
            .expect("camera should exist")
    };
    if let Some(mut transform) = app.world_mut().get_mut::<Transform>(camera) {
        transform.translation.x += 1.0;
    }
    run_tick(&mut app);

    assert!(app.world().resource::<TrailDiagnostics>().total_mesh_rebuilds > initial);
}
