use std::{
    f32::consts::{FRAC_PI_3, FRAC_PI_6, TAU},
    time::Duration,
};

use bevy::{platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_tweening::{
    AnimCompletedEvent, AnimTarget, Delay, Tween, TweenAnim, lens::TransformPositionLens,
};

use crate::{
    actions::{Action, CardArt, Played, SecondaryTarget, Target, tags::*},
    asset::GameState,
    damage::{Damage, DamageEvent, DamageVariant, Health, HealthCap, KillingSomething, Resist},
    field::{Field, HexCoord, MovementEvent, OccupiedByUnit, OccupyingCell},
    status::{Mark, Stacks},
    systems::AnimatorEntity,
    turn::{Killed, Player},
    unit::{Unit, UnitDesc},
};

pub struct NinePlugin;

impl Plugin for NinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NineKillEffect>()
            .add_observer(Crush::play)
            .add_observer(Slap::play)
            .add_observer(Gaze::play)
            .add_observer(crate::systems::log_added_component::<AnimationPlayer>)
            .add_observer(kill_effect)
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

#[derive(Component, Debug, Clone)]
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
    fn play(
        _trigger: On<Played>,
        mut commands: Commands,
        _assets: Res<NineAssets>,
        _animation_graphs: ResMut<Assets<AnimationGraph>>,
        target_query: Query<(Entity, &GlobalTransform, &OccupiedByUnit), With<Target>>,
        unit_query: Query<(Entity, &OwnsHand), With<Nine>>,
        mut hand_query: Query<(
            Entity,
            &Hand,
            &mut Transform,
            &HandAnimationGraph,
            &AnimatorEntity,
        )>,
        mut animator_query: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    ) {
        let (_e_cell, target_tf, occupied_by) = target_query.single().unwrap();

        let e_enemy = occupied_by.get();
        let (e_unit, owns_hand) = unit_query.single().unwrap();
        let (e_hand, hand, hand_tf, graph, AnimatorEntity(e_animator)) =
            hand_query.get_mut(owns_hand.0).unwrap();

        let (mut animator, mut transitions) = animator_query.get_mut(*e_animator).unwrap();

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
        ));

        transitions.play(animator.as_mut(), graph.fist, Duration::from_millis(500));

        let target_tf = (*target_tf).clone();
        let hand = (*hand).clone();

        commands
            .spawn((
                TweenAnim::new(tween),
                AnimTarget::component::<Transform>(e_hand),
            ))
            .observe(
                move |trigger: On<AnimCompletedEvent>,
                      mut damage_events: MessageWriter<DamageEvent>,
                      mut commands: Commands| {
                    damage_events.write(DamageEvent {
                        damage: Damage {
                            ty: DamageVariant::Bash,
                            value: 2.0,
                            source: Some(e_unit),
                        },
                        target: e_enemy,
                    });

                    let tween = Delay::new(Duration::from_millis(500)).then(Tween::new(
                        EaseFunction::QuadraticOut,
                        Duration::from_secs(2),
                        TransformPositionLens {
                            start: target_tf.translation(),
                            end: hand.rest_position.translation,
                        },
                    ));

                    commands.spawn((
                        TweenAnim::new(tween),
                        AnimTarget::component::<Transform>(trigger.anim_entity),
                    ));
                    // transitions.play(animator.as_mut(), graph.idle, Duration::from_millis(500));
                },
            );
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
    fn play(
        _trigger: On<Played>,
        unit_query: Single<Entity, With<Nine>>,
        target_query: Single<(Entity, &OccupiedByUnit), With<Target>>,
        secondary_target_query: Single<Entity, With<SecondaryTarget>>,
        coord_query: Query<&HexCoord>,
        mut damage_events: MessageWriter<DamageEvent>,
        mut move_events: MessageWriter<MovementEvent>,
    ) {
        // man, all this querying is convoluted
        // perhaps there is an easier way to apply movement
        let e_unit = unit_query.into_inner();
        let (e_enemy_cell, occupied_by) = target_query.into_inner();
        let e_enemy = occupied_by.iter().next().unwrap();
        let e_knock_cell = secondary_target_query.into_inner();

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
    fn play(
        _trigger: On<Played>,
        mut commands: Commands,
        target_query: Single<&OccupiedByUnit, With<Target>>,
        _unit_query: Query<Entity, With<Nine>>,
    ) {
        let occupied_by = target_query.into_inner();
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

#[derive(Resource)]
pub struct NineKillEffect(pub Handle<EffectAsset>);

impl FromWorld for NineKillEffect {
    fn from_world(world: &mut World) -> Self {
        let mut effects = world.resource_mut::<Assets<EffectAsset>>();

        let writer = ExprWriter::new();

        let angle = writer.rand(ScalarType::Float) * writer.lit(TAU);

        let pi_div_3 = writer.lit(FRAC_PI_3);
        let pi_div_6 = writer.lit(FRAC_PI_6);

        let local_angle = (angle.clone() % pi_div_3) - pi_div_6;
        let edge_profile = local_angle.cos();

        let max_radius = writer.lit(0.5);
        let r = writer.rand(ScalarType::Float).sqrt() * (max_radius / edge_profile);

        let x = r.clone() * angle.clone().cos();
        let y = r * angle.sin();
        let z = writer.lit(-0.1);
        let position = x.vec3(y, z).expr();

        let init_pos = SetAttributeModifier::new(Attribute::POSITION, position);
        let init_velocity = SetAttributeModifier::new(
            Attribute::VELOCITY,
            writer.lit(Vec3::new(0.0, 0.0, -1.0)).expr(),
        );
        let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.0).expr());
        let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(1.0).expr());
        let init_color = SetAttributeModifier::new(
            Attribute::COLOR,
            writer.lit(LinearRgba::BLACK.as_u32()).expr(),
        );
        let init_size = SetAttributeModifier::new(Attribute::SIZE, writer.lit(0.1).expr());
        let gravity = AccelModifier::new(writer.lit(Vec3::new(0.0, 0.0, 0.1)).expr());

        let h_effect = effects.add(
            EffectAsset::new(4096, SpawnerSettings::rate(100.0.into()), writer.finish())
                .with_name("HexagonSpawner")
                .init(init_pos)
                .init(init_velocity)
                .init(init_age)
                .init(init_lifetime)
                .init(init_color)
                .init(init_size)
                .with_simulation_space(SimulationSpace::Local)
                .update(gravity),
        );

        Self(h_effect)
    }
}

pub fn kill_effect(
    trigger: On<KillingSomething>,
    mut commands: Commands,
    unit_query: Query<&OccupyingCell>,
    kill_effect: Res<NineKillEffect>,
) {
    info!("Activating kill effect");
    let occupying_cell = unit_query.get(trigger.killed).unwrap();
    let e_cell = occupying_cell.0;
    let e_effect = commands
        .spawn(ParticleEffect::new(kill_effect.0.clone()))
        .id();
    commands.entity(e_cell).add_child(e_effect);

    commands.entity(trigger.killed).observe(
        move |trigger: On<Killed>, mut commands: Commands, unit_query: Query<&Transform>| {
            info!("Killing");
            let unit_tf = unit_query.get(trigger.entity).unwrap();
            let tween = Tween::new(
                EaseFunction::QuinticIn,
                Duration::from_millis(1000),
                TransformPositionLens {
                    start: unit_tf.translation,
                    end: unit_tf.translation - Vec3::new(0.0, 8.0, 0.0),
                },
            );
            commands
                .entity(trigger.entity)
                .insert(TweenAnim::new(tween))
                .observe(
                    move |_trigger: On<AnimCompletedEvent>, mut commands: Commands| {
                        info!("killed");
                        commands.entity(e_effect).despawn();
                    },
                );
        },
    );
}
