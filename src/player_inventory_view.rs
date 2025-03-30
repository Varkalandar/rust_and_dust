use std::rc::Rc;
use std::collections::HashMap;

use glium::Texture2d;
use glium::Program;
use glium::Frame;
use glutin::surface::WindowSurface;
use glium::Display;

use crate::ui::{UiArea, UiFont, MouseButton, MouseMoveEvent, MouseState, ButtonEvent};
use crate::Inventory;
use crate::inventory::Slot;
use crate::inventory::Entry;
use crate::TileSet;
use crate::item::Item;
use crate::GameWorld;
use crate::sound::Sound;
use crate::ui::UI;
use crate::ui::Button;
use crate::ui::ButtonState;

use crate::gl_support::BlendMode;
use crate::gl_support::draw_texture;

pub struct PlayerInventoryView {
    area: UiArea,
    texture: Texture2d,
    item_tiles: TileSet,

    slot_offsets: HashMap<Slot, [i32; 2]>,
    slot_sizes: HashMap<Slot, [i32; 2]>,

    hover_item: Option<u64>,
    dragged_item: Option<u64>,
    drag_x: f32,
    drag_y: f32,

    font: Rc<UiFont>,
}


impl PlayerInventoryView {

    pub fn new(x: i32, y: i32, font: &Rc<UiFont>, tiles: &TileSet, texture: Texture2d) -> PlayerInventoryView {

        let mut slot_offsets = HashMap::new();
        slot_offsets.insert(Slot::Bag, [10, 452]);
        slot_offsets.insert(Slot::Body, [204, 213]);
        slot_offsets.insert(Slot::LWing, [400, 202]);
        slot_offsets.insert(Slot::RWing, [20, 205]);
        slot_offsets.insert(Slot::Engine, [214, 96]);

        let mut slot_sizes = HashMap::new();
        slot_sizes.insert(Slot::Bag, [15*32, 9*32]);
        slot_sizes.insert(Slot::LWing, [2*32, 3*32]);
        slot_sizes.insert(Slot::RWing, [2*32, 3*32]);
        slot_sizes.insert(Slot::Engine, [2*32, 3*32]);
        slot_sizes.insert(Slot::Body, [2*32, 3*32]);

        // let query = texture.query();

        PlayerInventoryView {
            area: UiArea {
                x, 
                y,
                w: 500,
                h: 750,                
            },
            
            texture,
            item_tiles: tiles.shallow_copy(),

            slot_offsets,
            slot_sizes,
            hover_item: None,
            dragged_item: None,
            drag_x: 0.0,
            drag_y: 0.0,
            font: font.clone(),
        }
    }


    fn find_slot_size(&self, item: &Item, slot: Slot) -> [i32; 2] {

        if slot == Slot::Bag {
            [item.inventory_w * 32, item.inventory_h * 32]
        }
        else {
            *self.slot_sizes.get(&slot).unwrap()
        }
    }


    fn show_item_popup(&self, 
                       ui: &UI, target: &mut Frame,
                       x: i32, y: i32, item: &Item) {

        let line_space = 20;

        let mut line_count = 1; // first line is item name

        for modifier in &item.mods {
            if modifier.max_value > 0 {
                line_count += 1;
            }
        }

        let mut line = y - line_count * line_space;

        ui.fill_box(target, x, line, 200, (line_count * line_space) + 8, &[0.1, 0.1, 0.1, 0.9]);
        ui.draw_box(target, x, line, 200, (line_count * line_space) + 8, &[0.6, 0.6, 0.6, 1.0]);
        
        // ui.draw_hline(target, x, line, 200, &[0.6, 0.6, 0.6, 1.0]);

        let left = x + 6;

        line += 5;

        let headline_width = self.font.calc_string_width(&item.name()) as i32;
        self.font.draw(&ui.display, target, &ui.program, x + (200 - headline_width) / 2, line, &item.name(), &[0.8, 1.0, 0.2, 1.0]);

        line += 2;
        line += line_space;

        for modifier in &item.mods {

            let min_value = modifier.min_value;
            let max_value = modifier.max_value;

            if max_value > 0 {
                let range = if min_value == max_value {
                    min_value.to_string()
                } else {
                    min_value.to_string() + "-" + &max_value.to_string()
                };

                let text = modifier.attribute.to_string() + ": " + &range;
                self.font.draw(&ui.display, target, &ui.program, left, line, &text, &[0.8, 0.8, 0.8, 1.0]);
                line += line_space;
            }
        }
    }


    fn find_slot_at(&self, mx: i32, my: i32) -> Option<Slot> {

        for key in self.slot_offsets.keys() {
            let offset = self.slot_offsets.get(key).unwrap();
            let size = self.slot_sizes.get(key).unwrap();
        
            if mx >= offset[0] && my >= offset[1] &&
               mx < offset[0] + size[0] && my < offset[1] + size[1] {
                return Some(*key);
            }
        }
 
        None
    }


    fn find_item_at(&self, inventory: &Inventory, mx: i32, my: i32) -> Option<u64> {
        let area = &self.area;

        for entry in &inventory.entries {
            if entry.slot != Slot::Stash && entry.slot != Slot::OnCursor {
                let offsets = self.slot_offsets.get(&entry.slot).unwrap();
                let entry_x = area.x + offsets[0] + entry.location_x * 32;
                let entry_y = area.y + offsets[1] + entry.location_y * 32;
                
                let item = inventory.bag.get(&entry.item_id).unwrap();
                let size = self.find_slot_size(item, entry.slot);

                if mx >= entry_x && my >= entry_y &&
                   mx < entry_x + size[0] && my < entry_y + size[1] {
                    // println!("Found {}", &item.name);
                    return Some(item.id);
                }
            }
        }
 
        None
    }


    fn draw_item(&self,
                 display: &Display<WindowSurface>, target: &mut Frame, program: &Program,
                 id: usize,
                 stack_size: u32, 
                 entry_x: f32, entry_y: f32, 
                 slot_w: f32, slot_h: f32,
                 item_inventory_w: f32 , item_inventory_h: f32,
                 inventory_scale: f32) {

        // item stacks have several images.
        let mut image_id = id;

        if stack_size > 1 {
            let offset = Item::calc_image_offset_for_stack_size(stack_size);
            image_id += offset;
        }

        let tile = self.item_tiles.tiles_by_id.get(&image_id).unwrap();

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

        draw_texture(&display, target, program, BlendMode::Blend, 
                     &tile.tex, 
                     entry_x + origin_x, entry_y + origin_y, scale, scale, &[1.0, 1.0, 1.0, 1.0]);
    }


    pub fn draw(&self, 
                ui: &UI, target: &mut Frame,
                x: i32, y: i32, inventory: &Inventory) {
        let area = &self.area;
        let xp = x + area.x;
        let yp = y + area.y;

        draw_texture(&ui.display, target, &ui.program, BlendMode::Blend, 
                     &self.texture, 
                     xp as f32, yp as f32, 1.0, 1.0, &[1.0, 1.0, 1.0, 0.95]);

        // show all items which are in the inventory space
        for entry in &inventory.entries {

            if entry.slot != Slot::Stash && entry.slot != Slot::OnCursor {
                let offsets = self.slot_offsets.get(&entry.slot).unwrap();
                let entry_x = (xp + offsets[0] + entry.location_x * 32) as f32;
                let entry_y = (yp + offsets[1] + entry.location_y * 32) as f32;
                
                let item = inventory.bag.get(&entry.item_id).unwrap();
                let size = self.find_slot_size(item, entry.slot);
                let w = size[0] as f32;
                let h = size[1] as f32;

                if self.hover_item == Some(item.id) {
                    draw_texture(&ui.display, target, &ui.program, BlendMode::Blend, 
                                 &ui.context.tex_white, 
                                 entry_x as f32 + 1.0, entry_y as f32 + 1.0, 
                                 (w - 2.0) / 16.0, 
                                 (h - 2.0) / 16.0, 
                                 &[0.2, 0.7, 0.0, 0.05]);
                }
                else {
                    draw_texture(&ui.display, target, &ui.program, BlendMode::Blend, 
                        &ui.context.tex_white, 
                        entry_x as f32 + 1.0, entry_y as f32 + 1.0, 
                        (w - 2.0) / 16.0, 
                        (h - 2.0) / 16.0, 
                        &[0.0, 0.02, 0.1, 0.7]);
                }

                self.draw_item(&ui.display, target, &ui.program,
                    item.inventory_tile_id, item.stack_size,
                    entry_x, entry_y, w, h, 
                    (item.inventory_w * 32) as f32, (item.inventory_h * 32) as f32,
                    item.inventory_scale);
            }
        }
       
        match self.hover_item {
            None => {},
            Some(id) => {
                let idx = inventory.find_entry_for_id(id).unwrap();
                let entry = &inventory.entries[idx];

                if self.dragged_item.is_none() && entry.slot != Slot::OnCursor {
                    let offsets = self.slot_offsets.get(&entry.slot).unwrap();
                    let item = inventory.bag.get(&id).unwrap();

                    let entry_x = xp + offsets[0] + entry.location_x * 32;
                    let entry_y = yp + offsets[1] + entry.location_y * 32;

                    self.show_item_popup(ui, target, entry_x - 4, entry_y, item);
                }
            }
        }

        match self.dragged_item {
            None => {},
            Some(id) => {
                let item = inventory.bag.get(&id).unwrap();

                self.draw_item(&ui.display, target, &ui.program,
                    item.inventory_tile_id, item.stack_size,
                    (self.drag_x - 16.0) as f32, (self.drag_y - 16.0) as f32, 
                    (item.inventory_w * 32) as f32, (item.inventory_h * 32) as f32, 
                    (item.inventory_w * 32) as f32, (item.inventory_h * 32) as f32,
                    item.inventory_scale);
            }
        }
    }


    pub fn handle_button_event(&mut self, event: &ButtonEvent, mouse: &MouseState, world: &mut GameWorld) -> bool {

        if event.args.state == ButtonState::Release &&
           event.args.button == Button::Mouse(MouseButton::Left) {

            match self.dragged_item {
                None => {
                    if self.hover_item.is_some() {
                        self.dragged_item = self.hover_item;
        
                        world.speaker.play(Sound::Click, 0.5);
                        println!("Started to drag item idx={:?} from {}, {}", self.dragged_item, event.mx, event.my);
                        
                        let item_id = self.dragged_item.unwrap();
                        let inventory = &mut world.player_inventory;
                        let idx = inventory.find_entry_for_id(item_id).unwrap();
                        let entry: &mut Entry = &mut inventory.entries[idx];
                        entry.slot = Slot::OnCursor;

                        return true;
                    }
                },
                Some(id) => {
                    let inventory = &mut world.player_inventory;
                    let item = inventory.bag.get(&id).unwrap();

                    world.speaker.play(Sound::Click, 0.5);

                    let idx = inventory.find_entry_for_id(id).unwrap();
                    let entry: &mut Entry = &mut inventory.entries[idx];

                    let mx = (mouse.position[0] as i32) - self.area.x;
                    let my = (mouse.position[1] as i32) - self.area.y;
                    
                    let slot_opt = self.find_slot_at(mx, my);

                    if slot_opt.is_some() {
                        let slot = slot_opt.unwrap();
                        entry.slot = slot;
                        self.dragged_item = None;

                        println!("Dropped an {} to slot {:?}", item.name(), slot);

                        if slot == Slot::Bag {
                            let offsets = self.slot_offsets.get(&Slot::Bag).unwrap();
                            let rel_x = mx - offsets[0];
                            let rel_y = my - offsets[1];
                            entry.location_x = rel_x / 32;
                            entry.location_y = rel_y / 32;
                        }
                        else {
                            entry.location_x = 0;
                            entry.location_y = 0;
                        }
    
                        return true;
                    }
                    else if mx < 0 { // dropped to the map floor?
                        self.dragged_item = None;
                        self.drop_item(world, id);
                    }
                    else {
                        println!("No suitable drop location {}, {}", mx, my);
                    }
                }
            }
        }

        false
    }


    pub fn drop_item(&mut self, world: &mut GameWorld, id: u64) {

        let inventory = &mut world.player_inventory;
        let item_opt = inventory.remove_item(id);

        match item_opt {
            None => {},
            Some(item) => {
                let position = world.map.get_player_position();

                println!("Dropping an {} to map floor at {}, {}", item.name(), position[0], position[1]);
        
                world.map.place_item(item, position);
            }
        }
    }


    pub fn handle_mouse_move_event(&mut self, event: &MouseMoveEvent, _mouse: &MouseState, inventory: &mut Inventory) -> bool {

        // println!("Mouse moved to {}, {}", event.mx, event.my);

        let item_opt = self.find_item_at(inventory, event.mx as i32, event.my as i32);
        self.hover_item = item_opt;

        self.drag_x = event.mx;
        self.drag_y = event.my;

        false
    }
}
