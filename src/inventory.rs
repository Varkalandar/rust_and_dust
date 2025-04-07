use std::collections::HashMap;
use crate::item::Item;
use crate::ui::UiArea;

#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy)]
pub enum Slot {
    OnCursor = 0,
    Bag = 1,
    Stash = 2,
    Nose = 3,
    Body = 4,
    LWing = 5,
    RWing = 6,
    Engine = 7,
}

#[derive(Debug)]
pub struct Entry {
    pub item_id: u64,
    pub slot: Slot,
    pub location_x: i32,
    pub location_y: i32,    
}

#[derive(Debug)]
pub struct Inventory {

    // Keeps all the items (owns all the items)
    pub bag: HashMap <u64, Item>,

    // describes details about each of the owned items
    pub entries: Vec<Entry>,
}


impl Inventory {
    pub fn new() -> Inventory {
        Inventory {
            bag: HashMap::new(),
            entries: Vec::new(),
        }
    }

    pub fn put_item(&mut self, item: Item, slot: Slot) {

        println!("Adding item {:?} to inventory slot {:?}", item, slot);

        let spot = 
            if slot == Slot::Bag 
                {self.find_free_location(&item)}
            else
                {[0, 0]};

        println!("  at position {}, {}", spot[0], spot[1]);

        let entry = Entry {
            item_id: item.id,
            slot,
            location_x: spot[0],
            location_y: spot[1],
        };

        self.bag.insert(item.id, item);
        self.entries.push(entry);
    }


    pub fn remove_item(&mut self, id: u64) -> Option<Item> {
        let item_opt = self.bag.remove(&id);

        if item_opt.is_some() {
            let idx = self.find_entry_for_id(id).unwrap();
            self.entries.remove(idx);
        }

        item_opt
    }


    fn find_free_location(&self, item: &Item) -> [i32; 2] {

        // look for free space
        for grid_y in 0..8 
        {
            for grid_x in 0..14 
            {
                let mut free = true;

                for entry in &self.entries {
                    let bag_item = self.bag.get(&entry.item_id).unwrap();

                    let area = UiArea {
                        x: entry.location_x,                        
                        y: entry.location_y,
                        w: bag_item.inventory_w,
                        h: bag_item.inventory_h,
                    };

                    for x in 0..item.inventory_w {
                        for y in 0..item.inventory_h {
                            if area.contains(grid_x + x, grid_y + y) {
                                free = false;
                                break;
                            }
                        }
                    }
                }

                if free {
                    return [grid_x, grid_y];
                }
            }
        }

        return [-1, -1];
    }

    
    pub fn find_entry_for_id(&self, item_id: u64) -> Option<usize> {
        for idx in 0..self.entries.len() {
            let entry = &self.entries[idx];
            if entry.item_id == item_id {
                return Some(idx);
            }
        }

        None
    }
    

    #[allow(dead_code)]
    pub fn print_contents(&self) {
        println!("Inventory listing:");
        
        for entry in &self.entries {
            println!("  id={}, pos={}, {}, slot={:?}", 
                     entry.item_id, entry.location_x, entry.location_y, entry.slot);
        }

        for (key, value) in &self.bag {
            println!("  id={}, item={}", key, value.name()); 
        }
    }
}