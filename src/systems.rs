use std::time::Duration;

use bevy::{
    ecs::query::QueryFilter, prelude::*, state::state::FreelyMutableState,
    world_serialization::WorldInstanceReady,
};
use bevy_tweening::AnimCompletedEvent;

pub fn enforce_one<T: Component>(
    trigger: On<Add, T>,
    mut commands: Commands,
    t_query: Query<Entity, With<T>>,
) {
    for e_old in t_query {
        if e_old == trigger.entity {
            continue;
        }

        commands.entity(e_old).remove::<T>();
    }
}

pub fn toggle_insert_on_click<T: Component + Default, F: QueryFilter>(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    t_query: Query<Option<&T>, F>,
) {
    if let Ok(component) = t_query.get(trigger.entity) {
        if component.is_none() {
            commands.entity(trigger.entity).insert(T::default());
        } else {
            commands.entity(trigger.entity).remove::<T>();
        }
    }
}

pub fn insert_on_hover<T: Component + Default, F: QueryFilter>(
    trigger: On<Pointer<Over>>,
    mut commands: Commands,
    t_query: Query<Option<&T>, F>,
) {
    if let Ok(component) = t_query.get(trigger.entity) {
        if component.is_none() {
            commands.entity(trigger.entity).insert(T::default());
        }
    }
}

pub fn uninsert_on_unhover<T: Component + Default, F: QueryFilter>(
    trigger: On<Pointer<Out>>,
    mut commands: Commands,
    t_query: Query<(), F>,
) {
    if let Ok(()) = t_query.get(trigger.entity) {
        commands.entity(trigger.entity).remove::<T>();
    }
}

/// Moves to the input state after a timer.
pub fn countdown_to_state<const T: u64, S: FreelyMutableState>(
    In(state): In<S>,
    time: Res<Time>,
    mut my_timer: Local<Timer>,
    mut initialized: Local<bool>,
    mut next_state: ResMut<NextState<S>>,
) {
    if !*initialized {
        *my_timer = Timer::new(Duration::from_millis(T), TimerMode::Once);
        *initialized = true;
    }

    my_timer.tick(time.delta());

    if my_timer.just_finished() {
        *initialized = false;
        next_state.set(state);
    }
}

pub fn clear_all<T: Component>(mut commands: Commands, t_query: Query<Entity, With<T>>) {
    for e_t in t_query {
        commands.entity(e_t).remove::<T>();
    }
}

pub fn log_added_component<T: Component>(trigger: On<Add, T>, mut commands: Commands) {
    commands.entity(trigger.entity).log_components();
}

/// Stores the first descendant with an [`AnimationPlayer`] when a world asset
/// instance has finished spawning. Attach this observer to the `WorldAssetRoot`
/// entity; consumers can then read [`AnimatorEntity`] from that root.
pub fn register_animator(
    scene_ready: On<WorldInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    animator_query: Query<(), With<AnimationPlayer>>,
) {
    let scene_root = scene_ready.entity;
    let Some(animator) = children
        .iter_descendants(scene_root)
        .find(|&entity| animator_query.contains(entity))
    else {
        warn!("no AnimationPlayer descendant found under {scene_root:?}");
        return;
    };

    commands.entity(scene_root).insert(AnimatorEntity(animator));
    commands
        .entity(animator)
        .insert(AnimationTransitions::new());
}

/// The first descendant `AnimationPlayer` found for a spawned world asset.
#[derive(Component, Copy, Clone, Debug)]
pub struct AnimatorEntity(pub Entity);

pub fn despawn_tween_entity(trigger: On<AnimCompletedEvent>, mut commands: Commands) {
    commands.entity(trigger.anim_entity).despawn();
}
