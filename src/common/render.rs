use crate::common::character::{Character, Renderable};
use crate::common::coords::{Area, Position, SquareArea};
use crate::common::enemies::enemy::Enemy;
use crate::common::pickups::PickupTypes;
use crate::common::rogue::{Layer, Rogue};
use crate::common::utils::get_mut_item_in_2d_enum_vec;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Span, Text};

#[must_use]
pub fn spans_to_text(spans: Vec<Vec<Span<'_>>>) -> Text<'_> {
    let map = spans;

    let out: Text<'_> = map
        .into_iter()
        .map(|style_line| Line::default().spans(style_line))
        .collect();

    out
}

#[must_use]
pub fn get_camera_area(content_area: Rect, player_pos: &Position, layer: &Layer) -> SquareArea {
    let view_height = i32::from(content_area.height);
    let view_width = i32::from(content_area.width);

    let layer_height = layer.len() as i32;
    let layer_width = layer[0].len() as i32;

    let (player_x, player_y) = player_pos.get();

    // Center the camera on the player
    let mut x1 = player_x - view_width / 2;
    let mut y1 = player_y - view_height / 2;
    let mut x2 = x1 + view_width;
    let mut y2 = y1 + view_height;

    // Clamp to the left edge
    if x1 < 0 {
        x1 = 0;
        x2 = view_width;
    }

    // Clamp to top edge
    if y1 < 0 {
        y1 = 0;
        y2 = view_height;
    }

    // Clamp to right edge
    if x2 > layer_width {
        x2 = layer_width;
        x1 = (layer_width - view_width).max(0);
    }

    // Clamp to bottom edge
    if y2 > layer_height {
        y2 = layer_height;
        y1 = (layer_height - view_height).max(0);
    }

    SquareArea {
        corner1: Position(x1, y1),
        corner2: Position(x2, y2),
    }
}

#[must_use]
pub fn flatten_to_span(rogue: &Rogue, area: Option<SquareArea>) -> Vec<Vec<Span<'static>>> {
    fn callback_creator<F: std::borrow::Borrow<T>, T: Renderable>(
        enum_2d: &mut Vec<(usize, Vec<(usize, Span)>)>,
        layer: &Layer,
    ) -> impl FnMut(F) {
        |entity: F| {
            let mut pos = entity.borrow().get_pos().clone();
            pos.constrain(layer);

            if let Some(entity_pos) = get_mut_item_in_2d_enum_vec(enum_2d, &pos) {
                *entity_pos = entity.borrow().get_entity_char().to_styled();
            }
        }
    }

    let (x1, y1, x2, y2);
    if let Some(inner_area) = area {
        (x1, y1, x2, y2) = inner_area.get_bounds();
    } else {
        (x1, y1, x2, y2) = (
            0,
            0,
            rogue.layer_base[0].len() as i32 - 1,
            rogue.layer_base.len() as i32 - 1,
        );
    }

    let mut enum_2d: Vec<(usize, Vec<(usize, Span<'static>)>)> = rogue
        .layer_base
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
            if i >= y1 as usize && i <= y2 as usize {
                Some((
                    i,
                    line.iter()
                        .enumerate()
                        .filter_map(|(i, entity)| {
                            if i >= x1 as usize && i <= x2 as usize {
                                Some((i, entity.to_styled()))
                            } else {
                                None
                            }
                        })
                        .collect(),
                ))
            } else {
                None
            }
        })
        .collect();

    rogue
        .pickup_wrangler
        .pickups
        .iter()
        .for_each(callback_creator::<_, PickupTypes>(
            &mut enum_2d,
            &rogue.layer_base,
        ));

    rogue
        .enemies
        .borrow()
        .iter()
        .for_each(callback_creator::<_, Enemy>(
            &mut enum_2d,
            &rogue.layer_base,
        ));

    rogue.active_damage_effects.iter().for_each(|effect| {
        effect
            .get_instructions()
            .for_each(callback_creator(&mut enum_2d, &rogue.layer_base));
    });

    {
        let mut character_callback =
            callback_creator::<_, Character>(&mut enum_2d, &rogue.layer_base);
        character_callback(&rogue.character);
    }

    enum_2d
        .into_iter()
        .map(|(_, vec): (usize, Vec<(usize, Span)>)| {
            vec.into_iter()
                .map(|(_, item): (usize, Span)| item)
                .collect()
        })
        .collect()
}
