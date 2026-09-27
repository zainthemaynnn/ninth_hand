use std::time::Duration;

use bevy::{ecs::event::Trigger, platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_tweening::{
    AnimTarget, PlaybackDirection, PlaybackState, Tween, TweenAnim, Tweenable,
    lens::{UiPositionLens, UiTransformTranslationPxLens},
};
use rand::seq::SliceRandom;

use crate::asset::GameAssetLoadState;

const CARD_STICKOUT: Val = Val::Percent(50.0);

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameAssetLoadState::Next), set_action_bar)
            .add_systems(
                Update,
                (update_action_bar, select_deselect_card)
                    .run_if(in_state(GameAssetLoadState::Next)),
            )
            .add_observer(select_card_animation)
            .add_observer(deselect_card_animation)
            .configure_loading_state(
                LoadingStateConfig::new(GameAssetLoadState::AssetLoading)
                    .load_collection::<CardAssets>(),
            );
    }
}

#[derive(Component, Default)]
pub struct Action {
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Component)]
pub struct AvailableAction;

pub fn draw_typed<T: Component>(
    mut commands: Commands,
    q_actions: Query<Entity, (With<Action>, With<T>)>,
) {
    let mut rng = rand::rng();
    let n = 3;

    let mut entities: Vec<Entity> = q_actions.iter().collect();

    entities.shuffle(&mut rng);
    for e_action in &entities[0..n.min(entities.len())] {
        commands.entity(*e_action).insert(AvailableAction);
    }
}

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
}

#[derive(Component)]
pub struct ActionBar;

#[derive(Component)]
pub struct ActionCard;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct SelectedCard;

#[derive(Component)]
pub struct CardArt(pub &'static str);

#[derive(AssetCollection, Resource)]
pub struct CardAssets {
    #[asset(path = "img/card_art", collection(typed, mapped))]
    pub backgrounds: HashMap<AssetFileStem, Handle<Image>>,
}

pub fn set_action_bar(mut commands: Commands) {
    info!("set action bar");
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

    commands.spawn((crate::maw::Chomp, AvailableAction));
    commands.spawn((crate::maw::Chew, AvailableAction));
}

pub fn update_action_bar(
    mut commands: Commands,
    card_assets: Res<CardAssets>,
    q_bar: Query<Entity, With<ActionBar>>,
    q_actions: Query<(&Action, &CardArt), Added<AvailableAction>>,
) {
    let e_bar = q_bar.single().unwrap();

    for (action, card_art) in q_actions {
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

fn select_deselect_card(
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

fn select_card_animation(
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

fn deselect_card_animation(
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
