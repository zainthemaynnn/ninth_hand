use std::time::Duration;

use bevy::{platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_tweening::{
    PlaybackDirection, PlaybackState, Tween, TweenAnim, Tweenable, lens::UiPositionLens,
};
use rand::seq::IteratorRandom;

use crate::{
    actions::tags::{FrontTarget, HasSecondaryTarget},
    asset::GameState,
    field::{Field, HexCoord, HighlightedCell, Highlighter, OccupyingCell},
    turn::{Player, Rival},
    unit::Unit,
};

const CARD_STICKOUT: Val = Val::Percent(50.0);

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DrawEvent>()
            .add_systems(OnEnter(GameState::InGame), set_action_bar)
            .add_systems(
                Update,
                (
                    update_action_bar,
                    select_deselect_card,
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_observer(select_card_animation)
            .add_observer(deselect_card_animation)
            .add_observer(select_action_on_card_select)
            .add_observer(deselect_action_on_card_deselect)
            .add_observer(deselect_action_cells_on_deselect)
            .add_observer(highlight_action_owner_cell)
            .add_observer(highlight_front_target_cells)
            .add_observer(add_target_on_click)
            .add_observer(play_primary_target_action)
            .add_observer(highlight_secondary_targets)
            .add_observer(play_secondary_target_action)
            .configure_loading_state(
                LoadingStateConfig::new(GameState::AssetLoading).load_collection::<CardAssets>(),
            );
    }
}

#[derive(Component, Default)]
pub struct Action {
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Message)]
pub struct DrawEvent {
    pub amount: usize,
    pub unit: Entity,
}

#[derive(Component)]
pub struct DrawCount {
    pub amount: usize,
}

pub fn draw_actions(
    mut commands: Commands,
    mut draw_events: MessageReader<DrawEvent>,
    collection_query: Query<&OwnsAction>,
    available_query: Query<(), With<AvailableAction>>,
) {
    let mut rng = rand::rng();
    for DrawEvent {
        amount,
        unit: e_unit,
    } in draw_events.read()
    {
        let owns_action = collection_query.get(*e_unit).unwrap();

        let selected_actions = owns_action
            .iter()
            .filter(|e_action| !available_query.contains(*e_action))
            .sample(&mut rng, *amount);

        for e_action in selected_actions {
            commands.entity(e_action).insert(AvailableAction);
        }
    }
}

#[derive(Component)]
pub struct AvailableAction;

pub mod tags {
    use bevy::prelude::*;

    #[derive(Component, Default)]
    pub struct SingleTarget;

    #[derive(Component, Default)]
    pub struct MultiTarget;

    #[derive(Component, Default)]
    pub struct FrontTarget;

    #[derive(Component, Default)]
    pub struct BackTarget;

    #[derive(Component, Default)]
    pub struct EnemyTarget;

    #[derive(Component, Default)]
    pub struct AllyTarget;

    #[derive(Component, Default)]
    pub struct AnyTarget;

    #[derive(Component, Default)]
    pub struct CellTarget;

    #[derive(Component, Default)]
    pub struct Pierce;

    #[derive(Component, Default)]
    pub struct LineOfSight;

    #[derive(Component, Default)]
    pub struct Damage;

    #[derive(Component, Default)]
    pub struct Heal;

    #[derive(Component, Default)]
    pub struct Shield;

    #[derive(Component, Default)]
    pub struct Buff;

    #[derive(Component, Default)]
    pub struct Debuff;

    #[derive(Component, Default)]
    pub struct MultiHit;

    #[derive(Component, Default)]
    pub struct CellEffect;

    #[derive(Component, Default)]
    pub struct Glyph;

    #[derive(Component, Default)]
    pub struct HasSecondaryTarget;

    #[derive(Component, Default)]
    #[require(HasSecondaryTarget)]
    pub struct AdjacentSecondaryTarget;
}

#[derive(Component)]
pub struct ActionBar;

#[derive(Component)]
pub struct ActionCard;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct SelectedCard;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct SelectedAction;

#[derive(Component)]
pub struct CardArt(pub &'static str);

#[derive(AssetCollection, Resource)]
pub struct CardAssets {
    #[asset(path = "img/card_art", collection(typed, mapped))]
    pub backgrounds: HashMap<AssetFileStem, Handle<Image>>,
}

/// The `Action` owning this card.
#[derive(Component, Debug)]
#[relationship(relationship_target = OwnsCard)]
pub struct OwnedByAction(pub Entity);

/// The `Card` owned by this action.
#[derive(Component, Debug)]
#[relationship_target(relationship = OwnedByAction)]
pub struct OwnsCard(Entity);

/// The `Unit` owning this action.
#[derive(Component, Debug)]
#[relationship(relationship_target = OwnsAction)]
pub struct OwnedByUnit(pub Entity);

/// The `Action`s owned by this unit.
#[derive(Component, Debug)]
#[relationship_target(relationship = OwnedByUnit)]
pub struct OwnsAction(Vec<Entity>);

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
pub struct Target;

#[derive(Component, Debug, Default)]
#[component(storage = "SparseSet")]
pub struct SecondaryTarget;

#[derive(Component, Debug, Default)]
#[component(storage = "SparseSet")]
#[require(HighlightedCell { ty: Highlighter::Gray })]
pub struct Untargettable;

#[derive(Component, Debug, Default)]
#[component(storage = "SparseSet")]
pub struct InPlay;

pub fn set_action_bar(mut commands: Commands) {
    commands.spawn((
        ActionBar,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Percent(0.0),
            width: Val::Percent(100.0),
            height: Val::Px(200.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            padding: UiRect::all(Val::Px(10.0)),
            ..Default::default()
        },
        BackgroundColor(Color::BLACK.with_alpha(0.2)),
    ));
}

pub fn update_action_bar(
    mut commands: Commands,
    card_assets: Res<CardAssets>,
    q_bar: Query<Entity, With<ActionBar>>,
    q_actions: Query<(Entity, &Action, &CardArt), Added<AvailableAction>>,
) {
    let e_bar = q_bar.single().unwrap();

    for (e_action, action, card_art) in q_actions {
        commands.entity(e_bar).with_children(|parent| {
            let mut selection_tween = TweenAnim::new(Tween::new(
                EaseFunction::QuadraticOut,
                Duration::from_millis(2000),
                UiPositionLens {
                    start: UiRect::bottom(Val::Percent(0.0)).with_top(Val::Percent(0.0)),
                    end: UiRect::bottom(CARD_STICKOUT).with_top(-CARD_STICKOUT),
                },
            ))
            .with_destroy_on_completed(false);
            selection_tween.stop();

            parent
                .spawn((
                    ActionCard,
                    OwnedByAction(e_action),
                    Node {
                        position_type: PositionType::Relative,
                        width: Val::Auto,
                        height: Val::Percent(100.0),
                        bottom: Val::Percent(0.0),
                        aspect_ratio: Some(400.0 / 300.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        border_radius: BorderRadius::all(Val::Px(16.0)),
                        ..Default::default()
                    },
                    ImageNode::new(card_assets.backgrounds.get(card_art.0).unwrap().clone()),
                    Interaction::default(),
                    selection_tween,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(action.name),
                        TextFont {
                            font_size: FontSize::Px(20.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                    parent.spawn((
                        Text::new(action.description),
                        TextFont {
                            font_size: FontSize::Px(16.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
    }
}

pub fn select_deselect_card(
    mut commands: Commands,
    interaction_query: Query<
        (Entity, &Interaction, Option<&SelectedCard>),
        (Changed<Interaction>, With<ActionCard>),
    >,
) {
    for (e_card, interaction, selected) in interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if selected.is_none() {
                    commands.entity(e_card).insert(SelectedCard);
                } else {
                    commands.entity(e_card).remove::<SelectedCard>();
                }
            }
            _ => (),
        }
    }
}

pub fn select_card_animation(
    trigger: On<Add, SelectedCard>,
    mut commands: Commands,
    selected_query: Query<Entity, With<SelectedCard>>,
    mut anim_query: Query<&mut TweenAnim>,
) {
    let mut selection_tween = Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(200),
        UiPositionLens {
            start: UiRect::bottom(Val::Percent(0.0)).with_top(Val::Percent(0.0)),
            end: UiRect::bottom(CARD_STICKOUT).with_top(-CARD_STICKOUT),
        },
    );
    let mut tween_anim = anim_query.get_mut(trigger.entity).unwrap();
    let elapsed = tween_anim.tweenable().elapsed();
    selection_tween.set_elapsed(elapsed);
    tween_anim
        .set_tweenable(selection_tween.with_playback_direction(PlaybackDirection::Forward))
        .unwrap();
    tween_anim.playback_state = PlaybackState::Playing;

    for e_old in selected_query {
        if e_old == trigger.entity {
            continue;
        }

        commands.entity(e_old).remove::<SelectedCard>();
    }
}

pub fn deselect_card_animation(
    trigger: On<Remove, SelectedCard>,
    mut anim_query: Query<&mut TweenAnim>,
) {
    let mut selection_tween = Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(200),
        UiPositionLens {
            start: UiRect::bottom(Val::Percent(0.0)).with_top(Val::Percent(0.0)),
            end: UiRect::bottom(CARD_STICKOUT).with_top(-CARD_STICKOUT),
        },
    );
    let mut tween_anim = anim_query.get_mut(trigger.entity).unwrap();
    let elapsed = tween_anim.tweenable().elapsed();
    selection_tween.set_elapsed(elapsed);
    tween_anim
        .set_tweenable(selection_tween.with_playback_direction(PlaybackDirection::Backward))
        .unwrap();
    tween_anim.playback_state = PlaybackState::Playing;
}

pub fn select_action_on_card_select(
    trigger: On<Add, SelectedCard>,
    mut commands: Commands,
    card_query: Query<&OwnedByAction>,
) {
    let OwnedByAction(e_action) = card_query.get(trigger.entity).unwrap();
    commands.entity(*e_action).insert(SelectedAction);
}

pub fn deselect_action_on_card_deselect(
    trigger: On<Remove, SelectedCard>,
    mut commands: Commands,
    card_query: Query<&OwnedByAction>,
) {
    let OwnedByAction(e_action) = card_query.get(trigger.entity).unwrap();
    commands.entity(*e_action).remove::<SelectedAction>();
}

pub fn highlight_action_owner_cell(
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

    commands.entity(*e_cell).insert(HighlightedCell {
        ty: Highlighter::Blue,
    });
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

pub fn highlight_front_target_cells(
    trigger: On<Add, SelectedAction>,
    mut commands: Commands,
    target_query: Query<(), With<FrontTarget>>,
    enemy_query: Query<&OccupyingCell, (With<Unit>, With<Rival>)>,
    cell_query: Query<(Entity, &HexCoord)>,
) {
    if !target_query.contains(trigger.entity) {
        return;
    }

    let enemy_positions = enemy_query
        .iter()
        .map(|OccupyingCell(e_cell)| cell_query.get(*e_cell).unwrap())
        .collect::<Vec<_>>();

    if let Some(max_row) = enemy_positions.iter().map(|&(_, coord)| coord.row()).max() {
        let (targets, _non_targets): (Vec<_>, Vec<_>) = enemy_positions
            .into_iter()
            .partition(|&(_, coord)| coord.row() == max_row);

        for (e_cell, _) in targets {
            commands.entity(e_cell).insert(Targettable);
        }

        // for (e_cell, _) in non_targets {
        //     commands.entity(e_cell).insert(Untargettable);
        // }
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

pub fn play_primary_target_action(
    _trigger: On<Add, Target>,
    mut commands: Commands,
    action_query: Query<Entity, (With<SelectedAction>, Without<HasSecondaryTarget>)>,
) {
    let Ok(e_action) = action_query.single() else {
        return;
    };

    commands
        .entity(e_action)
        .insert(InPlay)
        .remove::<SelectedAction>();
}

pub fn highlight_secondary_targets(
    trigger: On<Add, Target>,
    mut commands: Commands,
    field: Res<Field<Player>>,
    target_query: Query<&HexCoord>,
    action_query: Query<(), (With<SelectedAction>, With<HasSecondaryTarget>)>,
) {
    let Ok(()) = action_query.single() else {
        return;
    };

    let coord = target_query.get(trigger.entity).unwrap();

    for e_adj in field.all_adjacent(*coord, HexCoord::DIRECTIONS.to_vec()) {
        commands.entity(e_adj).insert(SecondTargettable);
    }
}

pub fn play_secondary_target_action(
    _trigger: On<Add, SecondaryTarget>,
    mut commands: Commands,
    action_query: Query<Entity, (With<SelectedAction>, With<HasSecondaryTarget>)>,
) {
    let Ok(e_action) = action_query.single() else {
        return;
    };

    commands
        .entity(e_action)
        .insert(InPlay)
        .remove::<SelectedAction>();
}
