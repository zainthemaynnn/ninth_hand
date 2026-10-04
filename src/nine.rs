use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    actions::{Action, CardArt, tags::*},
    damage::{DamageVariant, Health, HealthCap, Resist},
    field::Field,
    turn::{Ally, Cpu, Player, Rival, TurnState},
    unit::{Unit, UnitDesc},
};

#[derive(Component)]
#[require(
    Unit,
    Player,
    UnitDesc {
        name: "Nine",
        passive: "When killing an enemy, gets an additional action.",
        flavor: "MWAHAHAHAHA",
    },
    Health(10.0),
    HealthCap(10.0),
    Resist(HashMap::from([
        (DamageVariant::Abyssal, 0.5),
        (DamageVariant::Radiant, 2.0),
    ])),
)]
pub struct Nine;

#[derive(Component, Debug)]
#[relationship(relationship_target = HandOwnedBy)]
pub struct OwnsHand(pub Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = OwnsHand)]
pub struct HandOwnedBy(Entity);

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
    fn play(
        ally_field: Res<Field<Player>>,
        rival_field: Res<Field<Cpu>>,
        mut turn_state: ResMut<NextState<TurnState>>,
        unit_query: Query<(&OwnsHand), With<Nine>>,
        mut hand_query: Query<(&mut Transform,)>,
    ) {
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
