use bevy::{platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;

use crate::{
    actions::{
        action::{Action, ActionType, OwnedByUnit, Played},
        tags::{FrontTarget, LineOfSight, SingleTarget},
        target::Target,
    },
    asset::GameState,
    damage::{Damage, DamageEvent, DamageVariant, Health, HealthCap, Resist},
    field::OccupiedByUnit,
    turn::Player,
    unit::{Unit, UnitAnimations, UnitAssetResource, UnitColor, UnitDesc, UnitPriority, UnitType},
};

pub struct AnimusPlugin;

impl Plugin for AnimusPlugin {
    fn build(&self, app: &mut App) {
        app.configure_loading_state(
            LoadingStateConfig::new(GameState::AssetLoading).load_collection::<AnimusAssets>(),
        );
    }
}

#[derive(Component, Default)]
#[require(
    Unit,
    UnitPriority(3),
    Player,
    UnitDesc {
        name: "Animus",
        passive: "No knockback. If adjacent to a construct, it doesn't deteriorate.",
        flavor: "HATE. ORGANICS.",
    },
    UnitColor(Srgba::hex("#CBC275").unwrap().into()),
    Health(10.0),
    HealthCap(10.0),
    Resist(HashMap::from([
        (DamageVariant::Bash, 1.5),
        (DamageVariant::Slash, 0.5),
        (DamageVariant::Heat, 0.8),
        (DamageVariant::Cold, 0.8),
    ])),
)]
pub struct Animus;

#[derive(AssetCollection, Resource)]
pub struct AnimusAssets {
    #[asset(path = "gltf/animus.glb#Scene0")]
    pub animus: Handle<WorldAsset>,
    #[asset(path = "gltf/animus.glb#Animation0")]
    pub idle: Handle<AnimationClip>,
    #[asset(path = "img/font/strange-marks-font/StrangeMarks-ovKMa.ttf")]
    pub font: Handle<Font>,
}

impl UnitAssetResource for AnimusAssets {
    fn model_handle(&self) -> Handle<WorldAsset> {
        self.animus.clone()
    }

    fn font_handle(&self) -> Option<Handle<Font>> {
        Some(self.font.clone())
    }
}

#[derive(Component)]
pub struct AnimusAnimations {
    pub idle: AnimationNodeIndex,
}

impl UnitAnimations for AnimusAnimations {
    type Assets = AnimusAssets;

    fn new(assets: &Self::Assets, graph: &mut AnimationGraph) -> Self {
        Self {
            idle: graph.add_clip(assets.idle.clone(), 1.0, graph.root),
        }
    }

    fn idle(&self) -> Option<Vec<AnimationNodeIndex>> {
        Some(vec![self.idle])
    }
}

impl UnitType for Animus {
    type ModelResource = AnimusAssets;
    type Animations = AnimusAnimations;
}

#[derive(Component)]
#[require(
    Action { name: "Smash", description: "Damage 4 front" },
    Damage,
    FrontTarget,
    SingleTarget,
    LineOfSight,
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
                        ty: DamageVariant::Bash,
                        value: 4.0,
                        source: Some(e_unit),
                    },
                    target: e_enemy,
                });
            },
        )
    }
}
