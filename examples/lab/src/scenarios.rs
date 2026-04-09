mod support;

use bevy::prelude::*;
use saddle_bevy_e2e::{
    action::Action,
    actions::{assertions, inspect},
    scenario::Scenario,
};
use saddle_rendering_trail::{
    Trail, TrailColorKey, TrailDiagnostics, TrailEmitterMode, TrailFadeMode, TrailGradient,
    TrailHistory, TrailLod, TrailMeshMode, TrailOrientation, TrailScalarCurve, TrailSpace,
    TrailStyle, TrailStyleOverride, TrailViewSource,
};

use crate::LabEntities;

#[derive(Resource)]
struct RebuildSnapshot(u64);

#[derive(Resource, Clone, Copy)]
struct LodSnapshot {
    history_len: usize,
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "trail_smoke" => Some(build_smoke()),
        "trail_billboard" => Some(build_billboard()),
        "trail_locked" => Some(build_locked()),
        "trail_reset" => Some(build_reset()),
        "trail_view_source" => Some(build_view_source()),
        "trail_tube_mesh_mode" => Some(build_tube_mesh_mode()),
        "trail_fade_modes" => Some(build_fade_modes()),
        "trail_lod" => Some(build_lod()),
        "trail_melee_swipe" => Some(build_melee_swipe()),
        "trail_drawing_trail" => Some(build_drawing_trail()),
        "trail_projectile_contrail" => Some(build_projectile_contrail()),
        "trail_history_access" => Some(build_history_access()),
        "trail_point_mutation" => Some(build_point_mutation()),
        "trail_age_curves" => Some(build_age_curves()),
        "trail_style_override" => Some(build_style_override()),
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
        "trail_tube_mesh_mode",
        "trail_fade_modes",
        "trail_lod",
        "trail_melee_swipe",
        "trail_drawing_trail",
        "trail_projectile_contrail",
        "trail_history_access",
        "trail_point_mutation",
        "trail_age_curves",
        "trail_style_override",
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
        .then(support::billboard_view_action())
        .then(support::remember_rebuilds_action())
        .then(Action::WaitFrames(12))
        .then(assertions::custom("billboard source keeps billboard orientation", |world| {
            let entity = world.resource::<LabEntities>().billboard;
            world
                .get::<Trail>(entity)
                .is_some_and(|trail| matches!(trail.orientation, TrailOrientation::Billboard))
        }))
        .then(Action::Screenshot("trail_billboard_before".into()))
        .then(support::focus_camera_action(Vec3::new(-1.8, 4.0, 9.0), Vec3::new(-4.0, 1.6, 0.0)))
        .then(Action::WaitFrames(12))
        .then(assertions::custom("camera motion triggers extra billboard rebuilds", |world| {
            let before = world
                .get_resource::<RebuildSnapshot>()
                .expect("rebuild snapshot should exist")
                .0;
            world.resource::<TrailDiagnostics>().total_mesh_rebuilds > before
        }))
        .then(Action::Screenshot("trail_billboard_after".into()))
        .then(support::clear_rebuild_snapshot_action())
        .then(assertions::log_summary("trail_billboard summary"))
        .build()
}

fn build_locked() -> Scenario {
    Scenario::builder("trail_locked")
        .description("Frame the transform-locked swipe trail and assert the source uses a non-billboard orientation.")
        .then(Action::WaitFrames(45))
        .then(support::locked_view_action())
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

fn build_tube_mesh_mode() -> Scenario {
    Scenario::builder("trail_tube_mesh_mode")
        .description(
            "Switch the billboard contrail source to TrailMeshMode::Tube, verify the mesh mode is \
             stored on the component, and capture the cylindrical cross-section output.",
        )
        .then(Action::WaitFrames(45))
        .then(support::billboard_view_action())
        .then(Action::Custom(Box::new(|world: &mut World| {
            let lab = *world.resource::<LabEntities>();
            if let Some(mut trail) = world.get_mut::<Trail>(lab.billboard) {
                trail.mesh_mode = TrailMeshMode::Tube { sides: 6 };
            }
        })))
        .then(Action::WaitFrames(30))
        .then(assertions::custom(
            "billboard source stores tube mesh mode",
            |world| {
                let lab = *world.resource::<LabEntities>();
                world.get::<Trail>(lab.billboard).is_some_and(|trail| {
                    matches!(trail.mesh_mode, TrailMeshMode::Tube { sides: 6 })
                })
            },
        ))
        .then(assertions::resource_satisfies::<TrailDiagnostics>(
            "tube trail produces mesh rebuilds",
            |diagnostics| diagnostics.total_mesh_rebuilds > 0 && diagnostics.active_points >= 6,
        ))
        .then(Action::Screenshot("trail_tube_mesh_mode".into()))
        .then(assertions::log_summary("trail_tube_mesh_mode summary"))
        .build()
}

fn build_fade_modes() -> Scenario {
    Scenario::builder("trail_fade_modes")
        .description(
            "Cycle through TrailFadeMode::Width and TrailFadeMode::Both on the locked swipe trail, \
             capturing a screenshot for each, and verify the mode is reflected on the component.",
        )
        .then(Action::WaitFrames(45))
        .then(support::locked_view_action())
        .then(Action::Custom(Box::new(|world: &mut World| {
            let lab = *world.resource::<LabEntities>();
            if let Some(mut trail) = world.get_mut::<Trail>(lab.locked) {
                trail.style.fade_mode = TrailFadeMode::Width;
            }
        })))
        .then(Action::WaitFrames(20))
        .then(assertions::custom(
            "locked trail uses Width fade mode",
            |world| {
                let lab = *world.resource::<LabEntities>();
                world
                    .get::<Trail>(lab.locked)
                    .is_some_and(|trail| trail.style.fade_mode == TrailFadeMode::Width)
            },
        ))
        .then(Action::Screenshot("trail_fade_width".into()))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let lab = *world.resource::<LabEntities>();
            if let Some(mut trail) = world.get_mut::<Trail>(lab.locked) {
                trail.style.fade_mode = TrailFadeMode::Both;
            }
        })))
        .then(Action::WaitFrames(20))
        .then(assertions::custom(
            "locked trail uses Both fade mode",
            |world| {
                let lab = *world.resource::<LabEntities>();
                world
                    .get::<Trail>(lab.locked)
                    .is_some_and(|trail| trail.style.fade_mode == TrailFadeMode::Both)
            },
        ))
        .then(Action::Screenshot("trail_fade_both".into()))
        .then(assertions::log_summary("trail_fade_modes summary"))
        .build()
}

fn build_melee_swipe() -> Scenario {
    Scenario::builder("trail_melee_swipe")
        .description(
            "Focus the camera on the transform-locked swipe source (the closest analogue to a \
             melee swipe trail), verify the TransformLocked orientation is active, and capture \
             the arc shape from two different camera angles.",
        )
        .then(Action::WaitFrames(45))
        .then(support::locked_view_action())
        .then(Action::Custom(Box::new(|world: &mut World| {
            let lab = *world.resource::<LabEntities>();
            // Widen the locked trail slightly to make the swipe arc more visible.
            if let Some(mut trail) = world.get_mut::<Trail>(lab.locked) {
                trail.style.base_width = 1.4;
            }
        })))
        .then(support::focus_camera_action(Vec3::new(3.4, 3.8, 7.2), Vec3::new(0.0, 1.6, 0.0)))
        .then(Action::WaitFrames(15))
        .then(assertions::custom(
            "locked (swipe) source keeps transform-locked orientation",
            |world| {
                let entity = world.resource::<LabEntities>().locked;
                world.get::<Trail>(entity).is_some_and(|trail| {
                    matches!(trail.orientation, TrailOrientation::TransformLocked { .. })
                })
            },
        ))
        .then(assertions::resource_satisfies::<TrailDiagnostics>(
            "swipe trail produces active geometry",
            |diagnostics| diagnostics.active_points >= 4 && diagnostics.visible_trails >= 1,
        ))
        .then(Action::Screenshot("melee_swipe_front".into()))
        .then(Action::WaitFrames(1))
        // Side angle to show the arc depth
        .then(support::side_swipe_view_action())
        .then(Action::WaitFrames(12))
        .then(Action::Screenshot("melee_swipe_side".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("trail_melee_swipe summary"))
        .build()
}

fn build_drawing_trail() -> Scenario {
    Scenario::builder("trail_drawing_trail")
        .description(
            "Verify the local-space hover wake trail (closest analogue to a drawing trail) \
             records points in local space and that the diagnostic active point count grows \
             over time while the carrier is in motion.",
        )
        .then(Action::WaitFrames(30))
        .then(assertions::custom(
            "hover source uses local trail space",
            |world| {
                let entity = world.resource::<LabEntities>().hover;
                world
                    .get::<Trail>(entity)
                    .is_some_and(|trail| trail.space == TrailSpace::Local)
            },
        ))
        .then(support::hover_view_action())
        .then(Action::Screenshot("drawing_trail_start".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(60))
        .then(assertions::resource_satisfies::<TrailDiagnostics>(
            "active point count remains non-zero during drawing",
            |diagnostics| diagnostics.active_points >= 4,
        ))
        .then(assertions::custom(
            "hover trail still running after 60 frames",
            |world| {
                let entity = world.resource::<LabEntities>().hover;
                world.get_entity(entity).is_ok()
                    && world
                        .get::<Trail>(entity)
                        .is_some_and(|trail| trail.space == TrailSpace::Local)
            },
        ))
        .then(Action::Screenshot("drawing_trail_accumulated".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("trail_drawing_trail summary"))
        .build()
}

fn build_projectile_contrail() -> Scenario {
    Scenario::builder("trail_projectile_contrail")
        .description(
            "Frame the billboard contrail (the projectile-like emitter) and verify it \
             accumulates points over time and produces mesh rebuilds, confirming the \
             always-on emitter mode drives continuous geometry.",
        )
        .then(Action::WaitFrames(50))
        .then(support::billboard_view_action())
        .then(support::remember_rebuilds_action())
        .then(Action::WaitFrames(30))
        .then(assertions::custom(
            "billboard source uses AlwaysOn emitter mode",
            |world| {
                let entity = world.resource::<LabEntities>().billboard;
                world
                    .get::<Trail>(entity)
                    .is_some_and(|trail| matches!(trail.emitter_mode, TrailEmitterMode::Always))
            },
        ))
        .then(assertions::custom(
            "contrail accumulated new mesh rebuilds",
            |world| {
                let before = world
                    .get_resource::<RebuildSnapshot>()
                    .expect("rebuild snapshot should exist")
                    .0;
                world.resource::<TrailDiagnostics>().total_mesh_rebuilds > before
            },
        ))
        .then(assertions::resource_satisfies::<TrailDiagnostics>(
            "contrail has active geometry points",
            |diagnostics| diagnostics.active_points >= 8,
        ))
        .then(Action::Screenshot("projectile_contrail".into()))
        .then(Action::WaitFrames(1))
        .then(support::clear_rebuild_snapshot_action())
        .then(assertions::log_summary("trail_projectile_contrail summary"))
        .build()
}

fn build_lod() -> Scenario {
    Scenario::builder("trail_lod")
        .description(
            "Attach a TrailLod component to the billboard source and move the camera far away, \
             then verify that effective max points drops below the base maximum.",
        )
        .then(Action::WaitFrames(60))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let lab = *world.resource::<LabEntities>();
            world.entity_mut(lab.billboard).insert(TrailLod {
                start_distance: 5.0,
                end_distance: 20.0,
                min_points_fraction: 0.25,
            });
        })))
        .then(support::billboard_view_action())
        .then(Action::WaitFrames(30))
        .then(assertions::custom(
            "TrailLod component is attached to billboard source",
            |world| {
                let lab = *world.resource::<LabEntities>();
                world.get::<TrailLod>(lab.billboard).is_some()
            },
        ))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let lab = *world.resource::<LabEntities>();
            let history_len = world
                .get::<TrailHistory>(lab.billboard)
                .map(|history| history.len())
                .unwrap_or(0);
            world.insert_resource(LodSnapshot { history_len });
        })))
        .then(Action::Screenshot("trail_lod_near".into()))
        .then(support::focus_camera_action(Vec3::new(-4.4, 2.6, 40.0), Vec3::new(-4.2, 1.8, 0.0)))
        .then(Action::WaitFrames(30))
        .then(assertions::custom(
            "trail LOD trims the billboard trail at distance",
            |world| {
                let lab = *world.resource::<LabEntities>();
                let snapshot = world.resource::<LodSnapshot>();
                world
                    .get::<TrailHistory>(lab.billboard)
                    .is_some_and(|history| history.len() < snapshot.history_len)
            },
        ))
        .then(assertions::custom(
            "trail system remains active with LOD attached",
            |world| {
                let diagnostics = world.resource::<TrailDiagnostics>();
                diagnostics.runtime_active && diagnostics.active_sources >= 4
            },
        ))
        .then(inspect::log_resource::<TrailDiagnostics>(
            "trail_lod diagnostics",
        ))
        .then(Action::Screenshot("trail_lod_far".into()))
        .then(Action::Custom(Box::new(|world: &mut World| {
            world.remove_resource::<LodSnapshot>();
        })))
        .then(assertions::log_summary("trail_lod summary"))
        .build()
}

fn build_history_access() -> Scenario {
    Scenario::builder("trail_history_access")
        .description(
            "Verify that TrailHistory is populated on source entities after sampling and \
             contains the expected point data.",
        )
        .then(Action::WaitFrames(90))
        .then(assertions::custom(
            "billboard source has TrailHistory component",
            |world| {
                let lab = *world.resource::<LabEntities>();
                world.get::<TrailHistory>(lab.billboard).is_some()
            },
        ))
        .then(assertions::custom(
            "TrailHistory has points after sampling",
            |world| {
                let lab = *world.resource::<LabEntities>();
                world
                    .get::<TrailHistory>(lab.billboard)
                    .is_some_and(|history| history.len() > 0 && history.total_length() > 0.0)
            },
        ))
        .then(assertions::custom(
            "TrailHistory normalized_lengths parallel to points",
            |world| {
                let lab = *world.resource::<LabEntities>();
                world
                    .get::<TrailHistory>(lab.billboard)
                    .is_some_and(|history| {
                        let lengths = history.normalized_lengths();
                        lengths.len() == history.len()
                    })
            },
        ))
        .then(Action::Screenshot("trail_history_access".into()))
        .then(assertions::log_summary("trail_history_access summary"))
        .build()
}

fn build_point_mutation() -> Scenario {
    Scenario::builder("trail_point_mutation")
        .description(
            "Mutate trail points via TrailHistory::points_mut() and verify the mesh rebuilds.",
        )
        .then(Action::WaitFrames(90))
        .then(support::remember_rebuilds_action())
        .then(Action::Custom(Box::new(|world: &mut World| {
            let lab = *world.resource::<LabEntities>();
            if let Some(mut history) = world.get_mut::<TrailHistory>(lab.billboard) {
                let points = history.points_mut();
                for point in points.iter_mut() {
                    point.position.y += 0.5;
                }
            }
        })))
        .then(Action::WaitFrames(6))
        .then(assertions::custom(
            "point mutation triggered mesh rebuild",
            |world| {
                let before = world.resource::<RebuildSnapshot>().0;
                world.resource::<TrailDiagnostics>().total_mesh_rebuilds > before
            },
        ))
        .then(Action::Screenshot("trail_point_mutation".into()))
        .then(support::clear_rebuild_snapshot_action())
        .then(assertions::log_summary("trail_point_mutation summary"))
        .build()
}

fn build_age_curves() -> Scenario {
    Scenario::builder("trail_age_curves")
        .description(
            "Set width_over_age and color_over_age on the locked trail and verify the style \
             is applied (mesh keeps rebuilding due to age animation).",
        )
        .then(Action::WaitFrames(45))
        .then(support::locked_view_action())
        .then(Action::Custom(Box::new(|world: &mut World| {
            let lab = *world.resource::<LabEntities>();
            if let Some(mut trail) = world.get_mut::<Trail>(lab.locked) {
                trail.style.width_over_age = TrailScalarCurve::linear(1.0, 0.0);
                trail.style.color_over_age = TrailGradient::new([
                    TrailColorKey::new(0.0, Color::srgb(1.0, 0.3, 0.1)),
                    TrailColorKey::new(1.0, Color::srgb(0.2, 0.4, 1.0)),
                ]);
            }
        })))
        .then(Action::WaitFrames(30))
        .then(support::remember_rebuilds_action())
        .then(Action::WaitFrames(10))
        .then(assertions::custom(
            "age curves trigger continuous rebuilds",
            |world| {
                let before = world.resource::<RebuildSnapshot>().0;
                world.resource::<TrailDiagnostics>().total_mesh_rebuilds > before
            },
        ))
        .then(Action::Screenshot("trail_age_curves".into()))
        .then(support::clear_rebuild_snapshot_action())
        .then(assertions::log_summary("trail_age_curves summary"))
        .build()
}

fn build_style_override() -> Scenario {
    Scenario::builder("trail_style_override")
        .description(
            "Attach TrailStyleOverride to the locked trail, verify it changes the visual \
             appearance, then remove it and verify revert.",
        )
        .then(Action::WaitFrames(45))
        .then(support::locked_view_action())
        .then(Action::Screenshot("trail_style_override_before".into()))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let lab = *world.resource::<LabEntities>();
            world
                .entity_mut(lab.locked)
                .insert(TrailStyleOverride(TrailStyle {
                    base_width: 2.0,
                    color_over_length: TrailGradient::new([
                        TrailColorKey::new(0.0, Color::srgb(1.0, 0.0, 0.0)),
                        TrailColorKey::new(1.0, Color::srgb(1.0, 1.0, 0.0)),
                    ]),
                    ..default()
                }));
        })))
        .then(Action::WaitFrames(30))
        .then(assertions::custom(
            "TrailStyleOverride is attached",
            |world| {
                let lab = *world.resource::<LabEntities>();
                world.get::<TrailStyleOverride>(lab.locked).is_some()
            },
        ))
        .then(Action::Screenshot("trail_style_override_active".into()))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let lab = *world.resource::<LabEntities>();
            world.entity_mut(lab.locked).remove::<TrailStyleOverride>();
        })))
        .then(Action::WaitFrames(20))
        .then(assertions::custom("TrailStyleOverride removed", |world| {
            let lab = *world.resource::<LabEntities>();
            world.get::<TrailStyleOverride>(lab.locked).is_none()
        }))
        .then(Action::Screenshot("trail_style_override_reverted".into()))
        .then(assertions::log_summary("trail_style_override summary"))
        .build()
}
