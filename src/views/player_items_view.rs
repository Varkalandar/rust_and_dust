use std::collections::HashMap;

use glium::Texture2d;
use glium::Frame;
use glutin::surface::WindowSurface;

use crate::ui::{UiArea, MouseButton, MouseMoveEvent, MouseState, ButtonEvent};
use crate::Inventory;
use crate::inventory::Slot;
use crate::inventory::Entry;
use crate::TileSet;
use crate::item::Item;
use crate::item::Activation;
use crate::item::DropEffect;

use crate::GameWorld;
use crate::sound::Sound;
use crate::ui::UI;
use crate::ui::Button;
use crate::ui::ButtonState;
use crate::views::inventory_view::InventoryView;
use crate::views::draw_item;
use crate::views::show_item_popup;

use crate::gl_support::BlendMode;
use crate::gl_support::draw_texture;


pub struct PlayerItemsView {
    area: UiArea,
    texture: Texture2d,

    slot_offsets: HashMap<Slot, [i32; 2]>,
    slot_sizes: HashMap<Slot, [i32; 2]>,
    inventory_view: InventoryView,

    hover_item: Option<u64>,
    dragged_item: Option<u64>,
    drag_x: f32,
    drag_y: f32,
}


impl PlayerItemsView {

    pub fn new(x: i32, y: i32, texture: Texture2d) -> PlayerItemsView {

        let mut slot_offsets = HashMap::new();
        slot_offsets.insert(Slot::Bag, [10, 452]);
        slot_offsets.insert(Slot::Head, [107, 63]);
        slot_offsets.insert(Slot::Body, [314, 311]);
        slot_offsets.insert(Slot::LHand, [400, 202]);
        slot_offsets.insert(Slot::RHand, [20, 205]);
        slot_offsets.insert(Slot::Amulet, [230, 52]);
        slot_offsets.insert(Slot::LRing, [367, 150]);
        slot_offsets.insert(Slot::RRing, [80, 154]);

        let mut slot_sizes = HashMap::new();
        slot_sizes.insert(Slot::Bag, [15*32, 9*32]);
        slot_sizes.insert(Slot::Head, [2*32, 2*32]);
        slot_sizes.insert(Slot::Body, [2*32, 3*32]);
        slot_sizes.insert(Slot::LHand, [2*32, 3*32]);
        slot_sizes.insert(Slot::RHand, [2*32, 3*32]);
        slot_sizes.insert(Slot::Amulet, [1*32, 1*32]);
        slot_sizes.insert(Slot::LRing, [1*32, 1*32]);
        slot_sizes.insert(Slot::RRing, [1*32, 1*32]);

        PlayerItemsView {
            area: UiArea {
                x, 
                y,
                w: 500,
                h: 750,                
            },
            
            texture,

            slot_offsets,
            slot_sizes,
            inventory_view: InventoryView::new(),
            hover_item: None,
            dragged_item: None,
            drag_x: 0.0,
            drag_y: 0.0,
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


    /**
     * @return the id of the found item or None
     */
    fn find_item_at(&self, inventory: &Inventory, mx: i32, my: i32) -> Option<u64> {
        let area = &self.area;

        for entry in &inventory.entries {
            if entry.slot != Slot::Stash && entry.slot != Slot::OnCursor {
                let offsets = self.slot_offsets.get(&entry.slot).unwrap();
                let entry_x = area.x + offsets[0] + entry.location_x * 32;
                let entry_y = area.y + offsets[1] + entry.location_y * 32;
                
                let item = inventory.bag.get(&entry.item_id).unwrap();
                let size = self.find_slot_size(item, entry.slot);

                // println!("Checking {}, {} vs entry {}, {}, {}, {}", mx, my, entry_x, entry_y, size[0], size[1]);

                if mx >= entry_x && my >= entry_y &&
                   mx < entry_x + size[0] && my < entry_y + size[1] {
                    // println!("Found {}", &item.name);
                    return Some(item.id);
                }
            }
        }
 
        None
    }


    pub fn draw(&self, ui: &UI, target: &mut Frame,
                x: i32, y: i32, inventory: &Inventory,
                item_tiles: &TileSet)
    {
        let area = &self.area;
        let xp = x + area.x;
        let yp = y + area.y;

        draw_texture(&ui.display, target, &ui.program, BlendMode::Blend, 
                     &self.texture, 
                     xp as f32, yp as f32, 1.0, 1.0, &[1.0, 1.0, 1.0, 0.95]);

        // show all items which are in the inventory space
        for entry in &inventory.entries {

            if entry.slot != Slot::Stash && entry.slot != Slot::OnCursor && entry.slot != Slot::Bag {
                let offsets = self.slot_offsets.get(&entry.slot).unwrap();
                let entry_x = xp + offsets[0] + entry.location_x * 32;
                let entry_y = yp + offsets[1] + entry.location_y * 32;
                
                let item = inventory.bag.get(&entry.item_id).unwrap();
                let size = self.find_slot_size(item, entry.slot);
                let w = size[0];
                let h = size[1];

                if self.hover_item == Some(item.id) {
                    ui.fill_box(target,
                                entry_x + 1, entry_y + 1, w - 2, h - 2, &[0.2, 0.7, 0.0, 0.05]);
                }
                else {
                    ui.fill_box(target,
                                entry_x + 1, entry_y + 1, w - 2, h - 2, &[0.0, 0.02, 0.1, 0.7]);
                }

                draw_item(ui, target, &ui.program,
                          entry_x as f32, entry_y as f32, w as f32, h as f32, 
                          item, item_tiles);
            }
        }

        let ipos = self.slot_offsets.get(&Slot::Bag).unwrap();
        self.inventory_view.draw(ui, target, xp + ipos[0], yp + ipos[1], 
                                 inventory, item_tiles);
       
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

                    show_item_popup(ui, target, entry_x - 4, entry_y, item);
                }
            }
        }

        match self.dragged_item {
            None => {},
            Some(id) => {
                let item = inventory.bag.get(&id).unwrap();

                draw_item(ui, target, &ui.program,
                          (self.drag_x - 16.0) as f32, (self.drag_y - 16.0) as f32, 
                          (item.inventory_w * 32) as f32, (item.inventory_h * 32) as f32,
                          item, item_tiles);
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

                    world.speaker.play(Sound::Click, 0.5);

                    let mx = (mouse.position[0] as i32) - self.area.x;
                    let my = (mouse.position[1] as i32) - self.area.y;
                    
                    let slot_opt = self.find_slot_at(mx, my);

                    match slot_opt {
                        None => {
                            if mx < 0 { // dropped to the map floor?
                                self.dragged_item = None;
                                self.drop_item(world, id);
        
                                return true;
                            }
                            else {
                                println!("No suitable drop location {}, {}", mx, my);
                            }
                        },
                        Some(slot) => {
                            let inventory = &mut world.player_inventory;
                            self.drop_item_to_slot(inventory, id, slot, mx, my);
                            self.dragged_item = None;
        
                            return true;
                        }
                    }
                }
            }
        }

        false
    }


    pub fn drop_item_to_slot(&self, inventory: &mut Inventory, item_id: u64, slot: Slot,
                            mx: i32, my: i32)
    {
        println!("Dropped item id={} to slot {:?}", item_id, slot);

        if slot == Slot::Bag {                            
            self.handle_item_dropped_to_bag(inventory, mx, my, item_id);
        }
        else {
            let entry_idx = inventory.find_entry_for_id(item_id).unwrap();
            let entry: &mut Entry = &mut inventory.entries[entry_idx];
    
            entry.slot = slot;
            entry.location_x = 0;
            entry.location_y = 0;
        }
    }


    pub fn drop_item(&mut self, world: &mut GameWorld, id: u64) 
    {
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


    fn handle_item_dropped_to_bag(&self, inventory: &mut Inventory,
                                  mx: i32, my: i32, 
                                  item_id: u64)
    {
        let item_opt = self.find_item_at(inventory, mx + self.area.x, my + self.area.y);

        println!("Found {:?} at drop location {}, {}", item_opt, mx, my);

        let offsets = self.slot_offsets.get(&Slot::Bag).unwrap();
        let rel_x = mx - offsets[0];
        let rel_y = my - offsets[1];

        match item_opt {
            None => {
                // dropped on nothing -> just set the new location for the item
                let entry_idx = inventory.find_entry_for_id(item_id).unwrap();
                let entry: &mut Entry = &mut inventory.entries[entry_idx];

                entry.slot = Slot::Bag;
                entry.location_x = rel_x / 32;
                entry.location_y = rel_y / 32;
            }
            Some(target_item_id) => {
                // the item was dropped onto another item.

                // Merge stacks if possible (todo)

                let ok = try_merge_stacks(inventory, item_id, target_item_id);

                // Merge items if there is a possible merge recipe
                if !ok {
                    let _ok = try_merge_items(inventory, item_id, target_item_id);
                }
            }
        }
    }
}


fn try_merge_stacks(inventory: &mut Inventory, dropped_item_id: u64, target_item_id: u64) -> bool
{
    let dropped_item_key;
    let dropped_item_stack;

    {
        let dropped_item = inventory.bag.get(&dropped_item_id).unwrap();
        dropped_item_key = dropped_item.key.to_string();
        dropped_item_stack = dropped_item.stack_size;
    }
    
    let target_item = inventory.bag.get_mut(&target_item_id).unwrap();

    if dropped_item_key == target_item.key && 
       dropped_item_stack < target_item.max_stack_size - target_item.stack_size 
    {
        target_item.stack_size += dropped_item_stack;
        return true;
    }
    
    false
}


fn try_merge_items(inventory: &mut Inventory, dropped_item_id: u64, target_item_id: u64) -> bool
{
    let drop_effect;
    {
        let dropped_item = inventory.bag.get(&dropped_item_id).unwrap();
        drop_effect = dropped_item.drop_effect.clone();
    }
    
    let target_item = inventory.bag.get_mut(&target_item_id).unwrap();

    if drop_effect == DropEffect::EnchantFireball {
        // todo: Check for correct target type. Not all items can take all enchantment types
        target_item.activation = Activation::Fireball;
        return true;
    }
    
    false
}