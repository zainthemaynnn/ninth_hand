use bevy::prelude::*;

pub fn wheel(radius: f32, icon_size: f32, icons: Vec<Handle<Image>>) -> impl Scene {
    let diameter = radius * 2.0;
    let sectors = icons.len();

    bsn! {
        Node {
            width: px(diameter),
            height: px(diameter),
            border_radius: BorderRadius::all(px(radius)),
        }
        BackgroundColor(Color::srgb(0.12, 0.12, 0.16))
        BorderColor::all(Color::WHITE)

        Children [
            {
                icons.into_iter().enumerate()
                    .map(|(sector, icon)| {
                        let angle = (sector as f32 + 0.5)
                            * std::f32::consts::TAU
                            / sectors as f32;
                        let image_center_radius = radius * 0.55;
                        let left = radius
                            + angle.sin() * image_center_radius
                            - icon_size * 0.5;
                        let top = radius
                            - angle.cos() * image_center_radius
                            - icon_size * 0.5;

                        bsn! {
                            Node {
                                position_type: PositionType::Absolute,
                                left: { px(left) },
                                top: { px(top) },
                                width: px(icon_size),
                                height: px(icon_size),
                                // border_radius: BorderRadius::all(px(icon_size * 0.5)),
                            }
                            UiTransform::from_rotation(Rot2::radians(
                                angle + std::f32::consts::PI,
                            ))
                            ImageNode { image: icon }
                            // BackgroundColor(Color::srgb(0.45, 0.45, 0.5))
                        }
                    })
                    .collect::<Vec<_>>()
            },
            {
                (0..sectors)
                    .map(|sector| {
                        let angle = sector as f32
                            * std::f32::consts::TAU
                            / sectors as f32;
                        let spoke_center_radius = radius * 0.5;
                        let left = radius + angle.sin() * spoke_center_radius - 1.0;
                        let top = radius - angle.cos() * spoke_center_radius - spoke_center_radius;

                        bsn! {
                            Node {
                                position_type: PositionType::Absolute,
                                left: { px(left) },
                                top: { px(top) },
                                width: px(2.0),
                                height: { px(radius) },
                            }
                            UiTransform::from_rotation(Rot2::radians(angle))
                            BackgroundColor(Color::WHITE)
                        }
                    })
                    .collect::<Vec<_>>()
            }
        ]
    }
}
