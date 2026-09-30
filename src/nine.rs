use bevy::prelude::*;

use crate::{
    actions::{Action, CardArt, tags::*},
    unit::{Unit, UnitDesc},
};

#[derive(Component)]
#[require(Unit)]
pub struct Nine;

impl Nine {
    const DESC: UnitDesc = UnitDesc {
        name: "Nine",
        passive: "Consume -- When killing an enemy, gets an additional action.",
    };
}

#[derive(Component)]
#[require(
	Action { name: "Crush", description: "Damage 5 front" },
	Damage,
	FrontTarget,
	SingleTarget,
	LineOfSight,
	CardArt("abyss"),
)]
pub struct Crush;

impl Crush {
    fn play() {
        todo!();
    }
}

#[derive(Component)]
#[require(
    Action { name: "Slap", description: "Damage 1 front 3x" },
    Damage,
    FrontTarget,
    SingleTarget,
    LineOfSight,
    MultiHit,
    CardArt("abyss"),
)]
pub struct Slap;

impl Slap {
    fn play() {
        todo!();
    }
}

#[derive(Component)]
#[require(
    Action { name: "Gaze", description: "Mark 1" },
    Debuff,
    FrontTarget,
    SingleTarget,
    LineOfSight,
    CardArt("abyss"),
)]
pub struct Gaze;

impl Gaze {
    fn play() {
        todo!();
    }
}

#[derive(Component)]
#[require(
    Action {
        name: "Swallow",
        description: "Move to front. Kill an enemy with 6 HP or less.",
    },
    Damage,
    FrontTarget,
    SingleTarget,
    LineOfSight,
)]
pub struct Swallow;

impl Swallow {
    fn play() {
        todo!();
    }
}
