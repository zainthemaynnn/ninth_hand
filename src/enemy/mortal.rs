use bevy::prelude::*;

use crate::{turn::Cpu, unit::{Unit, UnitDesc}};

#[derive(Component)]
#[require(Unit, Cpu)]
pub struct Mortal;

impl Mortal {
    const DESC: UnitDesc = UnitDesc {
        name: "Mortal",
        passive: "Nothing.",
    };
}
