pub mod inventory_view;
pub mod player_items_view;
pub mod shop_view;

use glium::Frame;
use glium::Program;

use crate::Activation;
use crate::item::Item;
use crate::ui::*;
use crate::BlendMode;
use crate::draw_texture_wb;
use crate::TileSet;
use crate::item::ModKind;


pub fn show_item_popup(ui: &UI, target: &mut Frame,
                       x: i32, y: i32, item: &Item) 
{
    let font = &ui.context.font_14;
    
    let left = x + 6;
    let line_space = 20;
    let box_width = 200;
    let mut line_count = 1; // first line is item name

    for modifier in &item.mods {
        if modifier.max_value > 0 {
            line_count += 1;
        }
    }

    if item.activation != Activation::None {
        line_count += 1;
    }

    if item.description.len() > 0 {
        line_count += 
        ui.context.font_10.draw_multiline(&ui.display, target, &ui.program, 
                                          0, 0, box_width,
                                          &item.description, &OFF_WHITE, false);
    }

    let mut line = y - line_count * line_space;
    let bottom_margin = if line_count > 1 {8} else {4};

    ui.fill_box(target, x, line, box_width, (line_count * line_space) + bottom_margin, &[0.1, 0.1, 0.1, 0.9]);
    ui.draw_box(target, x, line, box_width, (line_count * line_space) + bottom_margin, &LIGHT_GREY);

    line += 5;

    let headline_width = font.calc_string_width(&item.name()) as i32;
    font.draw(&ui.display, target, &ui.program, x + (box_width - headline_width) / 2, line, &item.name(), &WHITE);

    line += 2;
    line += line_space;

    if item.activation != Activation::None {
        font.draw(&ui.display, target, &ui.program, left, line, 
                  "Activation: Fireball", &OFF_WHITE);
                  line += line_space;
    }

    for modifier in &item.mods {
        let text = modifier.assemble_mod_line_text();

        let color = if modifier.kind == ModKind::Implicit {&OFF_WHITE} else {&[0.6, 0.8, 1.0, 1.0]};

        font.draw(&ui.display, target, &ui.program, left, line, &text, color);
        line += line_space;
    }

    if item.description.len() > 0 {
        ui.context.font_10.draw_multiline(&ui.display, target, &ui.program, 
                                          left, line, box_width,
                                          &item.description, &OFF_WHITE, true);
        // line += line_space;
    }
}


pub fn draw_item(ui: &UI, target: &mut Frame, program: &Program,
                 entry_x: f32, entry_y: f32, 
                 slot_w: f32, slot_h: f32,
                 item: &Item,
                 item_tiles: &TileSet) 
{
    let item_inventory_w: f32 = (item.inventory_w * 32) as f32;
    let item_inventory_h: f32 = (item.inventory_h * 32) as f32;
    let inventory_scale: f32 = item.inventory_scale;

    // item stacks have several images.
    let mut image_id = item.inventory_tile_id;

    if item.stack_size > 1 {
        let offset = Item::calc_image_offset_for_stack_size(item.stack_size);
        image_id += offset;
    }

    let tile = item_tiles.tiles_by_id.get(&image_id).unwrap();

    let mut tw = tile.tex.width() as f32;
    let mut th = tile.tex.height() as f32;

    let s1 = item_inventory_w / tw;
    let s2 = item_inventory_h / th;

    let scale = if s1 < s2 { s1 } else { s2 };
    let scale = scale * 0.95 * inventory_scale;

    tw = tw * scale;
    th = th * scale;

    let origin_x = (slot_w - tw) / 2.0;
    let origin_y = (slot_h - th) / 2.0;

    let size = ui.context.window_size;

    draw_texture_wb(target, program, &ui.context.vertex_buffer, BlendMode::Blend,
                    size[0], size[1],
                    &tile.tex, 
                    entry_x + origin_x, entry_y + origin_y, scale, scale, &item.color);
}
