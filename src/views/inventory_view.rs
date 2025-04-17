use glium::Frame;

use crate::Slot;
use crate::Inventory;
use crate::UI;
use crate::TileSet;
use crate::views::draw_item;


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


