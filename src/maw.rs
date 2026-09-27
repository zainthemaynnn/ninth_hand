use bevy::prelude::*;

use crate::{
    actions::{Action, CardArt, tags::*},
    unit::{Unit, UnitDesc},
};

#[derive(Component)]
#[require(Unit)]
pub struct Maw;

impl Maw {
    const DESC: UnitDesc = UnitDesc {
        name: "Maw",
        passive: "Consume -- When Maw kills an enemy, it gets an additional action.",
    };
}

#[derive(Component)]
#[require(
	Action { name: "Chomp", description: "Damage 5 front" },
	Damage,
	FrontTarget,
	SingleTarget,
	LineOfSight,
	CardArt("abyss"),
)]
pub struct Chomp;

impl Chomp {
    fn play() {
        todo!();
    }
}

#[derive(Component)]
#[require(
    Action { name: "Chew", description: "Damage 1 front 3x" },
    Damage,
    FrontTarget,
    SingleTarget,
    LineOfSight,
    MultiHit,
    CardArt("abyss"),
)]
pub struct Chew;

impl Chew {
    fn play() {
        todo!();
    }
}

#[derive(Component)]
#[require(Action, Debuff, FrontTarget, SingleTarget, LineOfSight)]
pub struct Lick;

impl Lick {
    const ACTION: Action = Action {
        name: "Lick",
        description: "Weakness 3 front",
    };

    fn play() {
        todo!();
    }
}

#[derive(Component)]
#[require(Action, Damage, FrontTarget, SingleTarget, LineOfSight)]
pub struct Swallow;

impl Swallow {
    const ACTION: Action = Action {
        name: "Swallow",
        description: "Move to front. Kill an enemy with 6 HP or less.",
    };

    fn play() {
        todo!();
    }
}
