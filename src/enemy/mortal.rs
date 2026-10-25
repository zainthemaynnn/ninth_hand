use bevy::prelude::*;

use crate::{
    actions::{
        Action, ActionType, OwnedByUnit, Played, Target,
        tags::{self, *},
    },
    damage::{Damage, DamageEvent, DamageVariant, Health, HealthCap},
    field::OccupiedByUnit,
    turn::Cpu,
    unit::{Unit, UnitDesc},
};

#[derive(Component)]
#[require(
    Unit,
    Cpu,
    UnitDesc {
        name: "Mortal",
        passive: "A useless mortal.",
        flavor: "Yeah, I'm just a guy.",
    },
    Health(999.0),
    HealthCap(999.0),
)]
pub struct Mortal;

#[derive(Component)]
#[require(
    Action { name: "Punch", description: "Damage 5 front" },
    tags::Damage,
    FrontTarget,
    SingleTarget,
    LineOfSight,
)]
pub struct Punch;

impl ActionType for Punch {
    fn play() -> Observer {
        Observer::new(
            |trigger: On<Played>,
             action_query: Query<&OwnedByUnit>,
             target_query: Single<&OccupiedByUnit, With<Target>>,
             mut damage_events: MessageWriter<DamageEvent>| {
                let e_unit = action_query.get(trigger.entity).unwrap().0;
                let occupied_by = target_query.into_inner();
                let e_enemy = occupied_by.iter().next().unwrap();

                damage_events.write(DamageEvent {
                    damage: Damage {
                        ty: DamageVariant::Bash,
                        value: 5.0,
                        source: Some(e_unit),
                    },
                    target: e_enemy,
                });
            },
        )
    }
}
