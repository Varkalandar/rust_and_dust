use glium::Frame;
use glium::Texture2d;

use crate::shop::Shop;
use crate::item::Item;
use crate::Inventory;
use crate::GameWorld;
use crate::ButtonEvent;
use crate::MouseMoveEvent;
use crate::ui::UI;
use crate::ui::MouseState;
use crate::views::player_items_view::PlayerItemsView;
use crate::views::draw_item;
use crate::views::show_item_popup;
use crate::TileSet;


pub struct ShopView
{
    player_items_view: PlayerItemsView,
    shop_item_index: i32,      // the index of the shop item the mouse point is currently pointing at

    pub shop_index: usize,     // the index of the shop in the current map to show
}


impl ShopView
{
    pub fn new(texture: Texture2d) -> ShopView 
    {
        ShopView 
        {
            player_items_view: PlayerItemsView::new(70 + 560, 10, texture),
            shop_index: 0,
            shop_item_index: -1,
        }
    }


    pub fn draw(&self, ui: &UI, target: &mut Frame, 
                shop: &Shop, player_inventory: &Inventory, item_tiles: &TileSet) 
    {
        let size = ui.context.window_size;
        let left = 70;
        let width = (size[0] as i32) - left * 2;
        let top = 10;
        let height = (size[1] as i32) - top * 2;

        ui.draw_box(target, left, top, width, height, &[0.6, 0.6, 0.6, 1.0]);
        ui.fill_box(target, left + 1, top + 1, width - 2, height -2 , &[0.08, 0.06, 0.03, 1.0]);

        self.player_items_view.draw(ui, target, 0, 0, player_inventory, item_tiles);

        let font = &ui.context.font_24;

        font.draw(&ui.display, target, &ui.program, 
                  left + 10, top + 20, &shop.name, &[1.0, 1.0, 1.0, 1.0]);

        self.draw_shop_inventory(ui, target, shop, item_tiles, player_inventory.total_money());
/*
        let text = "Shop under construction, no sales today.";
        let headline_width = font.calc_string_width(text) as i32;
        font.draw(&ui.display, target, &ui.program, 
                  left + (width - headline_width) / 2, top + height/2 - 20, text, &[1.0, 0.8, 0.2, 1.0]);
                  */
    }


    pub fn handle_button_event(&mut self, event: &ButtonEvent, mouse_state: &MouseState, world: &mut GameWorld) -> bool 
    {
        // forward the event to the player item view
        self.player_items_view.handle_button_event(event, mouse_state, world)
    }


    pub fn handle_mouse_move_event(&mut self, event: &MouseMoveEvent, mouse_state: &MouseState, player_inventory: &mut Inventory) -> bool 
    {
        // these must match the display code
        let left = 84;
        let top = 60;

        let w = 106;
        let h = 96;

        let x = event.mx as i32 - left;
        let y = event.my as i32 - top;

        if x >= 0 && x < w * 5 && y >= 0 && y < h * 4 {
            self.shop_item_index = (y / h) * 5 + x / w;
        }
        else {
            self.shop_item_index = -1;
        }

        // forward the event to the player item view
        self.player_items_view.handle_mouse_move_event(event, mouse_state, player_inventory);

        false
    }


    fn draw_shop_inventory(&self, ui: &UI, target: &mut Frame, 
                          shop: &Shop, item_tiles: &TileSet, player_money: u32)
    {
        let font = &ui.context.font_14;
        let x = 84;
        let y = 60;
        
        let mut row = 0;
        let mut col = 0;

        let w = 106;
        let h = 96;

        for item in &shop.items {

            let entry_x = x + col * w;
            let entry_y = y + row * h;

            let back_color = if item.base_price <= player_money {[0.0, 0.02, 0.1, 0.7]} else {[0.12, 0.02, 0.0, 0.7]};

            ui.draw_box(target, entry_x, entry_y, w, h, &[0.4, 0.5, 0.6, 1.0]);
            ui.fill_box(target, entry_x + 1, entry_y + 1, w - 2, h - 2, &back_color);

            draw_item(ui, target, &ui.program,
                      entry_x as f32, entry_y as f32, 
                      w as f32, h as f32, 
                      item, item_tiles);

            let limit = 17;
            let name = item.name().to_string();
            let mut parts = name.split(" ");
            let mut line_y = 12 - (name.len() / limit) as i32 * 8;

            // try to assemble lines which are short enough for 64 pixels.
            // we make a rough guess of about 'limit' character to fit on such a line

            let mut text_line = parts.next().unwrap().to_string() + " ";

            loop {
                let mut word_opt = parts.next();
                
                if word_opt.is_some() {
                    let mut word = word_opt.unwrap();
    
                    // collect as many words as fit into "limit" characters, but not more
                    while text_line.len() + word.len() < limit {
                        text_line = text_line + word + " ";

                        word_opt = parts.next();
                        if word_opt.is_none() {
                            // no more words
                            word = "";
                            break;
                        }
                        word = word_opt.unwrap();
                    }
    
                    // keep in mind, there is now one word that isn't part of the line
                    // yet, display what we have assembled so far

                    // there is a space at the end of each line, we must subract one space width
                    let text_width = font.calc_string_width(&text_line) as i32 - 4;
                    font.draw(&ui.display, target, &ui.program, 
                              entry_x + (w - text_width) / 2, entry_y + line_y, &text_line, &[0.9, 0.9, 0.9, 1.0]);
        
                    line_y += font.lineheight;

                    // now start a new text line with the remaining word
                    text_line = word.to_string() + " ";
                }
                else {
                    
                    // draw the line as it is, even if it didn't reach "limit" characters
                    let text_width = font.calc_string_width(&text_line) as i32 - 4;
                    font.draw(&ui.display, target, &ui.program, 
                              entry_x + (w - text_width) / 2, entry_y + line_y, &text_line, &[0.9, 0.9, 0.9, 1.0]);
                    
                    // there are no more words to process.
                    break;
                }
            }

            // display the price at the bottom
            let text_line = calculate_price_string(item); // "100c";
            let text_width = font.calc_string_width(&text_line) as i32;
            font.draw(&ui.display, target, &ui.program, 
                      entry_x + (w - text_width) / 2, entry_y + h - 18, &text_line, &[1.0, 0.9, 0.5, 1.0]);


            col += 1;

            if col > 4 {
                col = 0;
                row += 1;
            }
        }

        // if the mouse was pointing at something in the shop inventory,
        // show the item details, too

        if self.shop_item_index >= 0 && (self.shop_item_index as usize) < shop.items.len() {
            // println!("Shop item index={}", self.shop_item_index);

            let item = &shop.items[self.shop_item_index as usize];
            let mx = ui.context.mouse_state.position[0] as i32;
            let my = ui.context.mouse_state.position[1] as i32;
            show_item_popup(ui, target, mx, my, item);
        }
    }
}


fn calculate_price_string(item: &Item) -> String
{
    let copper = item.base_price % 100;
    let silver = item.base_price / 100;
    
    if silver > 0 {
        if copper > 0 {
            return silver.to_string() + "s " + &copper.to_string() + "c";
        }
        else {
            return silver.to_string() + "s";
        }
    }

    return copper.to_string() + "c";
}