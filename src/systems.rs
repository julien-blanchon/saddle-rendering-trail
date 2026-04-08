use bevy::{camera::primitives::Aabb, camera::visibility::NoFrustumCulling, prelude::*};

use crate::{
    Trail, TrailDiagnostics, TrailHistory, TrailRuntimeState, TrailSpace, TrailStyleOverride,
    TrailViewSource, TrailViewState,
    components::{
        TrailCustomMaterial, TrailLod, TrailRenderInstance, TrailRenderTag, TrailSourceLink,
        maybe_disable_frustum_culling,
    },
    events::{TrailEmissionStarted, TrailFullyFaded, TrailOrphaned, TrailReset},
    mesh_builder::{apply_buffers, build_mesh, camera_position_for_space},
    sampling::EmitResult,
};

type SourceQueryItem = (
    Entity,
    &'static Trail,
    &'static Transform,
    &'static GlobalTransform,
    Option<&'static ChildOf>,
    &'static TrailSourceLink,
    Option<&'static TrailCustomMaterial>,
    Option<&'static TrailLod>,
    Option<&'static TrailStyleOverride>,
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
    global_transforms: Query<&GlobalTransform, Without<TrailRenderInstance>>,
) {
    let selected = cameras
        .iter()
        .filter(|(_, camera)| camera.is_active)
        .min_by_key(|(_, camera)| camera.order);

    let previous = *view_state;
    if let Some((entity, _)) = selected {
        let Ok(global_transform) = global_transforms.get(entity) else {
            *view_state = TrailViewState {
                changed: previous.camera_entity.is_some(),
                ..default()
            };
            return;
        };
        let (_, current_rotation, current_position) =
            global_transform.to_scale_rotation_translation();
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
    sources: Query<
        (Entity, &Trail, Option<&Name>, Option<&TrailCustomMaterial>),
        Without<TrailSourceLink>,
    >,
) {
    for (source, trail, name, custom_material) in &sources {
        let mesh = meshes.add(Mesh::new(
            bevy::render::render_resource::PrimitiveTopology::TriangleList,
            bevy::asset::RenderAssetUsages::MAIN_WORLD
                | bevy::asset::RenderAssetUsages::RENDER_WORLD,
        ));

        let (material, using_custom) = if let Some(custom) = custom_material {
            (custom.0.clone(), true)
        } else {
            (
                materials.add(trail.style.material.to_standard_material()),
                false,
            )
        };

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
                view_state: default(),
                history: default(),
                source_missing: false,
                dirty: true,
                uv_scroll_offset: 0.0,
                using_custom_material: using_custom,
                scratch_lengths: Vec::new(),
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
        commands.entity(source).insert((
            TrailSourceLink { render_entity },
            TrailHistory::default(),
        ));
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_sources_and_sample(
    mut commands: Commands,
    time: Res<Time>,
    auto_view_state: Res<TrailViewState>,
    mut runtime: ResMut<TrailRuntimeState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut instances: Query<(Entity, &mut TrailRenderInstance, &mut Transform), Without<Trail>>,
    sources: Query<SourceQueryItem, SourceQueryFilter>,
    global_transforms: Query<&GlobalTransform, Without<TrailRenderInstance>>,
    mut histories: Query<&mut TrailHistory>,
) {
    let delta_secs = time.delta_secs();

    for (source, trail, transform, global_transform, parent, link, custom_material, lod, style_override) in
        &sources
    {
        let effective_style = style_override
            .map(|o| &o.0)
            .unwrap_or(&trail.style);
        let Ok((render_entity, mut instance, mut render_transform)) =
            instances.get_mut(link.render_entity)
        else {
            continue;
        };
        instance.source = source;
        instance.source_missing = false;

        let config_changed =
            instance.config != *trail || instance.config.style != *effective_style;
        let no_cull_changed = instance.config.style.material.disable_frustum_culling
            != effective_style.material.disable_frustum_culling;
        let resolved_view_state = resolve_view_state(
            trail.view_source,
            *auto_view_state,
            instance.view_state,
            &global_transforms,
        );
        let view_changed = resolved_view_state.changed;
        instance.view_state = resolved_view_state;

        // LOD: compute effective max points based on camera distance.
        let effective_max_points =
            if let (Some(lod), Some(camera_pos)) = (lod, instance.view_state.camera_position) {
                let world_pos = global_transform.translation();
                lod.effective_max_points(camera_pos.distance(world_pos), trail.max_points)
            } else {
                trail.max_points
            };

        let history_changed =
            instance
                .history
                .advance(delta_secs, trail.lifetime_secs, effective_max_points);
        let age_animation_dirty =
            effective_style.animates_over_age() && !instance.history.points.is_empty();

        // UV scroll
        let scroll_active =
            effective_style.uv_scroll_speed.abs() > f32::EPSILON
                && !instance.history.points.is_empty();
        if scroll_active {
            instance.uv_scroll_offset += effective_style.uv_scroll_speed * delta_secs;
        }

        if no_cull_changed {
            sync_frustum_culling(
                &mut commands,
                render_entity,
                effective_style.material.disable_frustum_culling,
            );
        }

        // Handle custom material changes.
        let custom_changed = match (custom_material, instance.using_custom_material) {
            (Some(custom), true) => {
                // User updated the handle.
                if instance.material != custom.0 {
                    instance.material = custom.0.clone();
                    commands
                        .entity(render_entity)
                        .insert(MeshMaterial3d(custom.0.clone()));
                    true
                } else {
                    false
                }
            }
            (Some(custom), false) => {
                // User added a custom material.
                instance.material = custom.0.clone();
                instance.using_custom_material = true;
                commands
                    .entity(render_entity)
                    .insert(MeshMaterial3d(custom.0.clone()));
                true
            }
            (None, true) => {
                // User removed custom material, revert to auto.
                let new_mat = materials.add(effective_style.material.to_standard_material());
                instance.material = new_mat.clone();
                instance.using_custom_material = false;
                commands
                    .entity(render_entity)
                    .insert(MeshMaterial3d(new_mat));
                true
            }
            (None, false) => false,
        };

        instance.config = trail.clone();
        instance.config.style = effective_style.clone();
        if config_changed && !instance.using_custom_material {
            if let Some(material) = materials.get_mut(&instance.material) {
                *material = effective_style.material.to_standard_material();
            }
        }

        *render_transform = match trail.space {
            TrailSpace::World => Transform::default(),
            TrailSpace::Local => parent
                .and_then(|p| global_transforms.get(p.parent()).ok())
                .map(|gt| gt.compute_transform())
                .unwrap_or_default(),
        };

        let (_, world_rotation, world_position) =
            global_transform.to_scale_rotation_translation();
        let sample_position = match trail.space {
            TrailSpace::World => world_position,
            TrailSpace::Local => transform.translation,
        };
        let sample_rotation = match trail.space {
            TrailSpace::World => world_rotation,
            TrailSpace::Local => transform.rotation,
        };

        let was_empty = instance.history.points.is_empty();
        let emit_result = instance
            .history
            .maybe_emit(trail, sample_position, sample_rotation);
        let lod_trimmed_after_emit = instance.history.trim_to_max_points(effective_max_points);
        if emit_result == EmitResult::ResetAndAppended {
            runtime.total_resets += 1;
            commands.trigger(TrailReset { entity: source });
        }
        if was_empty && emit_result != EmitResult::Ignored {
            commands.trigger(TrailEmissionStarted { entity: source });
        }

        instance.dirty = instance.dirty
            || config_changed
            || custom_changed
            || history_changed
            || lod_trimmed_after_emit
            || age_animation_dirty
            || scroll_active
            || emit_result != EmitResult::Ignored
            || matches!(trail.orientation, crate::TrailOrientation::Billboard) && view_changed;

        // Sync the public TrailHistory component on the source entity.
        if let Ok(mut history) = histories.get_mut(source) {
            history.sync_from_buffer(&instance.history);
        }
    }
}

pub(crate) fn tick_orphaned_instances(
    mut commands: Commands,
    time: Res<Time>,
    auto_view_state: Res<TrailViewState>,
    all_entities: Query<()>,
    global_transforms: Query<&GlobalTransform, Without<TrailRenderInstance>>,
    mut instances: Query<(Entity, &mut TrailRenderInstance), Without<Trail>>,
) {
    let delta_secs = time.delta_secs();
    for (render_entity, mut instance) in &mut instances {
        if !instance.source_missing && all_entities.get(instance.source).is_err() {
            instance.source_missing = true;
            commands.trigger(TrailOrphaned {
                entity: render_entity,
                former_source: instance.source,
            });
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
        let resolved_view_state = resolve_view_state(
            instance.config.view_source,
            *auto_view_state,
            instance.view_state,
            &global_transforms,
        );
        let view_changed = resolved_view_state.changed;
        instance.view_state = resolved_view_state;
        let age_animation_dirty =
            instance.config.style.animates_over_age() && !instance.history.points.is_empty();
        let scroll_active = instance.config.style.uv_scroll_speed.abs() > f32::EPSILON
            && !instance.history.points.is_empty();
        if scroll_active {
            instance.uv_scroll_offset += instance.config.style.uv_scroll_speed * delta_secs;
        }
        let changed = instance
            .history
            .advance(delta_secs, lifetime_secs, max_points);
        instance.dirty = instance.dirty
            || changed
            || age_animation_dirty
            || scroll_active
            || is_billboard && view_changed;
    }
}

/// Syncs user mutations from [`TrailHistory`] back into the render instance.
/// Runs at the start of [`TrailSystems::BuildMesh`] after the user's
/// modifier systems in [`TrailSystems::Modify`] have had a chance to run.
pub(crate) fn sync_history_mutations(
    mut instances: Query<&mut TrailRenderInstance>,
    mut sources: Query<(&TrailSourceLink, &mut TrailHistory), Without<TrailRenderInstance>>,
) {
    for (link, mut history) in &mut sources {
        if !history.take_dirty() {
            continue;
        }
        let Ok(mut instance) = instances.get_mut(link.render_entity) else {
            continue;
        };
        history.sync_to_buffer(&mut instance.history);
        instance.dirty = true;
    }
}

pub(crate) fn rebuild_dirty_meshes(
    mut runtime: ResMut<TrailRuntimeState>,
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

        let camera_position = instance.view_state.camera_position.map(|position| {
            camera_position_for_space(instance.config.space, render_transform, position)
        });
        // Take scratch buffer out to avoid borrow conflict with instance.history.
        let mut scratch = std::mem::take(&mut instance.scratch_lengths);
        let buffers = build_mesh(
            &instance.history.points,
            &instance.config,
            camera_position,
            instance.uv_scroll_offset,
            &mut scratch,
        );
        instance.scratch_lengths = scratch;
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

fn resolve_view_state(
    view_source: TrailViewSource,
    auto_view_state: TrailViewState,
    previous: TrailViewState,
    global_transforms: &Query<&GlobalTransform, Without<TrailRenderInstance>>,
) -> TrailViewState {
    let mut next = match view_source {
        TrailViewSource::ActiveCamera3d => TrailViewState {
            changed: false,
            ..auto_view_state
        },
        TrailViewSource::Entity(entity) => global_transforms.get(entity).map_or_else(
            |_| TrailViewState::default(),
            |gt| {
                let (_, rotation, position) = gt.to_scale_rotation_translation();
                TrailViewState {
                    camera_entity: Some(entity),
                    camera_position: Some(position),
                    camera_rotation: Some(rotation),
                    changed: false,
                }
            },
        ),
    };

    next.changed = view_state_changed(previous, next);
    next
}

fn view_state_changed(previous: TrailViewState, current: TrailViewState) -> bool {
    previous.camera_entity != current.camera_entity
        || previous
            .camera_position
            .zip(current.camera_position)
            .is_some_and(|(a, b)| a.distance(b) > 0.0001)
        || previous.camera_position.is_some() != current.camera_position.is_some()
        || previous
            .camera_rotation
            .zip(current.camera_rotation)
            .is_some_and(|(a, b)| a.dot(b).abs() < 0.999_999)
        || previous.camera_rotation.is_some() != current.camera_rotation.is_some()
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
                commands.trigger(TrailFullyFaded {
                    entity,
                    former_source: instance.source,
                });
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
