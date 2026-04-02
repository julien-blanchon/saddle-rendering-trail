use bevy::prelude::*;

#[derive(Resource, Clone, Debug, Default, Reflect, PartialEq)]
#[reflect(Resource, Default)]
pub struct TrailDiagnostics {
    pub runtime_active: bool,
    pub active_sources: usize,
    pub active_render_entities: usize,
    pub orphaned_render_entities: usize,
    pub active_points: usize,
    pub visible_trails: usize,
    pub dirty_trails: usize,
    pub total_mesh_rebuilds: u64,
    pub total_resets: u64,
}
