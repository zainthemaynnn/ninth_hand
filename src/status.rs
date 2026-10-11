use std::marker::PhantomData;

use bevy::{color::palettes, prelude::*};

use crate::{
    actions::{DrawCount, DrawEvent, draw_actions},
    damage::{Damage, DamageEvent, DamageVariant, apply_damage_events},
    turn::{Cpu, Player, TurnState, shuffle_unit_actions},
};

pub struct StatusPlugin;

impl Plugin for StatusPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(TurnState::Neutral),
            (
                apply_dot_damage_events::<Burn>,
                apply_dot_damage_events::<Frostbite>,
                apply_dot_damage_events::<Affliction>,
            ),
        )
        .add_systems(
            Update,
            (
                apply_marks.before(apply_damage_events),
                apply_daze
                    .after(shuffle_unit_actions::<Player>)
                    .after(shuffle_unit_actions::<Cpu>)
                    .before(draw_actions),
                remove_expired_effects::<Burn>,
                remove_expired_effects::<Frostbite>,
                remove_expired_effects::<Affliction>,
                remove_expired_effects::<Mark>,
                remove_expired_effects::<Daze>,
            ),
        );
    }
}

#[derive(Component)]
pub struct Stacks<S: Status> {
    pub value: usize,
    pub phantom_data: PhantomData<S>,
}

impl<S: Status> Stacks<S> {
    pub fn new(value: usize) -> Self {
        Self {
            value,
            phantom_data: PhantomData::default(),
        }
    }
}

impl<S: Status> Default for Stacks<S> {
    fn default() -> Self {
        Self {
            value: 0,
            phantom_data: PhantomData::default(),
        }
    }
}

impl<S: Status> Stacks<S> {
    pub fn value(&self) -> usize {
        self.value
    }

    pub fn add(&mut self, amount: usize) {
        self.value += amount;
    }

    pub fn consume(&mut self, amount: usize) {
        self.value = std::cmp::max(self.value - amount, 0);
    }
}

pub trait Status: Send + Sync + 'static {
    fn color() -> Color;
}

pub trait DotDamageEffect: Status {
    fn damage_ty() -> DamageVariant;
}

macro_rules! create_dot_effect {
    ($name:ident, $damage_ty:expr) => {
        /// DOT status effect with
        #[doc = stringify!($damage_ty)]
        pub struct $name;

        impl Status for $name {
            fn color() -> Color {
                $damage_ty.color()
            }
        }

        impl DotDamageEffect for $name {
            fn damage_ty() -> DamageVariant {
                $damage_ty
            }
        }
    };
}

create_dot_effect!(Burn, DamageVariant::Heat);
create_dot_effect!(Frostbite, DamageVariant::Cold);
create_dot_effect!(Affliction, DamageVariant::Plague);

// literally have no idea why this mutable thing is needed here, but sure
pub fn apply_dot_damage_events<E: DotDamageEffect>(
    effect_query: Query<(Entity, &mut Stacks<E>)>,
    mut events: MessageWriter<DamageEvent>,
) {
    for (e_target, mut stacks) in effect_query {
        events.write(DamageEvent {
            damage: Damage {
                ty: E::damage_ty(),
                value: stacks.value() as f32,
                source: None,
            },
            target: e_target,
        });

        stacks.consume(1);
    }
}

pub fn remove_expired_effects<S: Status>(
    mut commands: Commands,
    effect_query: Query<(Entity, &Stacks<S>)>,
) {
    for (e_effect, stacks) in effect_query {
        if stacks.value() == 0 {
            commands.entity(e_effect).remove::<Stacks<S>>();
        }
    }
}

/// Doubles the next instance of damage.
pub struct Mark;

impl Status for Mark {
    fn color() -> Color {
        Color::Srgba(palettes::css::CRIMSON)
    }
}

pub fn apply_marks(
    mut damage_events: MessageMutator<DamageEvent>,
    mut mark_query: Query<&mut Stacks<Mark>>,
) {
    for event in damage_events.read() {
        if let Ok(mut stacks) = mark_query.get_mut(event.target) {
            event.damage *= 2.0;
            stacks.consume(1);
        }
    }
}

/// The unit draws less actions, equal to stacks. Consumed for each draw prevented.
pub struct Daze;

impl Status for Daze {
    fn color() -> Color {
        Color::Srgba(palettes::css::INDIGO)
    }
}

pub fn apply_daze(
    mut draw_events: MessageMutator<DrawEvent>,
    mut daze_query: Query<(&mut Stacks<Daze>, &mut DrawCount)>,
) {
    for event in draw_events.read() {
        if let Ok((mut stacks, mut draw_count)) = daze_query.get_mut(event.unit) {
            let penalty = std::cmp::min(stacks.value, draw_count.amount);
            draw_count.amount -= penalty;
            stacks.consume(penalty);
        }
    }
}
