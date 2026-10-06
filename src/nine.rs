use bevy::{ecs::system::IntoObserverSystem, platform::collections::HashMap, prelude::*};

use crate::{
    actions::{Action, ActionHandler, CardArt, InPlay, Target, tags::*}, damage::{Damage, DamageBuffer, DamageVariant, Health, HealthCap, Resist}, field::{Field, OccupiedByUnit}, turn::{Ally, Cpu, Player, Rival, TurnState}, unit::{Unit, UnitDesc},
};

pub struct NinePlugin;

impl Plugin for NinePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(Crush::play);
    }
}

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
        trigger: On<Add, InPlay>,
        crush_query: Query<(), With<Crush>>,
        target_query: Query<(Entity, &OccupiedByUnit), With<Target>>,
        mut enemy_query: Query<&mut DamageBuffer>,
        unit_query: Query<(Entity, &OwnsHand), With<Nine>>,
        mut hand_query: Query<(&mut Transform,)>,
    ) {
        if !crush_query.contains(trigger.entity) {
            return;
        }

        let (e_cell, OccupiedByUnit(&e_target)) = target_query.single().unwrap();
        let (e_unit, owns_hand) = unit_query.single().unwrap();

        let mut damage_buffer = enemy_query.get_mut(e_target).unwrap();
        damage_buffer.0.push(crate::damage::Damage {
            ty: DamageVariant::Bash,
            value: 5f32,
            source: Some(e_unit),
        });
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
