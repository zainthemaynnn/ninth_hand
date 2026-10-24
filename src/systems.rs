use std::time::Duration;

use bevy::{
    ecs::query::QueryFilter, gltf::GltfMaterialName, prelude::*, state::state::FreelyMutableState,
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

pub fn despawn_tween_entity(trigger: On<AnimCompletedEvent>, mut commands: Commands) {
    commands.entity(trigger.anim_entity).despawn();
}

pub fn find_named_node(
    e_scene: Entity,
    name: &str,
    children_query: &Query<&Children>,
    name_query: &Query<&Name>,
) -> Option<Entity> {
    children_query
        .iter_descendants(e_scene)
        .find(move |e_node| {
            name_query
                .get(*e_node)
                .is_ok_and(|node_name| node_name.as_str() == name)
        })
}

pub fn find_named_material(
    e_scene: Entity,
    name: &str,
    children_query: &Query<&Children>,
    name_query: &Query<&GltfMaterialName>,
) -> Option<Entity> {
    children_query
        .iter_descendants(e_scene)
        .find(move |e_node| {
            name_query
                .get(*e_node)
                .is_ok_and(|node_name| node_name.0.as_str() == name)
        })
}
