use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

mod components;
mod debug;
mod mesh_builder;
mod resources;
mod sampling;
mod systems;

pub use components::{
    Trail, TrailColorKey, TrailDebugSettings, TrailEmitterMode, TrailGradient, TrailMaterial,
    TrailOrientation, TrailScalarCurve, TrailScalarKey, TrailSpace, TrailStyle, TrailUvMode,
};
pub use resources::TrailDiagnostics;

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TrailSystems {
    Sample,
    BuildMesh,
    Cleanup,
    Diagnostics,
    Debug,
}

#[derive(Resource, Default)]
pub(crate) struct TrailRuntimeState {
    pub active: bool,
    pub total_resets: u64,
    pub total_mesh_rebuilds: u64,
}

#[derive(Resource, Default, Clone, Copy)]
pub(crate) struct TrailViewState {
    pub camera_entity: Option<Entity>,
    pub camera_position: Option<Vec3>,
    pub camera_rotation: Option<Quat>,
    pub changed: bool,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

pub struct TrailPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl TrailPlugin {
    #[must_use]
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    #[must_use]
    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for TrailPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for TrailPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        app.init_resource::<TrailRuntimeState>()
            .init_resource::<TrailViewState>()
            .init_resource::<TrailDebugSettings>()
            .init_resource::<TrailDiagnostics>()
            .register_type::<Trail>()
            .register_type::<TrailColorKey>()
            .register_type::<TrailDebugSettings>()
            .register_type::<TrailDiagnostics>()
            .register_type::<TrailEmitterMode>()
            .register_type::<TrailGradient>()
            .register_type::<TrailMaterial>()
            .register_type::<TrailOrientation>()
            .register_type::<TrailScalarCurve>()
            .register_type::<TrailScalarKey>()
            .register_type::<TrailSpace>()
            .register_type::<TrailStyle>()
            .register_type::<TrailUvMode>()
            .add_systems(self.activate_schedule, systems::activate_runtime)
            .add_systems(self.deactivate_schedule, systems::deactivate_runtime)
            .configure_sets(
                self.update_schedule,
                (
                    TrailSystems::Sample,
                    TrailSystems::BuildMesh,
                    TrailSystems::Cleanup,
                    TrailSystems::Diagnostics,
                    TrailSystems::Debug,
                )
                    .chain(),
            );

        app.add_systems(
            self.update_schedule,
            (
                (
                    systems::spawn_missing_instances,
                    systems::refresh_view_state,
                    systems::sync_sources_and_sample,
                    systems::tick_orphaned_instances,
                )
                    .run_if(systems::runtime_is_active)
                    .chain()
                    .in_set(TrailSystems::Sample),
                systems::rebuild_dirty_meshes.in_set(TrailSystems::BuildMesh),
                (
                    systems::handle_removed_sources,
                    systems::cleanup_dead_instances,
                )
                    .chain()
                    .in_set(TrailSystems::Cleanup),
                systems::publish_diagnostics.in_set(TrailSystems::Diagnostics),
            ),
        );

        if app
            .world()
            .contains_resource::<bevy::prelude::GizmoConfigStore>()
        {
            app.add_systems(
                self.update_schedule,
                debug::draw_debug
                    .run_if(systems::runtime_is_active)
                    .in_set(TrailSystems::Debug),
            );
        }
    }
}

#[cfg(test)]
#[path = "sampling_tests.rs"]
mod sampling_tests;

#[cfg(test)]
#[path = "mesh_builder_tests.rs"]
mod mesh_builder_tests;

#[cfg(test)]
#[path = "systems_tests.rs"]
mod systems_tests;
