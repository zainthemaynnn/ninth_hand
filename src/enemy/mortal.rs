use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::{
    actions::{
        action::{Action, ActionType, OwnedByUnit, Played},
        tags::{self, *},
        target::Target,
    },
    damage::{Damage, DamageEvent, DamageVariant, Health, HealthCap},
    enemy::ai::RandomAi,
    field::OccupiedByUnit,
    turn::Cpu,
    unit::{Unit, UnitAnimations, UnitAssetResource, UnitDesc, UnitType},
};

#[derive(AssetCollection, Resource)]
pub struct MortalAssets {
    #[asset(path = "gltf/enemy/mortal.glb#Scene0")]
    pub mortal: Handle<WorldAsset>,
    #[asset(path = "gltf/enemy/mortal.glb#Animation0")]
    pub idle: Handle<AnimationClip>,
}

#[derive(Component, Default)]
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
    RandomAi,
)]
pub struct Mortal;

impl UnitAssetResource for MortalAssets {
    fn model_handle(&self) -> Handle<WorldAsset> {
        self.mortal.clone()
    }
}

#[derive(Component)]
pub struct MortalAnimations {
    pub idle: AnimationNodeIndex,
}

impl UnitAnimations for MortalAnimations {
    type Assets = MortalAssets;

    fn new(assets: &Self::Assets, graph: &mut AnimationGraph) -> Self {
        Self {
            idle: graph.add_clip(assets.idle.clone(), 1.0, graph.root),
        }
    }

    fn idle(&self) -> Option<Vec<AnimationNodeIndex>> {
        Some(vec![self.idle])
    }
}

impl UnitType for Mortal {
    type ModelResource = MortalAssets;
    type Animations = MortalAnimations;
}

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
