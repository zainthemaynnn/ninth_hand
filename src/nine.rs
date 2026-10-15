use std::time::Duration;

use bevy::{platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_tweening::{AnimTarget, Delay, Tween, TweenAnim, lens::TransformPositionLens};

use crate::{
    actions::{Action, CardArt, InPlay, SecondaryTarget, Target, tags::*},
    asset::GameState,
    damage::{Damage, DamageBuffer, DamageEvent, DamageVariant, Health, HealthCap, Resist},
    field::{Field, HexCoord, MovementEvent, OccupiedByUnit, OccupyingCell},
    status::{Mark, Stacks},
    turn::Player,
    unit::{Unit, UnitDesc},
};

pub struct NinePlugin;

impl Plugin for NinePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(Crush::_play)
            .add_observer(Slap::_play)
            .add_observer(Gaze::_play)
            .add_observer(crate::systems::log_added_component::<AnimationPlayer>)
            .configure_loading_state(
                LoadingStateConfig::new(GameState::AssetLoading).load_collection::<NineAssets>(),
            );
    }
}

#[derive(AssetCollection, Resource)]
pub struct NineAssets {
    #[asset(path = "gltf/nine.glb#Scene0")]
    pub nine: Handle<WorldAsset>,
    #[asset(path = "gltf/hand.glb#Scene0")]
    pub hand: Handle<WorldAsset>,
    #[asset(path = "gltf/hand.glb#Animation1")]
    pub hand_float: Handle<AnimationClip>,
    #[asset(path = "gltf/hand.glb#Animation2")]
    pub hand_roll: Handle<AnimationClip>,
    #[asset(path = "gltf/hand.glb#Animation0")]
    pub hand_fist: Handle<AnimationClip>,
}

#[derive(Component)]
#[require(
    Unit,
    Player,
    UnitDesc {
        name: "Nine",
        passive: "When killing an enemy, gets an additional action.",
        flavor: "MUAHAHAHAHA",
    },
    Health(10.0),
    HealthCap(10.0),
    Resist(HashMap::from([
        (DamageVariant::Abyssal, 0.5),
        (DamageVariant::Radiant, 2.0),
    ])),
)]
pub struct Nine;

#[derive(Component)]
pub struct Hand {
    pub rest_position: Transform,
}

#[derive(Component, Debug)]
#[relationship(relationship_target = OwnsHand)]
pub struct HandOwnedBy(pub Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = HandOwnedBy)]
pub struct OwnsHand(Entity);

#[derive(Component)]
pub struct HandAnimationGraph {
    pub graph_handle: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub fist: AnimationNodeIndex,
}

pub fn spawn_at_location(
    In(coord): In<HexCoord>,
    mut commands: Commands,
    assets: Res<NineAssets>,
    field: Res<Field<Player>>,
) -> Result<Entity> {
    let Some(e_cell) = field.cells.get(&coord) else {
        let err = BevyError::new(
            Severity::Error,
            format!("coord {:?} not found in field cells", coord),
        );
        return Err(err);
    };

    Ok(commands
        .spawn((
            WorldAssetRoot(assets.nine.clone()),
            Nine,
            OccupyingCell(*e_cell),
            Health(1.0),
        ))
        .id())
}

pub fn spawn_with_hand(
    In(e_nine_result): In<Result<Entity>>,
    mut commands: Commands,
    assets: Res<NineAssets>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let Ok(e_nine) = e_nine_result else {
        return;
    };

    let mut graph = AnimationGraph::new();
    let idle = graph.add_clip(assets.hand_float.clone(), 1.0, graph.root);
    let fist = graph.add_clip(assets.hand_fist.clone(), 1.0, graph.root);
    commands.spawn((
        Hand {
            rest_position: Transform::default(),
        },
        WorldAssetRoot(assets.hand.clone()),
        HandOwnedBy(e_nine),
        HandAnimationGraph {
            graph_handle: graphs.add(graph),
            idle,
            fist,
        },
    ));
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
    fn _play(
        trigger: On<Add, InPlay>,
        mut commands: Commands,
        _assets: Res<NineAssets>,
        _animation_graphs: ResMut<Assets<AnimationGraph>>,
        children: Query<&Children>,
        crush_query: Query<(), With<Crush>>,
        target_query: Query<(Entity, &GlobalTransform, &OccupiedByUnit), With<Target>>,
        mut enemy_query: Query<&mut DamageBuffer>,
        unit_query: Query<(Entity, &OwnsHand), With<Nine>>,
        mut hand_query: Query<(Entity, &Hand, &mut Transform, &HandAnimationGraph)>,
        mut animator_query: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    ) {
        if !crush_query.contains(trigger.entity) {
            return;
        }

        let (_e_cell, target_tf, occupied_by) = target_query.single().unwrap();
        let (e_unit, owns_hand) = unit_query.single().unwrap();
        let (e_hand, hand, hand_tf, graph) = hand_query.get_mut(owns_hand.0).unwrap();

        let Some(e_animator) = children
            .iter_descendants(e_hand)
            .find(|&e_child| animator_query.contains(e_child))
        else {
            warn!("failed to locate animator");
            return;
        };

        let (mut animator, mut transitions) = animator_query.get_mut(e_animator).unwrap();

        let mut damage_buffer = enemy_query.get_mut(occupied_by.get()).unwrap();
        damage_buffer.0.push(crate::damage::Damage {
            ty: DamageVariant::Bash,
            value: 5f32,
            source: Some(e_unit),
        });

        let tween = Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_secs(2),
            TransformPositionLens {
                start: hand_tf.translation,
                end: target_tf.translation() + Vec3::Y * 8.0,
            },
        )
        .then(Delay::new(Duration::from_millis(500)))
        .then(Tween::new(
            EaseFunction::CubicOut,
            Duration::from_millis(200),
            TransformPositionLens {
                start: target_tf.translation() + Vec3::Y * 8.0,
                end: target_tf.translation(),
            },
        ))
        .then(Delay::new(Duration::from_millis(500)))
        .then(Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_secs(2),
            TransformPositionLens {
                start: target_tf.translation(),
                end: hand.rest_position.translation,
            },
        ));

        transitions.play(animator.as_mut(), graph.fist, Duration::from_millis(500));

        commands.spawn((
            TweenAnim::new(tween),
            AnimTarget::component::<Transform>(e_hand),
        ));
        // .observe(|trigger: On<AnimCompletedEvent>| {
        //     transitions.play(animator.as_mut(), graph.idle, Duration::from_millis(500));
        // });
    }
}

#[derive(Component)]
#[require(
    Action { name: "Slap", description: "Damage 2 front, move to adjacent tile" },
    Damage,
    FrontTarget,
    SingleTarget,
    LineOfSight,
    MultiHit,
    CardArt("abyss"),
)]
pub struct Slap;

impl Slap {
    fn _play(
        trigger: On<Add, InPlay>,
        unit_query: Query<Entity, With<Nine>>,
        action_query: Query<(), With<Slap>>,
        target_query: Query<(Entity, &OccupiedByUnit), With<Target>>,
        secondary_target_query: Query<Entity, With<SecondaryTarget>>,
        coord_query: Query<&HexCoord>,
        mut damage_events: MessageWriter<DamageEvent>,
        mut move_events: MessageWriter<MovementEvent>,
    ) {
        if !action_query.contains(trigger.entity) {
            return;
        }

        // man, all this querying is convoluted
        // perhaps there is an easier way to apply movement
        let e_unit = unit_query.single().unwrap();
        let (e_enemy_cell, occupied_by) = target_query.single().unwrap();
        let e_enemy = occupied_by.iter().next().unwrap();
        let e_knock_cell = secondary_target_query.single().unwrap();

        let attack_src = coord_query.get(e_enemy_cell).unwrap();
        let attack_dst = coord_query.get(e_knock_cell).unwrap();

        damage_events.write(DamageEvent {
            damage: Damage {
                ty: DamageVariant::Bash,
                value: 2.0,
                source: Some(e_unit),
            },
            target: e_enemy,
        });

        move_events.write(MovementEvent {
            direction: HexCoord::new(attack_dst.q - attack_src.q, attack_dst.r - attack_src.r),
            target: e_enemy,
        });
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
