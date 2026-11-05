use bevy::{platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;

use crate::{
    actions::{
        Action, ActionType, CardArt, OwnedByUnit, Played, Target,
        tags::{FrontTarget, LineOfSight, SingleTarget},
    },
    asset::GameState,
    damage::{Damage, DamageEvent, DamageVariant, Health, HealthCap, Resist},
    field::OccupiedByUnit,
    turn::Player,
    unit::{Unit, UnitAnimations, UnitColor, UnitDesc, UnitModelResource, UnitPriority, UnitType},
};

pub struct MaryPlugin;

impl Plugin for MaryPlugin {
    fn build(&self, app: &mut App) {
        app.configure_loading_state(
            LoadingStateConfig::new(GameState::AssetLoading).load_collection::<MaryAssets>(),
        );
    }
}

#[derive(Component, Default)]
#[require(
    Unit,
    UnitPriority(4),
    Player,
    UnitDesc {
        name: "Mary",
        passive: "Starts at 50% HP. Every turn loses 10% HP.",
        flavor: "The air will smell of the mortals' flesh burning in hellfire.",
    },
    UnitColor(Srgba::hex("#0900FF").unwrap().into()),
    Health(5.0),
    HealthCap(10.0),
    Resist(HashMap::from([
        (DamageVariant::Bash, 1.5),
        (DamageVariant::Slash, 0.5),
        (DamageVariant::Heat, 0.8),
        (DamageVariant::Cold, 0.8),
    ])),
)]
pub struct Mary;

#[derive(AssetCollection, Resource)]
pub struct MaryAssets {
    #[asset(path = "gltf/mary.glb#Scene0")]
    pub mary: Handle<WorldAsset>,
    #[asset(path = "gltf/mary.glb#Animation0")]
    pub idle: Handle<AnimationClip>,
}

impl UnitModelResource for MaryAssets {
    fn model_handle(&self) -> Handle<WorldAsset> {
        self.mary.clone()
    }
}

#[derive(Component)]
pub struct MaryAnimations {
    pub idle: AnimationNodeIndex,
}

impl UnitAnimations for MaryAnimations {
    type Assets = MaryAssets;

    fn new(assets: &Self::Assets, graph: &mut AnimationGraph) -> Self {
        Self {
            idle: graph.add_clip(assets.idle.clone(), 1.0, graph.root),
        }
    }

    fn idle(&self) -> Option<Vec<AnimationNodeIndex>> {
        Some(vec![self.idle])
    }
}

impl UnitType for Mary {
    type ModelResource = MaryAssets;
    type Animations = MaryAnimations;
}

#[derive(Component)]
#[require(
    Action { name: "Slime", description: "Damage 2 front, Slow" },
    Damage,
    FrontTarget,
    SingleTarget,
    LineOfSight,
    CardArt("abyss"),
)]
pub struct Smash;

impl ActionType for Smash {
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
