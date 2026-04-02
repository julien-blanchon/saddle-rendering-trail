use bevy::{camera::primitives::Aabb, camera::visibility::NoFrustumCulling, prelude::*};

use crate::{
    Trail, TrailDiagnostics, TrailRuntimeState, TrailSpace, TrailViewState,
    components::{
        TrailRenderInstance, TrailRenderTag, TrailSourceLink, maybe_disable_frustum_culling,
    },
    mesh_builder::{apply_buffers, build_mesh, camera_position_for_space},
    sampling::EmitResult,
};

type SourceQueryItem = (
    Entity,
    &'static Trail,
    &'static Transform,
    Option<&'static ChildOf>,
    &'static TrailSourceLink,
);

type SourceQueryFilter = (Without<TrailRenderTag>, Without<TrailRenderInstance>);

pub(crate) fn runtime_is_active(runtime: Res<TrailRuntimeState>) -> bool {
    runtime.active
}

pub(crate) fn activate_runtime(mut runtime: ResMut<TrailRuntimeState>) {
    runtime.active = true;
}

pub(crate) fn deactivate_runtime(
    mut commands: Commands,
    mut runtime: ResMut<TrailRuntimeState>,
    sources: Query<(Entity, &Trail, &TrailSourceLink)>,
    mut instances: Query<&mut TrailRenderInstance>,
) {
    runtime.active = false;

    for (source, trail, link) in &sources {
        let Ok(mut instance) = instances.get_mut(link.render_entity) else {
            continue;
        };
        if trail.clear_on_deactivate {
            commands.entity(link.render_entity).despawn();
            commands.entity(source).remove::<TrailSourceLink>();
        } else {
            instance.history.clear();
            instance.dirty = true;
        }
    }
}

pub(crate) fn refresh_view_state(
    mut view_state: ResMut<TrailViewState>,
    cameras: Query<(Entity, &Camera), With<Camera3d>>,
    transforms: Query<(&Transform, Option<&ChildOf>), Without<TrailRenderInstance>>,
) {
    let selected = cameras
        .iter()
        .filter(|(_, camera)| camera.is_active)
        .min_by_key(|(_, camera)| camera.order);

    let previous = *view_state;
    if let Some((entity, _)) = selected {
        let Some(transform) = current_world_transform(entity, &transforms) else {
            *view_state = TrailViewState {
                changed: previous.camera_entity.is_some(),
                ..default()
            };
            return;
        };
        let current_position = transform.translation;
        let current_rotation = transform.rotation;
        *view_state = TrailViewState {
            camera_entity: Some(entity),
            camera_position: Some(current_position),
            camera_rotation: Some(current_rotation),
            changed: previous.camera_entity != Some(entity)
                || previous
                    .camera_position
                    .is_none_or(|position| position.distance(current_position) > 0.0001)
                || previous
                    .camera_rotation
                    .is_none_or(|rotation| rotation.dot(current_rotation).abs() < 0.999_999),
        };
    } else {
        *view_state = TrailViewState {
            changed: previous.camera_entity.is_some(),
            ..default()
        };
    }
}

pub(crate) fn spawn_missing_instances(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sources: Query<(Entity, &Trail, Option<&Name>), Without<TrailSourceLink>>,
) {
    for (source, trail, name) in &sources {
        let mesh = meshes.add(Mesh::new(
            bevy::render::render_resource::PrimitiveTopology::TriangleList,
            bevy::asset::RenderAssetUsages::MAIN_WORLD
                | bevy::asset::RenderAssetUsages::RENDER_WORLD,
        ));
        let material = materials.add(trail.style.material.to_standard_material());
        let render_name = name
            .map(|name| format!("{name} Trail"))
            .unwrap_or_else(|| "Trail Render".to_string());

        let mut entity_commands = commands.spawn((
            Name::new(render_name),
            TrailRenderTag,
            TrailRenderInstance {
                source,
                mesh: mesh.clone(),
                material: material.clone(),
                config: trail.clone(),
                history: default(),
                source_missing: false,
                dirty: true,
            },
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::default(),
            Visibility::Hidden,
            Aabb::from_min_max(Vec3::ZERO, Vec3::ZERO),
        ));
        if let Some(no_cull) = maybe_disable_frustum_culling(&trail.style.material) {
            entity_commands.insert(no_cull);
        }
        let render_entity = entity_commands.id();
        commands
            .entity(source)
            .insert(TrailSourceLink { render_entity });
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_sources_and_sample(
    mut commands: Commands,
    time: Res<Time>,
    view_state: Res<TrailViewState>,
    mut runtime: ResMut<TrailRuntimeState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut instances: Query<(Entity, &mut TrailRenderInstance, &mut Transform), Without<Trail>>,
    sources: Query<SourceQueryItem, SourceQueryFilter>,
    transforms: Query<(&Transform, Option<&ChildOf>), Without<TrailRenderInstance>>,
) {
    let delta_secs = time.delta_secs();

    for (source, trail, transform, parent, link) in &sources {
        let Ok((render_entity, mut instance, mut render_transform)) =
            instances.get_mut(link.render_entity)
        else {
            continue;
        };
        instance.source = source;
        instance.source_missing = false;

        let config_changed = instance.config != *trail;
        let no_cull_changed = instance.config.style.material.disable_frustum_culling
            != trail.style.material.disable_frustum_culling;
        let history_changed =
            instance
                .history
                .advance(delta_secs, trail.lifetime_secs, trail.max_points);
        let age_animation_dirty =
            trail.style.animates_alpha_over_age() && !instance.history.points.is_empty();

        if no_cull_changed {
            sync_frustum_culling(
                &mut commands,
                render_entity,
                trail.style.material.disable_frustum_culling,
            );
        }
        instance.config = trail.clone();
        if config_changed {
            if let Some(material) = materials.get_mut(&instance.material) {
                *material = trail.style.material.to_standard_material();
            }
        }

        *render_transform = match trail.space {
            TrailSpace::World => Transform::default(),
            TrailSpace::Local => parent
                .and_then(|parent| current_world_transform(parent.parent(), &transforms))
                .unwrap_or_default(),
        };

        let world_transform = current_world_transform(source, &transforms).unwrap_or(*transform);
        let sample_position = match trail.space {
            TrailSpace::World => world_transform.translation,
            TrailSpace::Local => transform.translation,
        };
        let sample_rotation = match trail.space {
            TrailSpace::World => world_transform.rotation,
            TrailSpace::Local => transform.rotation,
        };

        let emit_result = instance
            .history
            .maybe_emit(trail, sample_position, sample_rotation);
        if emit_result == EmitResult::ResetAndAppended {
            runtime.total_resets += 1;
        }

        instance.dirty = instance.dirty
            || config_changed
            || history_changed
            || age_animation_dirty
            || emit_result != EmitResult::Ignored
            || matches!(trail.orientation, crate::TrailOrientation::Billboard)
                && view_state.changed;
    }
}

pub(crate) fn tick_orphaned_instances(
    time: Res<Time>,
    view_state: Res<TrailViewState>,
    all_entities: Query<()>,
    mut instances: Query<&mut TrailRenderInstance, Without<Trail>>,
) {
    let delta_secs = time.delta_secs();
    for mut instance in &mut instances {
        if !instance.source_missing && all_entities.get(instance.source).is_err() {
            instance.source_missing = true;
        }

        if !instance.source_missing {
            continue;
        }

        let lifetime_secs = instance.config.lifetime_secs;
        let max_points = instance.config.max_points;
        let is_billboard = matches!(
            instance.config.orientation,
            crate::TrailOrientation::Billboard
        );
        let age_animation_dirty =
            instance.config.style.animates_alpha_over_age() && !instance.history.points.is_empty();
        let changed = instance
            .history
            .advance(delta_secs, lifetime_secs, max_points);
        instance.dirty =
            instance.dirty || changed || age_animation_dirty || is_billboard && view_state.changed;
    }
}

pub(crate) fn rebuild_dirty_meshes(
    mut runtime: ResMut<TrailRuntimeState>,
    view_state: Res<TrailViewState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut instances: Query<(
        &mut TrailRenderInstance,
        &mut Visibility,
        &mut Aabb,
        &Transform,
    )>,
) {
    for (mut instance, mut visibility, mut aabb, render_transform) in &mut instances {
        if !instance.dirty {
            continue;
        }

        let camera_position = view_state.camera_position.map(|position| {
            camera_position_for_space(instance.config.space, render_transform, position)
        });
        let buffers = build_mesh(&instance.history.points, &instance.config, camera_position);
        let visible = buffers.visible;
        let bounds = buffers.aabb;

        if let Some(mesh) = meshes.get_mut(&instance.mesh) {
            apply_buffers(mesh, buffers);
        }

        if visible {
            *visibility = Visibility::Visible;
            if let Some(bounds) = bounds {
                *aabb = bounds;
            }
        } else {
            *visibility = Visibility::Hidden;
        }

        instance.dirty = false;
        runtime.total_mesh_rebuilds += 1;
    }
}

fn sync_frustum_culling(commands: &mut Commands, render_entity: Entity, disable: bool) {
    if disable {
        commands.entity(render_entity).insert(NoFrustumCulling);
    } else {
        commands.entity(render_entity).remove::<NoFrustumCulling>();
    }
}

fn current_world_transform(
    entity: Entity,
    transforms: &Query<(&Transform, Option<&ChildOf>), Without<TrailRenderInstance>>,
) -> Option<Transform> {
    let (transform, mut parent) = transforms.get(entity).ok()?;
    let mut world_transform = *transform;
    while let Some(link) = parent {
        let (parent_transform, next_parent) = transforms.get(link.parent()).ok()?;
        world_transform = parent_transform.mul_transform(world_transform);
        parent = next_parent;
    }
    Some(world_transform)
}

pub(crate) fn handle_removed_sources(
    mut commands: Commands,
    mut removed_sources: RemovedComponents<Trail>,
    mut instances: Query<(Entity, &mut TrailRenderInstance)>,
    all_entities: Query<()>,
) {
    for source in removed_sources.read() {
        let source_still_exists = all_entities.get(source).is_ok();
        for (render_entity, mut instance) in &mut instances {
            if instance.source != source {
                continue;
            }

            if source_still_exists {
                commands.entity(render_entity).despawn();
                commands.entity(source).remove::<TrailSourceLink>();
            } else if instance.config.keep_after_source_despawn {
                instance.source_missing = true;
            } else {
                commands.entity(render_entity).despawn();
            }
        }
    }
}

pub(crate) fn cleanup_dead_instances(
    mut commands: Commands,
    mut instances: Query<(Entity, &TrailRenderInstance, &mut Visibility)>,
) {
    for (entity, instance, mut visibility) in &mut instances {
        if instance.history.points.is_empty() {
            *visibility = Visibility::Hidden;
            if instance.source_missing {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub(crate) fn publish_diagnostics(
    runtime: Res<TrailRuntimeState>,
    mut diagnostics: ResMut<TrailDiagnostics>,
    sources: Query<&Trail>,
    instances: Query<(&TrailRenderInstance, &Visibility)>,
) {
    diagnostics.runtime_active = runtime.active;
    diagnostics.active_sources = sources.iter().count();
    diagnostics.active_render_entities = instances.iter().count();
    diagnostics.orphaned_render_entities = instances
        .iter()
        .filter(|(instance, _)| instance.source_missing)
        .count();
    diagnostics.active_points = instances
        .iter()
        .map(|(instance, _)| instance.history.points.len())
        .sum();
    diagnostics.visible_trails = instances
        .iter()
        .filter(|(_, visibility)| matches!(visibility, Visibility::Visible | Visibility::Inherited))
        .count();
    diagnostics.dirty_trails = instances
        .iter()
        .filter(|(instance, _)| instance.dirty)
        .count();
    diagnostics.total_mesh_rebuilds = runtime.total_mesh_rebuilds;
    diagnostics.total_resets = runtime.total_resets;
}
