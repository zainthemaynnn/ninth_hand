//! !!! SPAGHETTI WARNING !!!
//!
//! This module manages what you think it does.
//! Also it only supports ASCII right now. Didn't think that far ahead. Whoops.
//!
//! Did I mention I hate UI?

use bevy::{
    prelude::*,
    reflect::TypePath,
    ui::FocusPolicy,
    utils::{HashMap, HashSet},
};
use bevy_asset_loader::prelude::*;
use itertools::Itertools;

pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DefaultTextStyle>()
            .init_resource::<StopChars>()
            .init_state::<DialogueAssetLoadState>()
            // .add_event::<DialogueEvent>()
            .add_systems(Startup, init_dialogue_box)
            .add_systems(
                Update,
                (
                    prepare_dialogue_block,
                    speak_dialogue,
                    continue_dialogue,
                    render_dialogue_portrait,
                )
                    .chain(),
            );
    }
}

/// Procedurally iterates the characters in a block of dialogue.
///
/// Characters in the `StopChars` resource will momentarily pause dialogue,
/// but *only* if followed by whitespace.
#[derive(Component)]
pub struct TextMotor {
    /// Characters per second.
    pub cps: f32,
    /// Delay after punctuation, in seconds.
    pub stop_delay: f32,
    /// Iterates `TextSection`s for the active block of dialogue.
    pub sections: Box<dyn Iterator<Item = TextSection> + Send + Sync + 'static>,
    /// Iterates characters of the current `TextSection`.
    pub chars: Box<dyn Iterator<Item = char> + Send + Sync + 'static>,
    /// Sound blip when iterating a character.
    pub blip: Handle<AudioSource>,
    /// Accumulated time without writing a character.
    pub acc: f32,
    /// Most recently pushed character.
    pub latest_char: char,
    /// Whether this will skip on the next frame.
    pub skip: bool,
}

/// Which characters followed by whitespace indicate a "pause" in dialogue.
#[derive(Resource)]
pub struct StopChars(pub HashSet<char>);

impl Default for StopChars {
    fn default() -> Self {
        Self(HashSet::from_iter(['.', ',', ';', ':', '!', '?']))
    }
}

#[derive(Component)]
pub struct DialogueWindow;

#[derive(Component)]
pub struct DialoguePortrait;

#[derive(Component)]
pub struct DialogueText;

#[derive(Component)]
pub struct IconContainer;

pub fn init_dialogue_box(mut commands: Commands) {
    commands
        .spawn(bsn! {
        // 1. Root Container
        Node {
            position_type: PositionType::Absolute,
            direction: Direction::LeftToRight,
            flex_direction: FlexDirection::Row,
            bottom: percent(0.0),
            right: percent(0.0),
            padding: UiRect::all(px(8.0)),
            width: percent(100.0),
            height: px(200.0),
            column_gap: px(8.0),
        }
        BackgroundColor(Color::BLACK.with_alpha(0.5))
        FocusPolicy::Block
        ZIndex::Global(1000)
        DialogueWindow

        Children [
            Node {
                aspect_ratio: 1.0,
                height: percent(100.0),
            }
            ImageNode::default()
            BackgroundColor(Color::WHITE)
            DialoguePortrait

            Node {
                flex_grow: 1.0,
                flex_basis: px(0.0),
                padding: UiRect::all(px(16.0)),
            }
            BackgroundColor(Color::BLACK.with_alpha(0.5))
            
            Children [
                Node {
                    height: percent(100.0),
                }
                Text::default()
                DialogueText
            ]
        ]
    });
}

#[derive(Asset, Default, Clone, TypePath)]
pub struct Dialogue {
    pub text: Text,
    pub portrait: Portrait,
    pub blip: Handle<AudioSource>,
    pub cps: f32,
    pub stop_delay: f32,
    pub next: DialogueNext,
}

#[derive(Component, Default, Clone)]
pub enum DialogueNext {
    /// Another block of text.
    Continue(String),
    /// End dialogue.
    #[default]
    Finish,
}

#[derive(Event)]
pub enum DialogueEvent {
    Say(Handle<Dialogue>),
    Finish,
}

pub fn prepare_dialogue_block(
    mut commands: Commands,
    dialogue_assets: Res<Assets<Dialogue>>,
    mut text_query: Query<(Entity, &mut Text), With<DialogueText>>,
    mut window_query: Query<&mut Style, With<DialogueWindow>>,
    mut events: EventReader<DialogueEvent>,
    mut portrait_events: EventWriter<DialoguePortraitEvent>,
) {
    let (e_text, mut text) = text_query.single_mut();
    for event in events.read() {
        text.sections.clear();
        match event {
            DialogueEvent::Say(h_dialogue) => {
                let Dialogue {
                    text,
                    portrait,
                    blip,
                    cps,
                    stop_delay,
                    next,
                } = dialogue_assets.get(h_dialogue).unwrap().clone();

                commands.entity(e_text).insert((
                    TextMotor {
                        cps,
                        stop_delay,
                        sections: Box::new(text.sections.into_iter()),
                        // will be filled on the first iteration
                        chars: Box::new(std::iter::empty()),
                        blip: blip.clone(),
                        acc: 0.0,
                        latest_char: '\n', // placeholder
                        skip: false,
                    },
                    next,
                ));
            }
            DialogueEvent::Finish => {
                let mut style = window_query.single_mut();
                style.display = Display::None;
            }
        }
    }
}

pub fn speak_dialogue(
    mut commands: Commands,
    time: Res<Time>,
    stop_chars: Res<StopChars>,
    sink_query: Query<&AudioSink>,
    mut text_query: Query<(Entity, &mut Text, &mut TextMotor), With<DialogueText>>,
) {
    let Ok((e_text, mut text, mut motor)) = text_query.get_single_mut() else {
        return;
    };

    motor.acc += time.delta_seconds();
    let mut spoke = false;

    while motor.skip || motor.acc >= 1.0 / motor.cps {
        match motor.chars.next() {
            Some(c) => {
                text.sections.last_mut().unwrap().value.push(c);

                if !spoke && !c.is_whitespace() {
                    spoke = true;
                }

                // apply extra delay for punctuation
                if (c.is_whitespace() || c == '"') && stop_chars.0.contains(&motor.latest_char) {
                    motor.acc -= motor.stop_delay;
                } else {
                    motor.acc -= 1.0 / motor.cps;
                }

                motor.latest_char = c;
            }
            None => match motor.sections.next() {
                Some(s) => {
                    // copy the style, but put the text in the motor
                    motor.chars = Box::new(s.value.chars().collect_vec().into_iter());
                    text.sections.push(TextSection::from_style(s.style));
                }
                None => {
                    // finished
                    commands.entity(e_text).remove::<TextMotor>();
                    break;
                }
            },
        }
    }

    if spoke {
        // terminate the current blip since it's not looking for overlaps
        if let Ok(blip) = sink_query.get(e_text) {
            blip.stop();
        }
        commands.entity(e_text).insert(AudioBundle {
            source: motor.blip.clone(),
            ..Default::default()
        });
    }
}
