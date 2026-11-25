use std::time::Duration;

use bevy::{
    ecs::relationship::Relationship, platform::collections::HashMap, prelude::*,
    text::FontSourceTemplate,
};
use bevy_asset_loader::prelude::*;
use bevy_tweening::{
    AnimCompletedEvent, AnimTarget, PlaybackDirection, PlaybackState, Tween, TweenAnim, Tweenable,
    lens::{UiPositionLens, UiTransformRotationLens},
};

use crate::{
    actions::action::{Action, AvailableAction, OwnedByUnit, SelectedAction},
    turn::Player,
    unit::{UnitColor, UnitFont, UnitPriority},
};

const CARD_STICKOUT: Val = Val::Percent(50.0);
#[derive(Component)]
pub struct ActionBar;

#[derive(Component)]
pub struct ActionCard;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct SelectedCard;

#[derive(Component)]
pub struct CardArt(pub &'static str);

#[derive(Component, Clone)]
pub struct CardTextColor(pub TextColor);

pub fn estimate_text_color(
    mut commands: Commands,
    card_assets: Res<CardAssets>,
    images: Res<Assets<Image>>,
    cards_query: Query<(Entity, &CardArt), Without<CardTextColor>>,
) {
    for (e_card, CardArt(art_key)) in &cards_query {
        let Some(image_handle) = card_assets.card_art.get(*art_key) else {
            commands
                .entity(e_card)
                .insert(CardTextColor(TextColor(Color::BLACK)));
            continue;
        };

        let image = images
            .get(image_handle)
            .expect("card art image should be loaded");
        let Some(pixels) = image.data.as_ref() else {
            continue;
        };

        let (greyscale_sum, pixel_count) =
            pixels
                .chunks_exact(4)
                .fold((0.0, 0usize), |(greyscale_sum, pixel_count), pixel| {
                    let greyscale =
                        (pixel[0] as f32 + pixel[1] as f32 + pixel[2] as f32) / (3.0 * 255.0);
                    (greyscale_sum + greyscale, pixel_count + 1)
                });
        if pixel_count == 0 {
            continue;
        }

        let average_greyscale = greyscale_sum / pixel_count as f32;
        let text_color = if average_greyscale < 0.5 {
            Color::WHITE
        } else {
            Color::BLACK
        };
        commands
            .entity(e_card)
            .insert(CardTextColor(TextColor(text_color)));
    }
}

pub fn add_generic_card_art(
    mut commands: Commands,
    cards_query: Query<Entity, (With<Action>, Without<CardArt>)>,
) {
    for e_card in &cards_query {
        commands.entity(e_card).insert(CardArt("generic_attack"));
    }
}

#[derive(AssetCollection, Resource)]
pub struct CardAssets {
    #[asset(path = "img/card_art", collection(typed, mapped))]
    pub card_art: HashMap<AssetFileStem, Handle<Image>>,
    #[asset(path = "img/patterns/nineslice", collection(typed, mapped))]
    pub patterns: HashMap<AssetFileStem, Handle<Image>>,
}

pub(crate) fn create_action_card(
    action: &Action,
    font: Handle<Font>,
    color: TextColor,
) -> impl Scene {
    let name = action.name;
    let description = action.description;
    let description_font = font.clone();
    let text_color = color.0;
    bsn! {
        Node {
            position_type: PositionType::Relative,
            height: Val::Px(240.0),
            width: Val::Px(180.0),
            bottom: Val::Percent(0.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            border_radius: BorderRadius::all(Val::Px(16.0)),
        }
        Children [
            {
                bsn! {
                Text::new(name)
                Node {
                    width: Val::Percent(85.0),
                    flex_shrink: 1.0,
                }
                TextFont {
                    font: FontSourceTemplate::Handle({ font }),
                    font_size: FontSize::Px(20.0),
                }
                TextLayout {
                    justify: Justify::Center,
                    linebreak: LineBreak::WordBoundary,
                }
                TextColor({ text_color })
                }
            },
            {
                bsn! {
                Text::new(description)
                Node {
                    width: Val::Percent(85.0),
                    flex_shrink: 1.0,
                }
                TextFont {
                    font: FontSourceTemplate::Handle({ description_font }),
                    font_size: FontSize::Px(16.0),
                }
                TextLayout {
                    justify: Justify::Center,
                    linebreak: LineBreak::WordBoundary,
                }
                TextColor({ text_color })
                }
            },
        ]
    }
}

/// The `Action` owning this card.
#[derive(Component, Debug)]
#[relationship(relationship_target = OwnsCard)]
pub struct OwnedByAction(pub Entity);

/// The `Card` owned by this action.
#[derive(Component, Debug)]
#[relationship_target(relationship = OwnedByAction)]
pub struct OwnsCard(Entity);

impl OwnsCard {
    pub(crate) fn entity(&self) -> Entity {
        self.0
    }
}

#[derive(Component, Debug, Default)]
pub struct Discarded;

/// The `Unit` owning this action.
#[derive(Component, Debug)]
#[relationship(relationship_target = UnitOwnsActionBar)]
pub struct ActionBarOwnedByUnit(pub Entity);

/// The `Action`s owned by this unit.
#[derive(Component, Debug)]
#[relationship_target(relationship = ActionBarOwnedByUnit)]
pub struct UnitOwnsActionBar(Entity);

pub fn set_action_bar(
    mut commands: Commands,
    unit_query: Query<(Entity, &UnitColor, &UnitPriority), With<Player>>,
) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Percent(0.0),
            width: Val::Percent(100.0),
            height: Val::Px(200.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            ..Default::default()
        })
        .with_children(move |parent| {
            for (e_unit, UnitColor(color), _) in
                unit_query
                    .iter()
                    .sort_by::<(Entity, &UnitColor, &UnitPriority)>(
                        |(_, _, UnitPriority(p0)), (_, _, UnitPriority(p1))| p0.cmp(p1),
                    )
            {
                /*
                let slicer = TextureSlicer {
                    border: BorderRect::all(20.0),
                    center_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
                    sides_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
                    ..Default::default()
                };
                */
                parent.spawn((
                    ActionBar,
                    Node {
                        bottom: Val::Percent(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceEvenly,
                        ..Default::default()
                    },
                    BackgroundColor(*color),
                    // Nine slice background, disabled for now:
                    // ImageNode {
                    //     image: assets.patterns.get(name.to_lowercase().as_str()).unwrap().clone(),
                    //     image_mode: NodeImageMode::Sliced(slicer),
                    //     ..Default::default()
                    // },
                    ActionBarOwnedByUnit(e_unit),
                ));
            }
        });
}

pub fn update_action_bar(
    mut commands: Commands,
    card_assets: Res<CardAssets>,
    actions_query: Query<
        (
            Entity,
            &Action,
            Option<&CardTextColor>,
            &CardArt,
            &OwnedByUnit,
        ),
        Added<AvailableAction>,
    >,
    unit_query: Query<(Option<&UnitFont>, &UnitOwnsActionBar)>,
) {
    for (e_action, action, card_text_color, card_art, owned_by) in actions_query {
        let Ok((font, owns)) = unit_query.get(owned_by.get()) else {
            continue;
        };

        let font = font
            .map(|unit_font| unit_font.0.clone())
            .unwrap_or_default();
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

        let image = card_assets
            .card_art
            .get(card_art.0)
            .expect(format!("missing card art {0}", card_art.0).as_str())
            .clone();

        commands
            .spawn_scene(create_action_card(
                action,
                font,
                card_text_color
                    .map(|color| color.0.clone())
                    .unwrap_or(TextColor(Color::BLACK)),
            ))
            .insert((
                ChildOf(owns.0),
                ActionCard,
                OwnedByAction(e_action),
                ImageNode::new(image),
                Interaction::default(),
                UiTransform::default(),
                selection_tween,
            ));
    }
}

pub fn select_deselect_card(
    mut commands: Commands,
    interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<ActionCard>)>,
    selected_query: Query<Entity, With<SelectedCard>>,
) {
    for (e_card, interaction) in interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if !selected_query.contains(e_card) {
                    commands.entity(e_card).insert(SelectedCard);

                    if let Ok(e_selected) = selected_query.single() {
                        commands.entity(e_selected).remove::<SelectedCard>();
                    }
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
    mut commands: Commands,
    added_cards: Query<(Entity, &OwnedByAction), Added<SelectedCard>>,
) {
    for (_, OwnedByAction(e_action)) in &added_cards {
        commands.entity(*e_action).insert(SelectedAction);
    }
}

pub fn deselect_action_on_card_deselect(
    mut commands: Commands,
    mut removed_cards: RemovedComponents<SelectedCard>,
    card_query: Query<&OwnedByAction>,
) {
    for e_card in removed_cards.read() {
        let Ok(OwnedByAction(e_action)) = card_query.get(e_card) else {
            continue;
        };
        commands.entity(*e_action).remove::<SelectedAction>();
    }
}

pub fn discard_card_animation(trigger: On<Add, Discarded>, mut commands: Commands) {
    let rot_tween = Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(100),
        UiTransformRotationLens {
            start: Rot2::IDENTITY,
            end: Rot2::degrees(15.0),
        },
    );
    let move_tween = Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(800),
        UiPositionLens {
            start: UiRect::bottom(CARD_STICKOUT).with_top(-CARD_STICKOUT),
            end: UiRect::bottom(Val::Percent(-120.0)).with_top(Val::Percent(120.0)),
        },
    );

    commands.entity(trigger.entity).with_children(|parent| {
        parent.spawn((
            TweenAnim::new(rot_tween),
            AnimTarget::component::<UiTransform>(parent.target_entity()),
        ));
        parent
            .spawn((
                TweenAnim::new(move_tween),
                AnimTarget::component::<Node>(parent.target_entity()),
            ))
            .observe(
                move |trigger: On<AnimCompletedEvent>,
                      mut commands: Commands,
                      parent: Query<&ChildOf>| {
                    commands
                        .entity(parent.get(trigger.anim_entity).unwrap().0)
                        .despawn();
                },
            );
    });
}
