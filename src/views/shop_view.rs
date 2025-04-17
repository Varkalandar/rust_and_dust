use glium::Frame;
use glium::Texture2d;

use crate::shop::Shop;
use crate::Inventory;
use crate::GameWorld;
use crate::ButtonEvent;
use crate::MouseMoveEvent;
use crate::ui::UI;
use crate::ui::MouseState;
use crate::views::player_items_view::PlayerItemsView;
use crate::views::inventory_view::draw_item;
use crate::TileSet;


pub struct ShopView
{
    player_items_view: PlayerItemsView,

    pub shop_index: usize,     // the index of the shop in the current map to show
}


impl ShopView
{
    pub fn new(texture: Texture2d) -> ShopView 
    {
        ShopView 
        {
            player_items_view: PlayerItemsView::new(0, 0, texture),
            shop_index: 0,
        }
    }


    pub fn draw(&self, ui: &UI, target: &mut Frame, 
                shop: &Shop, player_inventory: &Inventory, item_tiles: &TileSet) 
    {
        let size = ui.context.window_size;
        let left = 40;
        let width = (size[0] as i32) - left * 2;
        let top = 10;
        let height = (size[1] as i32) - top * 2;

        ui.draw_box(target, left, top, width, height, &[0.6, 0.6, 0.6, 1.0]);
        ui.fill_box(target, left + 1, top + 1, width - 2, height -2 , &[0.3, 0.3, 0.3, 1.0]);

        self.player_items_view.draw(ui, target, left + 500, top, player_inventory, item_tiles);

        let font = &ui.context.font_14;

        font.draw(&ui.display, target, &ui.program, 
                  left + 10, top + 20, &shop.name, &[1.0, 1.0, 1.0, 1.0]);

        self.draw_shop_inventory(ui, target, shop, item_tiles);

        let text = "Shop under construction, no sales today.";
        let headline_width = font.calc_string_width(text) as i32;
        font.draw(&ui.display, target, &ui.program, 
                  left + (width - headline_width) / 2, top + height/2 - 20, text, &[1.0, 0.8, 0.2, 1.0]);
    }


    pub fn handle_button_event(&mut self, _event: &ButtonEvent, _mouse: &MouseState, _world: &mut GameWorld) -> bool 
    {
        false
    }


    pub fn handle_mouse_move_event(&mut self, _event: &MouseMoveEvent, _mouse: &MouseState, _player_inventory: &mut Inventory) -> bool 
    {
        false
    }


    fn draw_shop_inventory(&self, ui: &UI, target: &mut Frame, shop: &Shop, item_tiles: &TileSet)
    {
        let x = 60;
        let y = 60;
        
        for entry in &shop.inventory.entries {

            let item = shop.inventory.bag.get(&entry.item_id).unwrap();
            let entry_x = x + entry.location_x * 32;
            let entry_y = y + entry.location_y * 32;
            let w = item.inventory_w * 32;
            let h = item.inventory_h * 32;

            ui.draw_box(target, entry_x, entry_y, w, h, &[0.4, 0.5, 0.6, 1.0]);
            ui.fill_box(target, entry_x + 1, entry_y + 1, w - 2, h - 2, &[0.0, 0.02, 0.1, 0.7]);

            draw_item(ui, target, &ui.program,
                      entry_x as f32, entry_y as f32, 
                      w as f32, h as f32, 
                      item, item_tiles);
        }
    }
}


