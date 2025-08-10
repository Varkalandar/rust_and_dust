use glium::Frame;
use glium::Texture2d;
use glium::Program;
use glutin::surface::WindowSurface;
use glium::Display;

use crate::shop::Shop;
use crate::item::Item;
use crate::item::ItemKind;
use crate::ItemFactory;
use crate::Inventory;
use crate::Slot;
use crate::GameWorld;
use crate::ButtonEvent;
use crate::MouseMoveEvent;
use crate::ui::*;
use crate::views::player_items_view::PlayerItemsView;
use crate::views::draw_item;
use crate::views::show_item_popup;
use crate::TileSet;

const SHOP_ITEMS_TOP: i32 = 60;
const SHOP_ITEMS_LEFT: i32 = 80;
const SHOP_TAB_HEIGHT: i32 = 28;

pub struct ShopView
{
    player_items_view: PlayerItemsView,
    shop_item_index: Option<usize>,      // the index of the shop item the mouse point is currently pointing at

    shop_index: usize,     // the index of the shop in the current map to show

    click_areas: Vec<UiArea>,
}


impl ShopView
{
    pub fn new(texture: Texture2d) -> ShopView 
    {
        ShopView 
        {
            player_items_view: PlayerItemsView::new(70 + 560, 10, texture),
            shop_index: 0,
            shop_item_index: None,
            click_areas: Vec::new(),
        }
    }


    pub fn get_shop_index(&self) -> usize
    {
        self.shop_index
    } 


    pub fn set_shop_index(&mut self, shop_index: usize)
    {
        self.shop_index = shop_index;
        self.player_items_view.drop_shop = Some(shop_index);
    }


    pub fn draw(&mut self, ui: &UI, target: &mut Frame, 
                shop: &Shop, player_inventory: &Inventory, item_tiles: &TileSet) 
    {
        let area = calc_view_area(ui.context.window_size);

        ui.draw_box(target, area.x, area.y, area.w, area.h, &[0.6, 0.6, 0.6, 1.0]);
        ui.fill_box(target, area.x + 1, area.y + 1, area.w - 2, area.h - 2 , &[0.08, 0.06, 0.03, 1.0]);

        self.player_items_view.draw(ui, target, player_inventory, item_tiles);

        let font = &ui.context.font_large;

        font.draw(&ui.display, target, &ui.program, 
                  area.x + 10, area.y + 20, &shop.name, &WHITE);

        self.draw_shop_inventory(ui, target, shop, item_tiles, player_inventory.total_money());

        let text = "Drop items here to sell.";
        font.draw(&ui.display, target, &ui.program, 
                  area.x + 130, area.y + 570, text, &ORANGE);
    }


    pub fn handle_button_event(&mut self, ui: &UI, event: &ButtonEvent, world: &mut GameWorld) 
        -> (bool, bool) 
    {
        let shop = &mut world.map.shops[self.shop_index];

        // did the player click a shop inventory tab?

        let mut index = 0;
        for area in &self.click_areas {
            if area.contains(event.mx as i32, event.my as i32) {
                shop.active_tab = index;
                return (true, false);
            }
            index += 1;
        }

        // no tab -> did the player click a shop item?
        let current_filter = self.get_current_filter(shop);
        let item_index = find_item_at(shop, current_filter, event.mx, event.my);

        if item_index.is_some() && item_index.unwrap() < shop.items.len() &&
            shop.items[item_index.unwrap()].calc_price() <= world.player_inventory.total_money() {
            buy_item_from_shop(item_index.unwrap(), shop, 
                               &mut self.player_items_view, &mut world.player_inventory,
                               &mut world.map.item_factory);
        }

        // forward the event to the player item view
        self.player_items_view.handle_button_event(event, &ui.context.mouse_state, world)
    }


    pub fn handle_mouse_move_event(&mut self, event: &MouseMoveEvent, mouse_state: &MouseState, world: &mut GameWorld) -> bool 
    {
        let shop = &mut world.map.shops[self.shop_index];
        let current_filter = self.get_current_filter(shop);

        self.shop_item_index = find_item_at(shop, current_filter, event.mx, event.my);

        // forward the event to the player item view
        self.player_items_view.handle_mouse_move_event(event, mouse_state, &mut world.player_inventory);

        false
    }


    fn draw_shop_inventory(&mut self, ui: &UI, target: &mut Frame, 
                           shop: &Shop, item_tiles: &TileSet, player_money: u32)
    {
        let font = &ui.context.font_small;
        let mut x = SHOP_ITEMS_LEFT;
        let y = SHOP_ITEMS_TOP;
        
        let mut col = 0;

        self.click_areas.clear();

        for tab in &shop.tabs {
            let w = font.calc_string_width(&tab) as i32 + 8;
            let y_off = if col == shop.active_tab {0} else {2};

            ui.draw_box(target, x, y+y_off, w, SHOP_TAB_HEIGHT-y_off, &[0.4, 0.5, 0.6, 1.0]);
            ui.fill_box(target, x + 1, y+y_off + 1, w - 2, SHOP_TAB_HEIGHT-y_off - 2, 
                        if col == shop.active_tab {&[0.2, 0.2, 0.2, 1.0]} else {&[0.1, 0.1, 0.1, 1.0]});

            font.draw(&ui.display, target, &ui.program,
                      x + 4, y + 6 + y_off/2, &tab, &[1.0, 0.9, 0.5, 1.0]);

            self.click_areas.push(UiArea::new(x, y+y_off, w, SHOP_TAB_HEIGHT-y_off, col));
            x += w;
            col += 1;
        }

        let mut row = 0;
        let mut col = 0;

        let x = SHOP_ITEMS_LEFT;
        let y = SHOP_ITEMS_TOP + SHOP_TAB_HEIGHT;
        let w = 108;
        let h = 96;

        ui.draw_box(target, x, y, w*5, h*4, &[0.4, 0.5, 0.6, 1.0]);
        ui.fill_box(target, x + 1, y + 1, w*5 - 2, h*4 - 2, &[0.1, 0.1, 0.1, 1.0]);

        let current_filter = self.get_current_filter(shop);

        for item in shop.items.iter().filter(|item| current_filter(*item)) {

            let entry_x = x + col * w;
            let entry_y = y + row * h;

            let back_color = if item.base_price <= player_money {[0.0, 0.02, 0.1, 1.0]} else {[0.1, 0.01, 0.0, 0.7]};

            ui.draw_box(target, entry_x, entry_y, w, h, &[0.4, 0.5, 0.6, 1.0]);
            ui.fill_box(target, entry_x + 1, entry_y + 1, w - 2, h - 2, &back_color);

            draw_item(ui, target, &ui.program,
                      entry_x as f32, entry_y as f32, 
                      w as f32, h as f32, 
                      item, item_tiles);

            let limit = 16;
            let mut name = item.name().to_string();

            if item.show_type {
                name = name + " " + item.kind.name_str()
            }

            draw_multiline_centered(&ui.display, target, &ui.program, 
                                    &name, entry_x, entry_y, w, limit, font);

            // display the price at the bottom
            let text_line = calculate_price_string(item);
            font.draw_centered(&ui.display, target, &ui.program, 
                               entry_x, entry_y + h - 18, w, &text_line, &[1.0, 0.9, 0.5, 1.0]);

            col += 1;

            if col > 4 {
                col = 0;
                row += 1;
            }
        }

        // if the mouse was pointing at something in the shop inventory,
        // show the item details, too

        if self.shop_item_index.is_some() && self.shop_item_index.unwrap() < shop.items.len() {
            // println!("Shop item index={}", self.shop_item_index);

            let item = &shop.items[self.shop_item_index.unwrap()];
            let mx = ui.context.mouse_state.position[0] as i32;
            let my = ui.context.mouse_state.position[1] as i32;
            show_item_popup(ui, target, mx, my, item);
        }
    }

    fn get_current_filter(&self, shop: &Shop) -> fn(&Item) -> bool
    {
        if shop.active_tab == 0 {
           return wand_staff_filter;
        }
        else if shop.active_tab == 1 {
           return jewelry_filter;
        }
        else if shop.active_tab == 2 {
           return all_items_filter;
        }
        else {
            return all_items_filter;
        }
    }
}


fn draw_multiline_centered(display: &Display<WindowSurface>, target: &mut Frame, program: &Program,
                           name: &String, x: i32, y:i32,
                           box_width: i32, limit: usize, font: &UiFont)
{
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
            font.draw(display, target, program, 
                      x + (box_width - text_width) / 2, y + line_y, &text_line, &[0.9, 0.9, 0.9, 1.0]);

            line_y += font.line_height;

            // now start a new text line with the remaining word
            text_line = word.to_string() + " ";
        }
        else {
            
            // draw the line as it is, even if it didn't reach "limit" characters
            let text_width = font.calc_string_width(&text_line) as i32 - 4;
            font.draw(display, target, program, 
                      x + (box_width - text_width) / 2, y + line_y, &text_line, &[0.9, 0.9, 0.9, 1.0]);
            
            // there are no more words to process.
            break;
        }
    }
}


fn calculate_price_string(item: &Item) -> String
{
    let price = item.calc_price();
    let copper = price % 100;
    let silver = price / 100;
    
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


fn find_item_at(shop: &Shop, filter: fn(&Item) -> bool, mx: f32, my: f32) -> Option<usize>
{
    let left = SHOP_ITEMS_LEFT;
    let top = SHOP_ITEMS_TOP + SHOP_TAB_HEIGHT;

    let w = 108;
    let h = 96;

    let x = mx as i32 - left;
    let y = my as i32 - top;

    if x >= 0 && x < w * 5 && y >= 0 && y < h * 4 {
        let location_item_index = ((y / h) * 5 + x / w) as usize;

        // there could be a filter on the ui, so we need to find the actual
        // index of that item in the shops inventory.

        let mut index = 0;
        let mut filter_index = 0;
        for item in &shop.items {

            if filter(item) {
                if filter_index == location_item_index {
                    return Some(index);
                }
                filter_index += 1;
            }

            index += 1;
        }

    }

    None
}


fn buy_item_from_shop(item_index: usize, shop: &mut Shop, 
                      piv: &mut PlayerItemsView, inventory: &mut Inventory,
                      item_factory: &mut ItemFactory)
{
    let item = shop.items.remove(item_index);
    piv.hover_item = Some(item.id);
    inventory.withdraw_money(item.calc_price(), item_factory);
    inventory.put_item(item, Slot::OnCursor);
}
 

fn calc_view_area(window_size: [u32; 2]) -> UiArea
{
    let left = 70;
    let width = (window_size[0] as i32) - left * 2;
    let top = 10;
    let height = (window_size[1] as i32) - top * 2;

    UiArea::new(left, top, width, height, 0)
}


fn wand_staff_filter(item: &Item) -> bool
{
    return item.kind == ItemKind::Wand || item.kind == ItemKind::Staff;
}


fn jewelry_filter(item: &Item) -> bool
{
    return item.kind == ItemKind::Ring || item.kind == ItemKind::Amulet;
}


fn all_items_filter(item: &Item) -> bool
{
    return true;
}