use std::time::Duration;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_tweening::{PlaybackDirection, PlaybackState, Tween, TweenAnim, lens::UiPositionLens};

use crate::{asset::{GameState, UiRightLens}, ui::UiRumble};

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, SubStates)]
#[source(GameState = GameState::InGame)]
pub enum TurnState {
    Player,
    PlayerBusy,
    Cpu,
    CpuBusy,
    #[default]
    Neutral,
    NeutralBusy,
}

#[derive(AssetCollection, Resource)]
struct TurnUiAssets {
    #[asset(path = "img/smile.png")]
    pub smile: Handle<Image>,
}

pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<TurnState>()
            .configure_loading_state(
                LoadingStateConfig::new(GameState::AssetLoading).load_collection::<TurnUiAssets>(),
            )
            .add_systems(OnEnter(GameState::InGame), insert_player_turn_button)
            .add_systems(OnEnter(TurnState::Player), (set_ally_rival::<Player, Cpu>, show_player_turn_button))
            .add_systems(OnEnter(TurnState::PlayerBusy), hide_player_turn_button)
            .add_systems(Update, plr_busy_wait.run_if(in_state(TurnState::PlayerBusy)))
            .add_systems(OnEnter(TurnState::Cpu), set_ally_rival::<Cpu, Player>)
            .add_systems(
                Update,
                advance_player_turn_button.run_if(in_state(TurnState::Player)),
            )
            // placeholders for now
            .add_systems(OnEnter(TurnState::Cpu), advance_enemy_turn)
            .add_systems(OnEnter(TurnState::Neutral), advance_neutral_turn);
    }
}

/// Describes components related to the current turn's player.
#[derive(Component, Default)]
pub struct Ally;

/// Describes components related to the current turn's enemy.
#[derive(Component, Default)]
pub struct Rival;

pub trait Team: Send + Sync + 'static {}

#[derive(Component, Default)]
pub struct Player;

impl Team for Player {}

#[derive(Component, Default)]
pub struct Cpu;

impl Team for Cpu {}

pub fn set_ally_rival<A: Component, R: Component>(
    mut commands: Commands,
    ally_query: Query<Entity, With<A>>,
    rival_query: Query<Entity, With<R>>,
) {
    for e_ally in ally_query {
        commands.entity(e_ally).remove::<A>().insert(Rival);
    }

    for e_rival in rival_query {
        commands.entity(e_rival).remove::<R>().insert(Ally);
    }
}

#[derive(Component, Default)]
pub struct EndTurnButton;

pub fn insert_player_turn_button(mut commands: Commands) {
    let mut tween_anim = TweenAnim::new(Tween::new(
        EaseFunction::QuadraticIn,
        Duration::from_millis(200),
        UiPositionLens  {
            start: UiRect::right(Val::Px(0.0)),
            end: UiRect::right(Val::Px(-48.0)),
        }
    )).with_destroy_on_completed(false);
    tween_anim.playback_state = PlaybackState::Paused;

    commands
        .spawn((
            EndTurnButton,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(50.0),
                right: Val::Percent(0.0),
                width: Val::Px(48.0),
                height: Val::Px(240.0),
                padding: UiRect::all(Val::Px(10.0)),
                border_radius: BorderRadius::left(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            BackgroundColor(Color::BLACK),
            UiTransform::from_xy(Val::Auto, Val::Px(-120.0)),
            Interaction::default(),
            tween_anim,
        ))
        .with_children(|parent| {
            for (i, letter) in "KILL".chars().enumerate() {
                parent.spawn((
                    Text(letter.into()),
                    TextFont {
                        font_size: FontSize::Px(48.0),
                        ..default()
                    },
                    TextLayout {
                        justify: Justify::Center,
                        linebreak: LineBreak::AnyCharacter,
                    },
                    UiRumble {
                        center: Val2::ZERO,
                        strength: Val2::ZERO,
                        speed: 8.0,
                        t: i as f32 * 100.0,
                    },
                ));
            }
        });
}

pub fn show_player_turn_button(mut button_query: Query<&mut TweenAnim, With<EndTurnButton>>) {
    let mut anim = button_query.single_mut().unwrap();

    let mut show_tween = Tween::new(
        EaseFunction::QuadraticIn,
        Duration::from_millis(200),
        UiRightLens  {
            start: Val::Px(-48.0),
            end: Val::Px(0.0),
        }
    );

    anim.set_tweenable(show_tween);
    anim.playback_state = PlaybackState::Playing;
}

pub fn hide_player_turn_button(mut button_query: Query<&mut TweenAnim, With<EndTurnButton>>) {
    let mut anim = button_query.single_mut().unwrap();

    let mut hide_tween = Tween::new(
        EaseFunction::QuadraticIn,
        Duration::from_millis(200),
        UiRightLens  {
            start: Val::Px(0.0),
            end: Val::Px(-48.0),
        }
    );

    anim.set_tweenable(hide_tween);
    anim.playback_state = PlaybackState::Playing;
}

pub fn advance_player_turn_button(
    interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<EndTurnButton>)>,
    children_query: Query<&Children>,
    mut rumble_query: Query<&mut UiRumble>,
    mut next_state: ResMut<NextState<TurnState>>,
) {
    for (e_button, interaction) in interaction_query {
        let children = children_query.get(e_button).unwrap();
        match *interaction {
            Interaction::Pressed => {
                next_state.set(TurnState::PlayerBusy);
            }
            Interaction::Hovered => {
                for e_child in children {
                    let mut rumble = rumble_query.get_mut(*e_child).unwrap();
                    rumble.strength = Val2::all(Val::Px(2.0));
                }
            }
            Interaction::None => {
                for e_child in children {
                    let mut rumble = rumble_query.get_mut(*e_child).unwrap();
                    rumble.strength = Val2::all(Val::ZERO);
                }
            }
        }
    }
}

pub fn advance_enemy_turn(mut next_state: ResMut<NextState<TurnState>>) {
    next_state.set(TurnState::Neutral);
}

pub fn advance_neutral_turn(mut next_state: ResMut<NextState<TurnState>>) {
    next_state.set(TurnState::Player);
}

// dummy turn wait thing
pub fn plr_busy_wait(
    time: Res<Time>, 
    // Local timer initialized automatically to a 1.0 second non-repeating fuse
    mut my_timer: Local<Timer>, 
    mut initialized: Local<bool>,
    mut next_state: ResMut<NextState<TurnState>>,
) {
    // Setup the timer properties on the very first frame it runs
    if !*initialized {
        *my_timer = Timer::new(Duration::from_secs(1), TimerMode::Once);
        *initialized = true;
    }

    // 1. Manually tick the timer forward by the elapsed frame time
    my_timer.tick(time.delta());

    // 2. Check if the 1 second wait just completed on this exact frame
    if my_timer.just_finished() {
        *initialized = false;
        next_state.set(TurnState::Cpu);
    }
}
