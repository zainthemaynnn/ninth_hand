use bevy::{color::palettes, ecs::component::Mutable, prelude::*};

use crate::{
    damage::{Damage, DamageEvent, DamageVariant, apply_damage_events},
    turn::TurnState,
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
                remove_expired_effects::<Burn>,
                remove_expired_effects::<Frostbite>,
                remove_expired_effects::<Affliction>,
                remove_expired_effects::<Mark>,
            ),
        );
    }
}

pub trait Status: Component {
    fn stacks(&self) -> u32;
    fn consume(&mut self, amount: u32);
    fn color() -> Color;
}

pub trait DotDamageEffect: Status {
    fn damage_ty() -> DamageVariant;
}

macro_rules! create_dot_effect {
    ($name:ident, $damage_ty:expr) => {
        /// DOT status effect with
        #[doc = stringify!($damage_ty)]
        #[derive(Component)]
        pub struct $name {
            pub stacks: u32,
        }

        impl Status for $name {
            fn stacks(&self) -> u32 {
                self.stacks
            }

            fn consume(&mut self, amount: u32) {
                self.stacks -= amount
            }

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
pub fn apply_dot_damage_events<E: DotDamageEffect + Component<Mutability = Mutable>>(
    effect_query: Query<(Entity, &mut E)>,
    mut events: MessageWriter<DamageEvent>,
) {
    for (e_target, mut effect) in effect_query {
        events.write(DamageEvent {
            damage: Damage {
                ty: E::damage_ty(),
                value: effect.stacks() as f32,
                source: None,
            },
            target: e_target,
        });

        effect.consume(1);
    }
}

pub fn remove_expired_effects<S: Status>(
    mut commands: Commands,
    effect_query: Query<(Entity, &S)>,
) {
    for (e_effect, status) in effect_query {
        if status.stacks() == 0 {
            commands.entity(e_effect).remove::<S>();
        }
    }
}

#[derive(Component)]
pub struct Mark {
    pub stacks: u32,
}

impl Status for Mark {
    fn stacks(&self) -> u32 {
        self.stacks
    }

    fn consume(&mut self, amount: u32) {
        self.stacks -= amount
    }

    fn color() -> Color {
        Color::Srgba(palettes::css::CRIMSON)
    }
}

pub fn apply_marks(
    mut damage_events: MessageMutator<DamageEvent>,
    mut mark_query: Query<&mut Mark>,
) {
    for event in damage_events.read() {
        if let Ok(mut mark) = mark_query.get_mut(event.target) {
            event.damage *= 2.0;
            mark.consume(1);
        }
    }
}
