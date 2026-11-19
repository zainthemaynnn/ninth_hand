use bevy::prelude::*;

use crate::{
    actions::{
        action::{Action, OwnedByUnit, Played, SelectedAction},
        tags::{CellTarget, FrontTarget, HasSecondaryTarget},
    },
    field::{Field, HexCoord, HighlightedCell, Highlighter, OccupyingCell},
    turn::{Cpu, Rival, TurnState},
    unit::Unit,
};
#[derive(Component, Debug, Default)]
#[component(storage = "SparseSet")]
#[require(HighlightedCell { ty: Highlighter::Red })]
pub struct Targettable;

#[derive(Component, Debug, Default)]
#[component(storage = "SparseSet")]
#[require(HighlightedCell { ty: Highlighter::Yellow })]
pub struct SecondTargettable;

#[derive(Component, Debug, Default)]
#[component(storage = "SparseSet")]
#[require(HighlightedCell { ty: Highlighter::Blue })]
pub struct ActiveUnitCell;

#[derive(Component, Debug, Default)]
#[component(storage = "SparseSet")]
pub struct Target;

#[derive(Component, Debug, Default)]
#[component(storage = "SparseSet")]
pub struct SecondaryTarget;

#[derive(Component, Debug, Default)]
#[component(storage = "SparseSet")]
#[require(HighlightedCell { ty: Highlighter::Gray })]
pub struct Untargettable;

fn front_target_candidates(
    enemy_query: &Query<&OccupyingCell, (With<Unit>, With<Rival>)>,
    cell_query: &Query<(Entity, &HexCoord)>,
) -> Vec<Entity> {
    let enemy_positions = enemy_query
        .iter()
        .filter_map(|OccupyingCell(e_cell)| {
            cell_query
                .get(*e_cell)
                .ok()
                .map(|(e_cell, coord)| (e_cell, coord.row()))
        })
        .collect::<Vec<_>>();

    let Some(front_row) = enemy_positions.iter().map(|(_, row)| *row).max() else {
        return Vec::new();
    };

    enemy_positions
        .into_iter()
        .filter_map(|(e_cell, row)| (row == front_row).then_some(e_cell))
        .collect()
}

fn cell_target_candidates(field: &Field<Cpu>) -> Vec<Entity> {
    field.cells.values().copied().collect()
}

fn secondary_target_candidates(
    e_target: Entity,
    field: &Field<Cpu>,
    target_query: &Query<&HexCoord>,
    action_query: &Query<(), (With<SelectedAction>, With<HasSecondaryTarget>)>,
) -> Vec<Entity> {
    if action_query.single().is_err() {
        return Vec::new();
    }

    let Ok(coord) = target_query.get(e_target) else {
        return Vec::new();
    };

    field.all_adjacent(*coord, HexCoord::DIRECTIONS.to_vec())
}

pub fn active_unit_target(
    trigger: On<Add, SelectedAction>,
    mut commands: Commands,
    action_query: Query<&OwnedByUnit>,
    unit_query: Query<&OccupyingCell>,
) {
    let Ok(OwnedByUnit(e_unit)) = action_query.get(trigger.entity) else {
        warn!("action has no owner");
        return;
    };

    let Ok(OccupyingCell(e_cell)) = unit_query.get(*e_unit) else {
        warn!("action owner not occupying cell");
        return;
    };

    commands.entity(*e_cell).insert(ActiveUnitCell);
    info!("highlighted selected action owner");
}

pub fn deselect_action_cells_on_deselect(
    _trigger: On<Remove, SelectedAction>,
    mut commands: Commands,
    highlighted_query: Query<Entity, With<HighlightedCell>>,
) {
    for e_cell in highlighted_query {
        commands
            .entity(e_cell)
            .remove::<(HighlightedCell, Targettable)>();
    }
}

pub fn front_target(
    trigger: On<Add, SelectedAction>,
    mut commands: Commands,
    target_query: Query<(), With<FrontTarget>>,
    enemy_query: Query<&OccupyingCell, (With<Unit>, With<Rival>)>,
    cell_query: Query<(Entity, &HexCoord)>,
) {
    if !target_query.contains(trigger.entity) {
        return;
    }

    for e_cell in front_target_candidates(&enemy_query, &cell_query) {
        commands.entity(e_cell).insert(Targettable);
    }
}

pub fn cell_target(
    trigger: On<Add, SelectedAction>,
    mut commands: Commands,
    target_query: Query<(), With<CellTarget>>,
    field: Res<Field<Cpu>>,
) {
    if !target_query.contains(trigger.entity) {
        return;
    }

    for e_cell in cell_target_candidates(&field) {
        commands.entity(e_cell).insert(Targettable);
    }
}

// TODO: depending on the target, need to pick the right field
pub fn secondary_target(
    trigger: On<Add, Target>,
    mut commands: Commands,
    field: Res<Field<Cpu>>,
    target_query: Query<&HexCoord>,
    action_query: Query<(), (With<SelectedAction>, With<HasSecondaryTarget>)>,
) {
    for e_adj in secondary_target_candidates(trigger.entity, &field, &target_query, &action_query) {
        commands.entity(e_adj).insert(SecondTargettable);
    }
}

pub fn add_target_on_click(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    target_query: Query<(), With<Targettable>>,
) {
    let Ok(()) = target_query.get(trigger.entity) else {
        return;
    };

    commands.entity(trigger.entity).insert(Target);
}

pub fn add_secondary_target_on_click(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    target_query: Query<(), With<SecondTargettable>>,
) {
    let Ok(()) = target_query.get(trigger.entity) else {
        return;
    };

    commands.entity(trigger.entity).insert(SecondaryTarget);
}

pub fn play_primary_target_action(
    _trigger: On<Add, Target>,
    mut commands: Commands,
    action_query: Single<(Entity, &Action), (With<SelectedAction>, Without<HasSecondaryTarget>)>,
    mut next_state: ResMut<NextState<TurnState>>,
) {
    let (e_action, action) = action_query.into_inner();

    info!("playing primary target action {:?}", action.name);

    commands.trigger(Played { entity: e_action });
    next_state.set(TurnState::PlayerBusy);
}

pub fn play_secondary_target_action(
    _trigger: On<Add, SecondaryTarget>,
    mut commands: Commands,
    action_query: Single<(Entity, &Action), (With<SelectedAction>, With<HasSecondaryTarget>)>,
    mut next_state: ResMut<NextState<TurnState>>,
) {
    let (e_action, action) = action_query.into_inner();

    info!("playing secondary target action {:?}", action.name);

    commands.trigger(Played { entity: e_action });
    next_state.set(TurnState::PlayerBusy);
}
