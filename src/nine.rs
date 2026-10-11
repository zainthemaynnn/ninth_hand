use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    actions::{Action, CardArt, InPlay, Target, tags::*},
    damage::{Damage, DamageBuffer, DamageVariant, Health, HealthCap, Resist},
    field::OccupiedByUnit,
    status::{Mark, Stacks},
    turn::Player,
    unit::{Unit, UnitDesc},
};

pub struct NinePlugin;

impl Plugin for NinePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(Crush::_play);
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
    fn _play(
        trigger: On<Add, InPlay>,
        crush_query: Query<(), With<Crush>>,
        target_query: Query<(Entity, &OccupiedByUnit), With<Target>>,
        mut enemy_query: Query<&mut DamageBuffer>,
        unit_query: Query<(Entity, &OwnsHand), With<Nine>>,
        _hand_query: Query<(&mut Transform,)>,
    ) {
        if !crush_query.contains(trigger.entity) {
            return;
        }

        let (_e_cell, occupied_by) = target_query.single().unwrap();
        let (e_unit, _owns_hand) = unit_query.single().unwrap();
        let e_target = occupied_by.iter().next().unwrap();

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
    Action { name: "Slap", description: "Damage 1 front, move to adjacent tile" },
    Damage,
    FrontTarget,
    SingleTarget,
    LineOfSight,
    MultiHit,
    CardArt("abyss"),
)]
pub struct Slap;

impl Slap {
    fn _play() {
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
    fn _play(
        trigger: On<Add, InPlay>,
        mut commands: Commands,
        action_query: Query<(), With<Gaze>>,
        target_query: Query<&OccupiedByUnit, With<Target>>,
        _unit_query: Query<Entity, With<Nine>>,
    ) {
        if !action_query.contains(trigger.entity) {
            return;
        }

        let occupied_by = target_query.single().unwrap();
        let e_target = occupied_by.iter().next().unwrap();
        commands
            .entity(e_target)
            .entry::<Stacks<Mark>>()
            .or_default()
            .and_modify(|mut stacks| stacks.add(1));
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
    fn _play() {
        todo!();
    }
}
