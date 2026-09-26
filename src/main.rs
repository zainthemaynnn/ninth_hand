#![feature(more_float_constants)]
use bevy::{color::palettes::css::SILVER, prelude::*};
use bevy_asset_loader::prelude::*;

use crate::{
    field::{Field, FieldPlugin, HexCoord, OccupyingCell, load_field},
    unit::{Unit, UnitPlugin},
};

pub mod actions;
pub mod damage;
pub mod field;
pub mod intents;
pub mod orthogon;
pub mod unit;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameAssetLoadState {
    #[default]
    AssetLoading,
    Next,
}

#[derive(AssetCollection, Resource)]
struct NineAssets {
    #[asset(path = "gltf/nine.glb")]
    nine: Handle<Gltf>,
    #[asset(path = "gltf/symmetra.glb")]
    symmetra: Handle<Gltf>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FieldPlugin)
        .add_plugins(UnitPlugin)
        .init_state::<GameAssetLoadState>()
        .add_loading_state(
            LoadingState::new(GameAssetLoadState::AssetLoading)
                .continue_to_state(GameAssetLoadState::Next)
                .load_collection::<NineAssets>(),
        )
        .add_systems(
            OnEnter(GameAssetLoadState::Next),
            (
                (|| {
                    vec![
                        HexCoord::new(-2, 1),
                        HexCoord::new(-2, 2),
                        HexCoord::new(-2, 3),
                        HexCoord::new(-2, 4),
                        HexCoord::new(-1, 1),
                        HexCoord::new(-1, 2),
                        HexCoord::new(-1, 3),
                        HexCoord::new(0, 0),
                        HexCoord::new(0, 1),
                        HexCoord::new(0, 2),
                        HexCoord::new(0, 3),
                        HexCoord::new(1, 0),
                        HexCoord::new(1, 1),
                        HexCoord::new(1, 2),
                        HexCoord::new(2, -1),
                        HexCoord::new(2, 0),
                        HexCoord::new(2, 1),
                        HexCoord::new(2, 2),
                    ]
                })
                .pipe(load_field),
                entry,
            )
                .chain(),
        )
        .run();
}

fn entry(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    gltfs: Res<Assets<Gltf>>,
    nine_assets: Res<NineAssets>,
    field: Res<Field>,
) {
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
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    ));

    let gltf = gltfs.get(&nine_assets.nine).unwrap();
    let h_scene = gltf.default_scene.as_ref().unwrap().clone();
    let e_cell = field.cells.get(&HexCoord::new(0, 0)).unwrap();
    commands.spawn((WorldAssetRoot(h_scene), Unit, OccupyingCell(*e_cell)));
}
