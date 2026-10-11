use bevy::prelude::*;

use crate::{
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
    }
)]
pub struct Mortal;
