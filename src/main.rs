#![feature(more_float_constants)]
#![feature(vec_try_remove)]
#![feature(const_trait_impl)]

use bevy::{
    color::palettes::css::SILVER,
    gltf::{GltfPlugin, convert_coordinates::GltfConvertCoordinates},
    prelude::*,
};
use bevy_hanabi::prelude::*;
use bevy_tweening::TweeningPlugin;

use crate::{
    actions::{ActionPlugin, ActionType, OwnedByUnit},
    asset::{GameState, NineAssetPlugin},
    damage::DamagePlugin,
    director::DirectorPlugin,
    enemy::{EnemyAssets, EnemyPlugin, mortal::Mortal},
    field::{Field, FieldPlugin, HexCoord, OccupyingCell, load_field},
    impi::{Impi, ImpiPlugin},
    nine::{Nine, NinePlugin},
    status::StatusPlugin,
    symmetra::{Symmetra, SymmetraPlugin},
    turn::{Cpu, Player, TurnPlugin},
    ui::NineUiPlugin,
    unit::UnitPlugin,
};

pub mod actions;
pub mod asset;
pub mod damage;
pub mod director;
pub mod enemy;
pub mod field;
pub mod impi;
pub mod nine;
pub mod status;
pub mod symmetra;
pub mod systems;
pub mod turn;
pub mod ui;
pub mod unit;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(GltfPlugin {
                convert_coordinates: GltfConvertCoordinates {
                    rotate_scene_entity: true,
                    rotate_meshes: false,
                },
                ..default()
            }),
            HanabiPlugin,
            NineAssetPlugin,
            FieldPlugin,
            UnitPlugin,
            ActionPlugin,
            TweeningPlugin,
            MeshPickingPlugin,
            TurnPlugin,
            NineUiPlugin,
            DamagePlugin,
            EnemyPlugin,
            StatusPlugin,
            DirectorPlugin,
        ))
        .add_plugins((NinePlugin, SymmetraPlugin, ImpiPlugin))
        .insert_resource(MeshPickingSettings {
            require_markers: true,
            ray_cast_visibility: RayCastVisibility::Any,
        })
        .add_systems(
            OnEnter(GameState::InGame),
            (
                (|| {
                    (
                        Transform::from_xyz(0.0, 0.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
                        field_coords(),
                    )
                })
                .pipe(load_field::<Player>),
                (|| {
                    (
                        Transform::from_xyz(0.0, 0.0, -20.0).looking_at(Vec3::ZERO, Vec3::Y),
                        field_coords(),
                    )
                })
                .pipe(load_field::<Cpu>),
                (|| HexCoord::new(0, 0))
                    // ??? why do I need to annotate this at all?
                    .pipe::<_, _, Result<Entity>, _>(crate::unit::spawn_at_location::<Nine>)
                    .pipe(nine::spawn_with_hand),
                (|| HexCoord::new(2, -1))
                    .pipe::<_, _, Result<Entity>, _>(crate::unit::spawn_at_location::<Symmetra>)
                    .pipe(|e_symmetra: In<Result<Entity>>, mut commands: Commands| {
                        commands
                            .entity(*e_symmetra.as_ref().unwrap())
                            .observe(symmetra::setup_symmetra_model);
                    }),
                (|| HexCoord::new(1, -1))
                    .pipe::<_, _, Result<Entity>, _>(crate::unit::spawn_at_location::<Impi>)
                    .pipe(|_in: In<Result<Entity>>| {}),
                entry.before(actions::set_action_bar),
            )
                .chain(),
        )
        .run();
}

fn field_coords() -> Vec<HexCoord> {
    vec![
        HexCoord::new(-3, 1),
        HexCoord::new(-3, 0),
        HexCoord::new(-3, -1),
        HexCoord::new(-3, -2),
        HexCoord::new(-2, 1),
        HexCoord::new(-2, 0),
        HexCoord::new(-2, -1),
        HexCoord::new(-2, -2),
        HexCoord::new(-1, 0),
        HexCoord::new(-1, -1),
        HexCoord::new(-1, -2),
        HexCoord::new(-1, -3),
        HexCoord::new(0, 0),
        HexCoord::new(0, -1),
        HexCoord::new(0, -2),
        HexCoord::new(0, -3),
        HexCoord::new(1, -1),
        HexCoord::new(1, -2),
        HexCoord::new(1, -3),
        HexCoord::new(1, -4),
        HexCoord::new(2, -1),
        HexCoord::new(2, -2),
        HexCoord::new(2, -3),
        HexCoord::new(2, -4),
        HexCoord::new(3, -2),
        HexCoord::new(3, -3),
        HexCoord::new(3, -4),
        HexCoord::new(3, -5),
    ]
}

fn entry(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    enemy_assets: Res<EnemyAssets>,
    cpu_field: Res<Field<Cpu>>,
    nine_query: Single<Entity, With<Nine>>,
    symmetra_query: Single<Entity, With<Symmetra>>,
    impi_query: Single<Entity, With<Impi>>,
) {
    info!("entry");

    commands.spawn((
        PointLight {
            shadow_maps_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
        Transform::from_translation(Vec3::new(0.0, -1.0, 0.0)),
    ));

    commands.spawn((
        Camera3d::default(),
        MeshPickingCamera,
        Transform::from_xyz(24.0, 24.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let e_cell = cpu_field.cells.get(&HexCoord::new(1, -1)).unwrap();
    let e_mortal = commands
        .spawn((
            WorldAssetRoot(enemy_assets.mortal.clone()),
            Mortal,
            OccupyingCell(*e_cell),
        ))
        .id();

    let e_cell2 = cpu_field.cells.get(&HexCoord::new(0, -3)).unwrap();
    let e_mortal2 = commands
        .spawn((
            WorldAssetRoot(enemy_assets.mortal.clone()),
            Mortal,
            OccupyingCell(*e_cell2),
        ))
        .id();

    let e_nine = nine_query.into_inner();

    commands
        .spawn((crate::enemy::mortal::Punch, OwnedByUnit(e_mortal)))
        .with_children(|parent| {
            parent.spawn(crate::enemy::mortal::Punch::play().with_entity(parent.target_entity()));
        });

    commands
        .spawn((crate::enemy::mortal::Punch, OwnedByUnit(e_mortal2)))
        .with_children(|parent| {
            parent.spawn(crate::enemy::mortal::Punch::play().with_entity(parent.target_entity()));
        });

    commands
        .spawn((crate::nine::Crush, OwnedByUnit(e_nine)))
        .with_children(|parent| {
            parent.spawn(crate::nine::Crush::play().with_entity(parent.target_entity()));
        });
    commands
        .spawn((crate::nine::Slap, OwnedByUnit(e_nine)))
        .with_children(|parent| {
            parent.spawn(crate::nine::Slap::play().with_entity(parent.target_entity()));
        });

    let e_symmetra = symmetra_query.into_inner();

    commands
        .spawn((crate::symmetra::SummonRedGlyph, OwnedByUnit(e_symmetra)))
        .with_children(|parent| {
            parent
                .spawn(crate::symmetra::SummonRedGlyph::play().with_entity(parent.target_entity()));
        });

    commands
        .spawn((crate::symmetra::SummonBlueGlyph, OwnedByUnit(e_symmetra)))
        .with_children(|parent| {
            parent.spawn(
                crate::symmetra::SummonBlueGlyph::play().with_entity(parent.target_entity()),
            );
        });

    commands
        .spawn((crate::symmetra::SummonGreenGlyph, OwnedByUnit(e_symmetra)))
        .with_children(|parent| {
            parent.spawn(
                crate::symmetra::SummonGreenGlyph::play().with_entity(parent.target_entity()),
            );
        });

    let e_impi = impi_query.into_inner();

    commands
        .spawn((crate::impi::Fork, OwnedByUnit(e_impi)))
        .with_children(|parent| {
            parent.spawn(crate::impi::Fork::play().with_entity(parent.target_entity()));
        });
}
