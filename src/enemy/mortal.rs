use bevy::prelude::*;

use crate::{
    actions::{Action, tags::*},
    damage::{Health, HealthCap},
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
    Damage,
    FrontTarget,
    SingleTarget,
    LineOfSight,
)]
pub struct Punch;
