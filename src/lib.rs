use std::marker::PhantomData;

use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

mod components;
mod debug;
mod events;
mod mesh_builder;
mod resources;
mod sampling;
mod systems;

pub use components::{
    Trail, TrailColorKey, TrailCustomMaterial, TrailDebugSettings, TrailEmitterMode, TrailFadeMode,
    TrailGradient, TrailHistory, TrailLod, TrailMaterial, TrailMaterial3d, TrailMeshMode,
    TrailOrientation, TrailScalarCurve, TrailScalarKey, TrailSourceLink, TrailSpace, TrailStyle,
    TrailStyleOverride, TrailUvMode, TrailViewSource,
};
pub use events::{TrailEmissionStarted, TrailFullyFaded, TrailOrphaned, TrailReset};
pub use resources::TrailDiagnostics;
pub use sampling::SamplePoint as TrailSamplePoint;

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TrailSystems {
    /// Point sampling: spawn render entities, read transforms, emit new points.
    Sample,
    /// Empty slot for user modifier systems that read/write [`TrailHistory`].
    Modify,
    /// Mesh generation from the current point buffer.
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
            .register_type::<TrailViewSource>()
            .register_type::<TrailFadeMode>()
            .register_type::<TrailMeshMode>()
            .register_type::<TrailLod>()
            .register_type::<TrailHistory>()
            .register_type::<TrailStyleOverride>()
            .register_type::<sampling::SamplePoint>()
            .add_systems(self.activate_schedule, systems::activate_runtime)
            .add_systems(self.deactivate_schedule, systems::deactivate_runtime)
            .configure_sets(
                self.update_schedule,
                (
                    TrailSystems::Sample,
                    TrailSystems::Modify,
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
                systems::sync_history_mutations.in_set(TrailSystems::BuildMesh),
                systems::rebuild_dirty_meshes
                    .after(systems::sync_history_mutations)
                    .in_set(TrailSystems::BuildMesh),
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

/// Plugin extension for custom material types. Add after [`TrailPlugin`]
/// for each custom material type you use.
///
/// ```rust,ignore
/// app.add_plugins(TrailPlugin::default());
/// app.add_plugins(TrailMaterialPlugin::<MyMaterial>::new(Update));
/// ```
pub struct TrailMaterialPlugin<M: Material> {
    update_schedule: Interned<dyn ScheduleLabel>,
    _marker: PhantomData<M>,
}

impl<M: Material> TrailMaterialPlugin<M> {
    #[must_use]
    pub fn new(update_schedule: impl ScheduleLabel) -> Self {
        Self {
            update_schedule: update_schedule.intern(),
            _marker: PhantomData,
        }
    }
}

impl<M: Material> Plugin for TrailMaterialPlugin<M> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            self.update_schedule,
            sync_trail_material::<M>
                .after(TrailSystems::Sample)
                .before(TrailSystems::BuildMesh),
        );
    }
}

fn sync_trail_material<M: Material>(
    sources: Query<(&TrailSourceLink, &TrailMaterial3d<M>), Changed<TrailMaterial3d<M>>>,
    mut commands: Commands,
) {
    for (link, material) in &sources {
        commands
            .entity(link.render_entity)
            .insert(MeshMaterial3d(material.0.clone()))
            .remove::<MeshMaterial3d<StandardMaterial>>();
    }
}

#[cfg(test)]
#[path = "sampling_tests.rs"]
mod sampling_tests;

#[cfg(test)]
#[path = "components_tests.rs"]
mod components_tests;

#[cfg(test)]
#[path = "mesh_builder_tests.rs"]
mod mesh_builder_tests;

#[cfg(test)]
#[path = "systems_tests.rs"]
mod systems_tests;
