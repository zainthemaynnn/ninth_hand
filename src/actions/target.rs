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

#[derive(Component, Debug, Default)]
pub struct CanTarget {
    pub cells: Vec<Entity>,
}

pub fn update_front_can_target(
    mut action_query: Query<&mut CanTarget, With<FrontTarget>>,
    enemy_query: Query<&OccupyingCell, (With<Unit>, With<Rival>)>,
    cell_query: Query<(Entity, &HexCoord)>,
) {
    let enemy_positions = enemy_query
        .iter()
        .filter_map(|OccupyingCell(e_cell)| {
            cell_query
                .get(*e_cell)
                .ok()
                .map(|(e_cell, coord)| (e_cell, coord.row()))
        })
        .collect::<Vec<_>>();

    let front_cells = if let Some(front_row) = enemy_positions.iter().map(|(_, row)| *row).max() {
        enemy_positions
            .into_iter()
            .filter_map(|(e_cell, row)| (row == front_row).then_some(e_cell))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    for mut can_target in &mut action_query {
        can_target.cells.clone_from(&front_cells);
    }
}

pub fn update_cell_can_target(
    mut action_query: Query<&mut CanTarget, With<CellTarget>>,
    field: Option<Res<Field<Cpu>>>,
) {
    let cells = field
        .map(|field| field.cells.values().copied().collect::<Vec<_>>())
        .unwrap_or_default();

    for mut can_target in &mut action_query {
        can_target.cells.clone_from(&cells);
    }
}

pub fn update_secondary_can_target(
    mut commands: Commands,
    mut action_query: Query<&mut CanTarget, (With<SelectedAction>, With<HasSecondaryTarget>)>,
    field: Option<Res<Field<Cpu>>>,
    target_query: Query<&HexCoord, With<Target>>,
) {
    let cells = field.and_then(|field| {
        target_query
            .single()
            .ok()
            .map(|coord| field.all_adjacent(*coord, HexCoord::DIRECTIONS.to_vec()))
    });

    for mut can_target in &mut action_query {
        if let Some(cells) = &cells {
            for e_old in can_target
                .cells
                .iter()
                .filter(|e_old| !cells.contains(e_old))
            {
                commands.entity(*e_old).remove::<SecondTargettable>();
            }
            can_target.cells.clone_from(cells);
            for &e_cell in cells {
                commands.entity(e_cell).insert(SecondTargettable);
            }
        } else {
            for e_old in can_target.cells.drain(..) {
                commands.entity(e_old).remove::<SecondTargettable>();
            }
        }
    }
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
            .remove::<(HighlightedCell, Targettable, SecondTargettable)>();
    }
}

pub fn front_target(
    trigger: On<Add, SelectedAction>,
    mut commands: Commands,
    tag_query: Query<(), With<FrontTarget>>,
    can_target_query: Query<&CanTarget>,
) {
    if !tag_query.contains(trigger.entity) {
        return;
    }

    let Ok(can_target) = can_target_query.get(trigger.entity) else {
        return;
    };

    for &e_cell in &can_target.cells {
        commands.entity(e_cell).insert(Targettable);
    }
}

pub fn cell_target(
    trigger: On<Add, SelectedAction>,
    mut commands: Commands,
    tag_query: Query<(), With<CellTarget>>,
    can_target_query: Query<&CanTarget>,
) {
    if !tag_query.contains(trigger.entity) {
        return;
    }

    let Ok(can_target) = can_target_query.get(trigger.entity) else {
        return;
    };

    for &e_cell in &can_target.cells {
        commands.entity(e_cell).insert(Targettable);
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
