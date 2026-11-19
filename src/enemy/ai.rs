use bevy::prelude::*;
use rand::seq::IteratorRandom;

use crate::{
    actions::action::{Action, AvailableAction, OwnsAction, Played, QueuedAction, SelectedAction},
    turn::{Cpu, TurnState},
};

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(TurnState::EnemySelect), queue_random_action)
            .add_systems(OnEnter(TurnState::CpuStart), play_actions);
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

        let Some(e_action) = actions.iter().next() else {
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

#[derive(Component)]
pub struct RandomTargetAi;

pub fn target_random_unit(
    trigger: On<Add, QueuedAction>,
    mut commands: Commands,
    unit_query: Query<&OwnsAction, With<RandomTargetAi>>,
    available_query: Query<(), With<AvailableAction>>,
    action_query: Query<&Action>,
) {
    let mut rng = rand::rng();
    for owns_actions in unit_query {
        let actions = owns_actions
            .iter()
            .filter(|e_action| available_query.contains(*e_action))
            .sample(&mut rng, 1);

        let Some(e_action) = actions.iter().next() else {
            continue;
        };

        info!(
            "selected action {:?}",
            action_query.get(*e_action).unwrap().name
        );

        commands.entity(*e_action).insert(SelectedAction);
    }
}
