use bevy::prelude::*;
use rand::seq::IteratorRandom;

use crate::{
    actions::{
        action::{Action, AvailableAction, OwnedByUnit, OwnsAction, Played, QueuedAction},
        target::CanTarget,
    },
    director::Director,
    field::OccupyingCell,
    turn::{Cpu, TurnState},
};

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(TurnState::EnemySelect), queue_random_action)
            .add_systems(OnEnter(TurnState::CpuStart), play_actions)
            .add_observer(target_random_unit);
    }
}

#[derive(Component)]
pub struct RandomAi;

pub fn queue_random_action(
    mut commands: Commands,
    unit_query: Query<&OwnsAction, With<RandomAi>>,
    available_query: Query<(), With<AvailableAction>>,
    action_query: Query<&Action>,
) {
    let mut rng = rand::rng();
    for owns_actions in unit_query {
        let actions = owns_actions
            .iter()
            .filter(|e_action| available_query.contains(*e_action))
            .sample(&mut rng, 1);

        let Some(e_action) = actions.first() else {
            continue;
        };

        info!(
            "selected action {:?}",
            action_query.get(*e_action).unwrap().name
        );

        commands.entity(*e_action).insert(QueuedAction);
    }
}

pub fn play_actions(
    mut commands: Commands,
    unit_query: Query<&OwnsAction, With<Cpu>>,
    selected_query: Query<(), With<QueuedAction>>,
) {
    for owns_actions in unit_query {
        let actions = owns_actions
            .iter()
            .filter(|e_action| selected_query.contains(*e_action));

        for e_action in actions {
            commands.trigger(Played { entity: e_action });
        }
    }
}

#[derive(Component, Debug, Default)]
pub struct PlannedTarget;

#[derive(Component)]
pub struct RandomTargetAi;

pub fn target_random_unit(
    trigger: On<Add, QueuedAction>,
    mut commands: Commands,
    action_query: Query<(&Action, &CanTarget, &OwnedByUnit)>,
    unit_cell_query: Query<&OccupyingCell>,
    global_transform_query: Query<&GlobalTransform>,
) {
    let Ok((action, can_target, OwnedByUnit(e_unit))) = action_query.get(trigger.entity) else {
        return;
    };

    debug!("selecting random target for {:?}", action.name);

    let mut rng = rand::rng();
    let Some(e_target) = can_target.cells.iter().copied().choose(&mut rng) else {
        return;
    };

    let Ok(OccupyingCell(e_owner_cell)) = unit_cell_query.get(*e_unit) else {
        warn!("action owner is not occupying a cell");
        return;
    };
    let Ok(owner_transform) = global_transform_query.get(*e_owner_cell) else {
        warn!("action owner's cell has no global transform");
        return;
    };
    let Ok(target_transform) = global_transform_query.get(e_target) else {
        warn!("target cell has no global transform");
        return;
    };

    let start = owner_transform.translation() + Vec3::Y * 0.1;
    let end = target_transform.translation() + Vec3::Y * 0.1;
    let arc_height = (end - start).length() * 0.2 + 0.5;
    let curve = CubicBezier::new([[
        start,
        start.lerp(end, 1.0 / 3.0) + Vec3::Y * arc_height,
        start.lerp(end, 2.0 / 3.0) + Vec3::Y * arc_height,
        end,
    ]])
    .to_curve()
    .unwrap();

    commands.spawn(Director {
        curve,
        width: 0.2,
        tiling_distance: 1.0,
    });
    commands.entity(e_target).insert(PlannedTarget);
}
