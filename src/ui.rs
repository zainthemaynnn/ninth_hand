use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{
    AnimCompletedEvent, AnimTarget, Tween, TweenAnim,
    lens::{TextColorLens, UiPositionLens},
};
use noisy_bevy::simplex_noise_2d;

pub struct NineUiPlugin;

impl Plugin for NineUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<FloatyTextEvent>()
            .add_systems(Update, (do_rumble, spawn_floaty_texts));
    }
}

#[derive(Component)]
#[require(UiTransform)]
pub struct UiRumble {
    pub center: Val2,
    pub strength: Val2,
    pub speed: f32,
    pub t: f32,
}

impl Default for UiRumble {
    fn default() -> Self {
        Self {
            center: Val2::ZERO,
            strength: Val2::ZERO,
            speed: 1.0,
            t: 0.0,
        }
    }
}

pub fn do_rumble(rumble_query: Query<(&mut UiTransform, &mut UiRumble)>, time: Res<Time>) {
    for (mut tf, mut rumble) in rumble_query {
        rumble.t += time.delta_secs() * rumble.speed;

        let noise_x = rumble.strength.x * simplex_noise_2d(Vec2::new(rumble.t, 0.0));
        let noise_y = rumble.strength.y * simplex_noise_2d(Vec2::new(rumble.t, 100.0));

        tf.translation = rumble.center.try_add(Val2::new(noise_x, noise_y)).unwrap();
    }
}

#[derive(Message)]
pub struct FloatyTextEvent {
    pub text: String,
    pub location: Vec2,
    pub fade: Duration,
    pub size: f32,
    pub color_start: Color,
    pub color_end: Color,
}

impl Default for FloatyTextEvent {
    fn default() -> Self {
        Self {
            text: String::default(),
            location: Vec2::default(),
            fade: Duration::from_secs(1),
            size: 24.0,
            color_start: Color::WHITE,
            color_end: Color::BLACK.with_alpha(0.0),
        }
    }
}

fn floaty_text_template(text: String, location: Vec2, size: f32) -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(location.y),
            left: px(location.x),
        }

        Text::new(text)

        TextFont {
            font_size: FontSize::Px(size),
        }
    }
}

pub fn spawn_floaty_texts(mut commands: Commands, mut events: MessageReader<FloatyTextEvent>) {
    for FloatyTextEvent {
        text,
        location,
        fade,
        size,
        color_start,
        color_end,
    } in events.read()
    {
        let mut rng = rand::rng();
        let spawn_zone = Circle::new(1.0);
        let float_dir = spawn_zone.sample_boundary(&mut rng);

        commands
            .spawn_scene(floaty_text_template(text.clone(), *location, *size))
            .with_children(|parent| {
                let move_tween = Tween::new(
                    EaseFunction::QuadraticOut,
                    fade.clone(),
                    UiPositionLens {
                        start: UiRect {
                            top: Val::Px(location.y),
                            left: Val::Px(location.x),
                            ..Default::default()
                        },
                        end: UiRect {
                            top: Val::Px(location.y + float_dir.y * 64.0),
                            left: Val::Px(location.x + float_dir.x * 64.0),
                            ..Default::default()
                        },
                    },
                );
                let color_tween = Tween::new(
                    EaseFunction::QuadraticIn,
                    fade.clone(),
                    TextColorLens {
                        start: *color_start,
                        end: *color_end,
                    },
                );

                parent.spawn((
                    TweenAnim::new(move_tween),
                    AnimTarget::component::<Node>(parent.target_entity()),
                ));
                parent
                    .spawn((
                        TweenAnim::new(color_tween),
                        AnimTarget::component::<TextColor>(parent.target_entity()),
                    ))
                    .observe(
                        move |trigger: On<AnimCompletedEvent>,
                              mut commands: Commands,
                              parent: Query<&ChildOf>| {
                            commands
                                .entity(parent.get(trigger.anim_entity).unwrap().0)
                                .despawn();
                            info!("floaty text despawned");
                        },
                    );
            });
    }
}
