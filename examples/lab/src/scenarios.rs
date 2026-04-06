use bevy::prelude::*;
use saddle_bevy_e2e::{
    action::Action,
    actions::{assertions, inspect},
    scenario::Scenario,
};
use saddle_rendering_trail::{Trail, TrailDiagnostics, TrailOrientation, TrailSpace, TrailViewSource};

use crate::LabEntities;

#[derive(Resource)]
struct RebuildSnapshot(u64);

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "trail_smoke" => Some(build_smoke()),
        "trail_billboard" => Some(build_billboard()),
        "trail_locked" => Some(build_locked()),
        "trail_reset" => Some(build_reset()),
        "trail_view_source" => Some(build_view_source()),
        _ => None,
    }
}

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "trail_smoke",
        "trail_billboard",
        "trail_locked",
        "trail_reset",
        "trail_view_source",
    ]
}

fn build_smoke() -> Scenario {
    Scenario::builder("trail_smoke")
        .description("Boot the trail lab, verify render entities and live point generation, then capture a wide screenshot.")
        .then(Action::WaitFrames(90))
        .then(assertions::resource_exists::<TrailDiagnostics>(
            "trail diagnostics exists",
        ))
        .then(assertions::resource_satisfies::<TrailDiagnostics>(
            "showcase spawns active trails",
            |diagnostics| {
                diagnostics.runtime_active
                    && diagnostics.active_sources >= 4
                    && diagnostics.active_render_entities >= 4
                    && diagnostics.active_points >= 24
                    && diagnostics.total_mesh_rebuilds > 0
            },
        ))
        .then(assertions::custom("all lab source entities are still present", |world| {
            let lab = *world.resource::<LabEntities>();
            [lab.billboard, lab.locked, lab.hover, lab.teleporter]
                .into_iter()
                .all(|entity| world.get_entity(entity).is_ok())
        }))
        .then(assertions::custom("hover source uses local trail space", |world| {
            let entity = world.resource::<LabEntities>().hover;
            world
                .get::<Trail>(entity)
                .is_some_and(|trail| trail.space == TrailSpace::Local)
        }))
        .then(inspect::log_resource::<TrailDiagnostics>("trail_smoke diagnostics"))
        .then(assertions::log_summary("trail_smoke summary"))
        .then(Action::Screenshot("trail_smoke".into()))
        .build()
}

fn build_billboard() -> Scenario {
    Scenario::builder("trail_billboard")
        .description("Frame the billboard contrail, move the camera, and verify the billboard trail rebuilds for the new view.")
        .then(Action::WaitFrames(50))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let lab = *world.resource::<LabEntities>();
            focus_camera(world, lab.camera, Vec3::new(-4.4, 2.6, 6.8), Vec3::new(-4.2, 1.8, 0.0));
            let rebuilds = world.resource::<TrailDiagnostics>().total_mesh_rebuilds;
            world.insert_resource(RebuildSnapshot(rebuilds));
        })))
        .then(Action::WaitFrames(12))
        .then(assertions::custom("billboard source keeps billboard orientation", |world| {
            let entity = world.resource::<LabEntities>().billboard;
            world
                .get::<Trail>(entity)
                .is_some_and(|trail| matches!(trail.orientation, TrailOrientation::Billboard))
        }))
        .then(Action::Screenshot("trail_billboard_before".into()))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let lab = *world.resource::<LabEntities>();
            focus_camera(world, lab.camera, Vec3::new(-1.8, 4.0, 9.0), Vec3::new(-4.0, 1.6, 0.0));
        })))
        .then(Action::WaitFrames(12))
        .then(assertions::custom("camera motion triggers extra billboard rebuilds", |world| {
            let before = world
                .get_resource::<RebuildSnapshot>()
                .expect("rebuild snapshot should exist")
                .0;
            world.resource::<TrailDiagnostics>().total_mesh_rebuilds > before
        }))
        .then(Action::Screenshot("trail_billboard_after".into()))
        .then(Action::Custom(Box::new(|world: &mut World| {
            world.remove_resource::<RebuildSnapshot>();
        })))
        .then(assertions::log_summary("trail_billboard summary"))
        .build()
}

fn build_locked() -> Scenario {
    Scenario::builder("trail_locked")
        .description("Frame the transform-locked swipe trail and assert the source uses a non-billboard orientation.")
        .then(Action::WaitFrames(45))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let lab = *world.resource::<LabEntities>();
            focus_camera(world, lab.camera, Vec3::new(2.8, 3.2, 6.5), Vec3::new(0.0, 1.6, 0.0));
        })))
        .then(Action::WaitFrames(10))
        .then(assertions::custom("locked source stays transform-locked", |world| {
            let entity = world.resource::<LabEntities>().locked;
            world
                .get::<Trail>(entity)
                .is_some_and(|trail| matches!(trail.orientation, TrailOrientation::TransformLocked { .. }))
        }))
        .then(assertions::resource_satisfies::<TrailDiagnostics>(
            "locked mode produces visible trail geometry",
            |diagnostics| diagnostics.visible_trails >= 1 && diagnostics.active_points >= 20,
        ))
        .then(Action::Screenshot("trail_locked_before".into()))
        .then(Action::WaitFrames(18))
        .then(Action::Screenshot("trail_locked_after".into()))
        .then(assertions::log_summary("trail_locked summary"))
        .build()
}

fn build_reset() -> Scenario {
    Scenario::builder("trail_reset")
        .description("Wait for teleport resets, then verify the diagnostics count increments and capture the before/after state.")
        .then(Action::WaitFrames(24))
        .then(Action::Screenshot("trail_reset_before".into()))
        .then(Action::WaitFrames(120))
        .then(assertions::resource_satisfies::<TrailDiagnostics>(
            "teleport trail produced at least one reset",
            |diagnostics| diagnostics.total_resets > 0,
        ))
        .then(assertions::custom("teleport source still exists", |world| {
            let entity = world.resource::<LabEntities>().teleporter;
            world.get_entity(entity).is_ok()
        }))
        .then(inspect::log_resource::<TrailDiagnostics>("trail_reset diagnostics"))
        .then(Action::Screenshot("trail_reset_after".into()))
        .then(assertions::log_summary("trail_reset summary"))
        .build()
}

fn build_view_source() -> Scenario {
    Scenario::builder("trail_view_source")
        .description("Verify the lab sources bind their trail view source to the lab camera entity and capture the configured result.")
        .then(Action::WaitFrames(36))
        .then(assertions::custom("billboard source uses explicit lab camera view source", |world| {
            let lab = *world.resource::<LabEntities>();
            world
                .get::<Trail>(lab.billboard)
                .is_some_and(|trail| trail.view_source == TrailViewSource::Entity(lab.camera))
        }))
        .then(assertions::custom("locked source uses explicit lab camera view source", |world| {
            let lab = *world.resource::<LabEntities>();
            world
                .get::<Trail>(lab.locked)
                .is_some_and(|trail| trail.view_source == TrailViewSource::Entity(lab.camera))
        }))
        .then(Action::Screenshot("trail_view_source".into()))
        .then(inspect::log_resource::<TrailDiagnostics>("trail_view_source diagnostics"))
        .then(assertions::log_summary("trail_view_source summary"))
        .build()
}

fn focus_camera(world: &mut World, camera: Entity, from: Vec3, look_at: Vec3) {
    let mut camera_entity = world.entity_mut(camera);
    let mut transform = camera_entity
        .get_mut::<Transform>()
        .expect("lab camera should exist");
    *transform = Transform::from_translation(from).looking_at(look_at, Vec3::Y);
}
