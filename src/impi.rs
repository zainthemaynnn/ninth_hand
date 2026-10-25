use bevy::{ecs::relationship::Relationship, platform::collections::HashMap, prelude::*};
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

pub struct ImpiPlugin;

impl Plugin for ImpiPlugin {
    fn build(&self, app: &mut App) {
        app.configure_loading_state(
            LoadingStateConfig::new(GameState::AssetLoading).load_collection::<ImpiAssets>(),
        );
    }
}

#[derive(Component, Default)]
#[require(
    Unit,
    UnitPriority(2),
    Player,
    UnitDesc {
        name: "Impi",
        passive: "???",
        flavor: "Fresh out of law school.",
    },
    UnitColor(Color::srgb(1.0, 0.0, 0.0)),
    Health(10.0),
    HealthCap(10.0),
    Resist(HashMap::from([
        (DamageVariant::Heat, 0.5),
        (DamageVariant::Radiant, 1.25),
        (DamageVariant::Abyssal, 1.25),
    ])),
)]
pub struct Impi;

#[derive(AssetCollection, Resource)]
pub struct ImpiAssets {
    #[asset(path = "gltf/impi.glb#Scene0")]
    pub impi: Handle<WorldAsset>,
    #[asset(path = "gltf/impi.glb#Animation0")]
    pub idle: Handle<AnimationClip>,
}

impl UnitModelResource for ImpiAssets {
    fn model_handle(&self) -> Handle<WorldAsset> {
        self.impi.clone()
    }
}

#[derive(Component)]
pub struct ImpiAnimations {
    pub idle: AnimationNodeIndex,
}

impl UnitAnimations for ImpiAnimations {
    type Assets = ImpiAssets;

    fn new(assets: &Self::Assets, graph: &mut AnimationGraph) -> Self {
        Self {
            idle: graph.add_clip(assets.idle.clone(), 1.0, graph.root),
        }
    }

    fn idle(&self) -> Option<Vec<AnimationNodeIndex>> {
        Some(vec![self.idle])
    }
}

impl UnitType for Impi {
    type ModelResource = ImpiAssets;
    type Animations = ImpiAnimations;
}

#[derive(Component)]
#[require(
    Action { name: "Fork", description: "Damage 3 front and behind" },
    Damage,
    FrontTarget,
    SingleTarget,
    LineOfSight,
    CardArt("abyss"),
)]
pub struct Fork;

impl ActionType for Fork {
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
                        ty: DamageVariant::Slash,
                        value: 2.0,
                        source: Some(e_unit),
                    },
                    target: e_enemy,
                });
            },
        )
    }
}
