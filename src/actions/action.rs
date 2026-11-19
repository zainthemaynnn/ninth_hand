use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::{asset::GameState, turn::TurnState};

use super::{card, draw, target};
pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<draw::DrawEvent>()
            .add_observer(card::add_default_card_art)
            .add_systems(OnEnter(GameState::InGame), card::set_action_bar)
            .add_systems(
                Update,
                (
                    target::update_front_can_target,
                    target::update_cell_can_target,
                    target::update_secondary_can_target,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    card::update_action_bar,
                    card::select_deselect_card,
                    card::deselect_action_on_card_deselect.after(card::select_deselect_card),
                    card::select_action_on_card_select
                        .after(card::deselect_action_on_card_deselect)
                        .after(target::update_secondary_can_target),
                    discard_deactivated_actions,
                )
                    .run_if(
                        in_state(TurnState::Player)
                            .or_else(in_state(TurnState::PlayerStart))
                            .or_else(in_state(TurnState::PlayerDone)),
                    ),
            )
            .add_observer(card::select_card_animation)
            .add_observer(card::deselect_card_animation)
            .add_observer(target::deselect_action_cells_on_deselect)
            .add_observer(target::active_unit_target)
            .add_observer(target::front_target)
            .add_observer(target::cell_target)
            .add_observer(target::add_target_on_click)
            .add_observer(target::add_secondary_target_on_click)
            .add_observer(target::play_primary_target_action)
            .add_observer(target::play_secondary_target_action)
            .add_observer(deselect_played_action)
            .add_observer(card::discard_card_animation)
            .configure_loading_state(
                LoadingStateConfig::new(GameState::AssetLoading)
                    .load_collection::<card::CardAssets>(),
            );
    }
}
#[derive(Component, Default)]
pub struct Action {
    pub name: &'static str,
    pub description: &'static str,
}

pub trait ActionType {
    fn play() -> Observer;
}

/// The `Unit` owning this action.
#[derive(Component, Debug)]
#[relationship(relationship_target = OwnsAction)]
pub struct OwnedByUnit(pub Entity);

/// The `Action`s owned by this unit.
#[derive(Component, Debug)]
#[relationship_target(relationship = OwnedByUnit)]
pub struct OwnsAction(Vec<Entity>);
#[derive(Component)]
pub struct AvailableAction;

#[derive(Component, Debug, Default)]
#[component(storage = "SparseSet")]
pub struct SelectedAction;

#[derive(Component)]
pub struct QueuedAction;

#[derive(Component, Debug, Default)]
#[component(storage = "SparseSet")]
pub struct InPlay;
#[derive(EntityEvent)]
pub struct Played {
    pub entity: Entity,
}

pub fn deselect_played_action(
    trigger: On<Played>,
    mut commands: Commands,
    action_query: Query<&super::card::OwnsCard>,
) {
    commands
        .entity(trigger.entity)
        .remove::<(SelectedAction, AvailableAction)>();
    let owns_card = action_query.get(trigger.entity).unwrap();
    commands
        .entity(owns_card.entity())
        .insert(super::card::Discarded);
}
pub fn discard_deactivated_actions(
    mut commands: Commands,
    mut discarded: RemovedComponents<AvailableAction>,
    action_query: Query<&super::card::OwnsCard>,
) {
    for e_action in discarded.read() {
        if let Ok(owns_card) = action_query.get(e_action) {
            commands
                .entity(owns_card.entity())
                .insert(super::card::Discarded);
        }
    }
}
