use bevy::prelude::*;
use rand::seq::IteratorRandom;

use super::action::{Action, AvailableAction, OwnsAction};
#[derive(Message)]
pub struct DrawEvent {
    pub amount: usize,
    pub unit: Entity,
}

#[derive(Component, Debug)]
pub struct DrawCount {
    pub amount: usize,
}

pub fn shuffle_selected_actions(
    mut commands: Commands,
    mut draw_events: MessageReader<DrawEvent>,
    collection_query: Query<&OwnsAction>,
    action_query: Query<&Action>,
) {
    let mut rng = rand::rng();
    for DrawEvent {
        amount,
        unit: e_unit,
    } in draw_events.read()
    {
        let Ok(owns_action) = collection_query.get(*e_unit) else {
            continue;
        };

        let selected_actions = owns_action.iter().sample(&mut rng, *amount);

        for e_action in selected_actions {
            info!(
                "shuffled action {:?}",
                action_query.get(e_action).unwrap().name
            );
            commands.entity(e_action).insert(AvailableAction);
        }
    }
}
