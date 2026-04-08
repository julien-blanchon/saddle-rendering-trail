use bevy::prelude::*;

/// Fired on the source entity when its first point is emitted after
/// the trail was empty or had been reset.
#[derive(EntityEvent, Clone, Debug)]
pub struct TrailEmissionStarted {
    pub entity: Entity,
}

/// Fired on the source entity when the trail history is cleared
/// due to a teleport/discontinuity reset.
#[derive(EntityEvent, Clone, Debug)]
pub struct TrailReset {
    pub entity: Entity,
}

/// Fired on the **render** entity when its source entity despawns
/// and the trail enters orphan-decay mode.
#[derive(EntityEvent, Clone, Debug)]
pub struct TrailOrphaned {
    pub entity: Entity,
    /// The entity ID of the now-despawned source.
    pub former_source: Entity,
}

/// Fired on the **render** entity when an orphaned trail's last point
/// expires and the render entity is about to be despawned.
#[derive(EntityEvent, Clone, Debug)]
pub struct TrailFullyFaded {
    pub entity: Entity,
    /// The entity ID of the original source (may no longer exist).
    pub former_source: Entity,
}
