use crate::BlendMode;
use crate::draw_texture_wb;
use crate::item::Item;
use crate::Slot;
use crate::Display;
use crate::Inventory;
use crate::UI;
use crate::TileSet;

use glutin::surface::WindowSurface;
use glium::Program;
use glium::Frame;


pub struct InventoryView
{

}


impl InventoryView
{
    pub fn new() -> InventoryView
    {
        InventoryView {

        }
    }


    pub fn draw(&self, ui: &UI, target: &mut Frame, x: i32, y: i32, 
                inventory: &Inventory, tiles: &TileSet) 
    {
        // show all items which are in the inventory space
        for entry in &inventory.entries {

            if entry.slot == Slot::Bag {
                let item = inventory.bag.get(&entry.item_id).unwrap();
                let entry_x = x + entry.location_x * 32;
                let entry_y = y + entry.location_y * 32;
                let w = item.inventory_w * 32;
                let h = item.inventory_h * 32;

                ui.fill_box(target, entry_x + 1, entry_y + 1, w - 2, h - 2, &[0.0, 0.02, 0.1, 0.7]);

                draw_item(ui, target, &ui.program,
                          entry_x as f32, entry_y as f32, 
                          w as f32, h as f32, 
                          item, tiles);
            }
        }
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
