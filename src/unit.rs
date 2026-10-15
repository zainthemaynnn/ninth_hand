use bevy::{color::palettes, math::VectorSpace, prelude::*};

use crate::{
    asset::GameState,
    damage::{DamageAssets, DamageBuffer, DamageVariant, Health, HealthCap, Resist},
    field::{Field, HexCoord, HoveredCell, OccupiedByUnit, OccupyingCell, SelectedCell},
    systems::enforce_one,
    turn::{Cpu, Player, Team},
};

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_unit_details_window)
            .add_systems(Update, (reposition::<Player>, reposition::<Cpu>))
            .add_observer(show_unit_details)
            .add_observer(hide_unit_details)
            .add_observer(show_unit_health)
            .add_observer(show_unit_resist)
            .add_observer(show_unit_text)
            .add_observer(select_unit)
            .add_observer(enforce_one::<HoveredUnit>);
    }
}

#[derive(Component, Default)]
#[require(
    Health,
    Resist,
    DamageBuffer,
    Transform,
    OccupyingCell(Entity::PLACEHOLDER)
)]
pub struct Unit;

#[derive(Component)]
pub struct UnitDesc {
    pub name: &'static str,
    pub passive: &'static str,
    pub flavor: &'static str,
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct HoveredUnit;

#[derive(Resource)]
pub struct ActiveUnits(pub Vec<Entity>);

pub fn reposition<T: Team + Component>(
    field: Res<Field<T>>,
    q_moved_units: Query<(&mut Transform, &OccupyingCell), (Changed<OccupyingCell>, With<T>)>,
    q_hexcoord: Query<&HexCoord>,
) {
    for (mut tf, OccupyingCell(e_cell)) in q_moved_units {
        let cell_hexcoord = q_hexcoord.get(*e_cell).unwrap();
        *tf = field.to_world_space(*cell_hexcoord);
        info!("unit moved to {:?}", cell_hexcoord);
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct UnitDetailsWindow;

#[derive(Component, Debug, Clone, FromTemplate)]
pub struct HealthBar;

#[derive(Component, Debug, Clone, Default)]
pub struct UnitResistDetailsVariant(pub DamageVariant);

#[derive(Component, Debug, Clone, FromTemplate)]
pub struct UnitName;

#[derive(Component, Debug, Clone, FromTemplate)]
pub struct UnitDescription;

#[derive(Component, Debug, Clone, FromTemplate)]
pub struct UnitFlavor;

pub fn spawn_unit_details_window(mut commands: Commands, damage_assets: Res<DamageAssets>) {
    #[rustfmt::skip]
    let resist_value_l = &DamageVariant::VALUES[0..2];
    let resist_value_m = &DamageVariant::VALUES[2..5];
    let resist_value_r = &DamageVariant::VALUES[5..7];
    let resist_values: Vec<Box<&[DamageVariant]>> = vec![
        Box::new(resist_value_r),
        Box::new(resist_value_m),
        Box::new(resist_value_l),
    ];

    commands.spawn_scene(bsn! {
        #MainWindow
        UnitDetailsWindow
        Visibility::Hidden
        Node {
            position_type: PositionType::Absolute,
            top: Val::ZERO,
            right: Val::ZERO,
            margin: UiRect::all(Val::Px(8.0)),
            padding: UiRect::all(Val::Px(8.0)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            height: Val::Px(160.0),
            column_gap: Val::Px(8.0),
        }
        BackgroundColor({ palettes::basic::BLACK.with_alpha(0.5) })

        Children [
            // (
            //     #Photo
            //     Node {
            //         height: Val::Auto,
            //         aspect_ratio: { Some(1.0) },
            //         align_self: AlignSelf::Stretch,
            //     }
            //     ImageNode::default()
            // ),

            (
                #ResistValues
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::RowReverse,
                    align_items: AlignItems::Center,
                }
                Children [
                    {
                        resist_values.iter().map(|resist_values_col| {
                            bsn! {
                                Node {
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Column,
                                }
                                Children [
                                    {
                                        resist_values_col.iter().map(|damage_ty| {
                                            bsn! {
                                                Node {
                                                    display: Display::Flex,
                                                    flex_direction: FlexDirection::Row,
                                                    padding: UiRect::all(Val::Px(4.0)),
                                                }
                                                Children [
                                                    (
                                                        Node {
                                                            height: Val::Px(28.0),
                                                            aspect_ratio: { Some(1.0) },
                                                        }
                                                        ImageNode {
                                                            image: { damage_assets.icon_from_damage_variant(*damage_ty).unwrap().clone() },
                                                            color: { damage_ty.color() },
                                                        }
                                                    ),
                                                    (
                                                        UnitResistDetailsVariant({*damage_ty})
                                                        Text::new("1.0x")
                                                        TextColor
                                                    )
                                                ]
                                            }
                                        }).collect::<Vec<_>>()
                                    }
                                ]
                            }
                        }).collect::<Vec<_>>()
                    }
                ]
            ),

            (
                #Text
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    width: Val::Px(400.0),
                }
                Children [
                    (
                        #Name
                        UnitName
                        Text::new("Mortal")
                        TextFont { font_size: { FontSize::Px(24.0) } }
                    ),
                    (
                        #UnitDescription
                        UnitDescription
                        Text::new("A useless mortal.")
                    ),
                    (
                        #UnitFlavor
                        UnitFlavor
                        Text::new("\"I'm just a guy.\"")
                        TextColor({ palettes::css::GRAY })
                        TextFont { font_size: { FontSize::Px(16.0) } }
                    )
                ]
            ),

            (
                #HealthBar
                HealthBar
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(100.0),
                    width: Val::Percent(100.0),
                    left: Val::ZERO,
                    height: Val::Px(10.0),
                }
                BackgroundColor(Color::from(palettes::basic::RED))
            ),
        ]
    });
}

pub fn show_unit_details(
    trigger: On<Add, HoveredUnit>,
    mut ui_query: Query<&mut Visibility, With<UnitDetailsWindow>>,
) {
    let mut visibility = ui_query.single_mut().unwrap();
    *visibility = Visibility::Visible;
}

pub fn hide_unit_details(
    trigger: On<Remove, HoveredUnit>,
    mut ui_query: Query<&mut Visibility, With<UnitDetailsWindow>>,
) {
    let mut visibility = ui_query.single_mut().unwrap();
    *visibility = Visibility::Hidden;
}

pub fn show_unit_health(
    trigger: On<Add, HoveredUnit>,
    unit_query: Query<(&Health, &HealthCap)>,
    mut health_bar_query: Query<&mut Node, With<HealthBar>>,
) {
    let (health, health_cap) = unit_query.get(trigger.entity).unwrap();

    let mut health_bar_node = health_bar_query.single_mut().unwrap();
    health_bar_node.width = Val::Percent(health.0 / health_cap.0 * 100.0);
}

pub fn show_unit_resist(
    trigger: On<Add, HoveredUnit>,
    mut commands: Commands,
    unit_query: Query<&Resist>,
    mut resist_query: Query<(&mut Text, &mut TextColor, &UnitResistDetailsVariant)>,
) {
    let resist = unit_query.get(trigger.entity).unwrap();

    for (mut text, mut color, damage_ty) in resist_query {
        let resist_value = resist.0.get(&damage_ty.0).unwrap_or(&1.0f32);
        text.0 = format!("{0:.1}x", resist_value);
        if *resist_value < 1.0 {
            color.0 = palettes::css::GRAY
                .lerp(palettes::css::WHITE, *resist_value)
                .into();
        } else {
            color.0 = palettes::css::WHITE
                .lerp(palettes::css::RED, f32::min(resist_value - 1.0, 1.0))
                .into();
        }
    }
}

pub fn show_unit_text(
    trigger: On<Add, HoveredUnit>,
    unit_query: Query<&UnitDesc>,
    mut text_query: ParamSet<(
        Query<&mut Text, With<UnitName>>,
        Query<&mut Text, With<UnitDescription>>,
        Query<&mut Text, With<UnitFlavor>>,
    )>,
) {
    let desc = unit_query.get(trigger.entity).unwrap();
    text_query.p0().single_mut().unwrap().0 = desc.name.to_owned();
    text_query.p1().single_mut().unwrap().0 = desc.passive.to_owned();
    text_query.p2().single_mut().unwrap().0 = format!("\"{0}\"", desc.flavor.to_owned());
}

pub fn select_unit(
    trigger: On<Add, HoveredCell>,
    mut commands: Commands,
    cell_query: Query<Option<&OccupiedByUnit>>,
    selected_query: Query<Entity, With<HoveredUnit>>,
) {
    if let Some(e_unit) = cell_query.get(trigger.entity).unwrap() {
        // lol?? how do you actually get the entity in here?
        commands
            .entity(e_unit.iter().next().unwrap())
            .insert(HoveredUnit);
    } else {
        if let Ok(e_unit) = selected_query.single() {
            commands.entity(e_unit).remove::<HoveredUnit>();
        }
    }
}
