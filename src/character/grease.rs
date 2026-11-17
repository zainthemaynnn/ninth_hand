use bevy::{platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;

use crate::{
    actions::{
        Action, ActionType, OwnedByUnit, Played, Target,
        tags::{FrontTarget, LineOfSight, SingleTarget},
    },
    asset::GameState,
    damage::{Damage, DamageEvent, DamageVariant, Health, HealthCap, Resist},
    field::OccupiedByUnit,
    turn::Player,
    unit::{Unit, UnitAnimations, UnitColor, UnitDesc, UnitAssetResource, UnitPriority, UnitType},
};

pub struct GreasePlugin;

impl Plugin for GreasePlugin {
    fn build(&self, app: &mut App) {
        app.configure_loading_state(
            LoadingStateConfig::new(GameState::AssetLoading).load_collection::<GreaseAssets>(),
        );
    }
}

#[derive(Component, Default)]
#[require(
    Unit,
    UnitPriority(6),
    Player,
    UnitDesc {
        name: "Grease",
        passive: "Damage due to status or field effects is redirected to a random enemy.",
        flavor: "I just can't do anything right... Blegh...",
    },
    UnitColor(Srgba::hex("#00FF1A").unwrap().into()),
    Health(10.0),
    HealthCap(10.0),
    Resist(HashMap::from([
        (DamageVariant::Bash, 1.5),
        (DamageVariant::Slash, 0.5),
        (DamageVariant::Heat, 0.8),
        (DamageVariant::Cold, 0.8),
    ])),
)]
pub struct Grease;

#[derive(AssetCollection, Resource)]
pub struct GreaseAssets {
    #[asset(path = "gltf/grease.glb#Scene0")]
    pub grease: Handle<WorldAsset>,
    #[asset(path = "gltf/grease.glb#Animation0")]
    pub idle: Handle<AnimationClip>,
}

impl UnitAssetResource for GreaseAssets {
    fn model_handle(&self) -> Handle<WorldAsset> {
        self.grease.clone()
    }
}

#[derive(Component)]
pub struct GreaseAnimations {
    pub idle: AnimationNodeIndex,
}

impl UnitAnimations for GreaseAnimations {
    type Assets = GreaseAssets;

    fn new(assets: &Self::Assets, graph: &mut AnimationGraph) -> Self {
        Self {
            idle: graph.add_clip(assets.idle.clone(), 1.0, graph.root),
        }
    }

    fn idle(&self) -> Option<Vec<AnimationNodeIndex>> {
        Some(vec![self.idle])
    }
}

impl UnitType for Grease {
    type ModelResource = GreaseAssets;
    type Animations = GreaseAnimations;
}

#[derive(Component)]
#[require(
    Action { name: "Slime", description: "Damage 2 front, Slow" },
    Damage,
    FrontTarget,
    SingleTarget,
    LineOfSight,
)]
pub struct Slime;

impl ActionType for Slime {
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
                        ty: DamageVariant::Plague,
                        value: 2.0,
                        source: Some(e_unit),
                    },
                    target: e_enemy,
                });
            },
        )
    }
}
