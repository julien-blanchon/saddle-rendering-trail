use bevy::prelude::*;
use saddle_bevy_e2e::action::Action;

use crate::LabEntities;

pub fn focus_camera(world: &mut World, camera: Entity, from: Vec3, look_at: Vec3) {
    let mut camera_entity = world.entity_mut(camera);
    let mut transform = camera_entity
        .get_mut::<Transform>()
        .expect("lab camera should exist");
    *transform = Transform::from_translation(from).looking_at(look_at, Vec3::Y);
}

pub fn focus_camera_action(from: Vec3, look_at: Vec3) -> Action {
    Action::Custom(Box::new(move |world| {
        let lab = *world.resource::<LabEntities>();
        focus_camera(world, lab.camera, from, look_at);
    }))
}

pub fn billboard_view_action() -> Action {
    focus_camera_action(Vec3::new(-4.4, 2.6, 6.8), Vec3::new(-4.2, 1.8, 0.0))
}

pub fn locked_view_action() -> Action {
    focus_camera_action(Vec3::new(2.8, 3.2, 6.5), Vec3::new(0.0, 1.6, 0.0))
}

pub fn side_swipe_view_action() -> Action {
    focus_camera_action(Vec3::new(7.0, 3.0, 3.5), Vec3::new(0.0, 1.6, 0.0))
}

pub fn hover_view_action() -> Action {
    focus_camera_action(Vec3::new(5.8, 4.2, 9.6), Vec3::new(4.0, 1.6, 0.0))
}

pub fn remember_rebuilds_action() -> Action {
    Action::Custom(Box::new(|world| {
        let rebuilds = world.resource::<saddle_rendering_trail::TrailDiagnostics>().total_mesh_rebuilds;
        world.insert_resource(super::RebuildSnapshot(rebuilds));
    }))
}

pub fn clear_rebuild_snapshot_action() -> Action {
    Action::Custom(Box::new(|world| {
        world.remove_resource::<super::RebuildSnapshot>();
    }))
}
