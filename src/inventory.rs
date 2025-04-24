use std::collections::HashMap;
use crate::item::Item;
use crate::item::ItemKind;
use crate::ItemFactory;
use crate::ui::*;

#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy)]
pub enum Slot {
    OnCursor = 0,
    Bag = 1,
    Stash = 2,
    Head = 3,
    Body = 4,
    LHand = 5,
    RHand = 6,
    Amulet = 7,
    LRing = 8,
    RRing = 9,
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


    pub fn clear(&mut self)
    {
        self.bag.clear();
        self.entries.clear();
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


    fn find_free_location(&self, item: &Item) -> [i32; 2] 
    {
        // look for free space
        for grid_y in 0..8 {
            for grid_x in 0..14 {
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
    

    /**
     * @return The total amount of curency in this inventoy, measured in copper coins
     */ 
    pub fn total_money(&self) -> u32
    {
        let mut total = 0;

        for (_key, item) in &self.bag {
            if item.kind == ItemKind::Currency {
                let mut count = item.stack_size;

                if "silver_coin" == item.key {
                    count *= 100;
                }

                total += count;
            }
        }

        total
    }


    /**
     * Counts items of the given type, accumulates stack sizes
     */
    pub fn count(&self, key: &str) -> u32
    {
        let mut count = 0;

        for (_key, item) in &self.bag {
            if item.key == key {
                count += item.stack_size;
            }
        }

        count
    }


    /**
     * Reduce the currency in the inventory by the given amount
     */
    pub fn withdraw_money(&mut self, amount: u32, item_factory: &mut ItemFactory)
    {
        let silver = amount / 100;
        let copper = amount % 100;

        let mut copper_available = self.count("copper_coin");

        println!("To pay: {} silver and {} copper. Available copper is {}", silver, copper, copper_available);

        if copper_available < copper {
            // we have too few copper coins, and must convert some silver first.
            // -> we need to convert at least and at most 1 silver coin.
            self.split_one_silver_coin(item_factory);
            copper_available += 100;
        }

        // we try to pay with small coins first
        // see how much we can cover.

        let n = copper_available - copper;
        let copper_to_silver = n / 100;

        // we can covert "copper_to_silver" amount of copper coins,
        // and sill pay the copper demanded. But we might not need as
        //  many coins, take min from silver demand and what we can convert.
        let mut copper_to_remove = std::cmp::min(copper_to_silver, silver) * 100 + copper;
        let mut silver_to_remove = silver - copper_to_remove / 100;

        println!("Paying {} copper and {} silver.", copper_to_remove, silver_to_remove);

        let mut items_to_remove = Vec::new();

        for (_key, item) in &mut self.bag {
            if item.kind == ItemKind::Currency {

                if "copper_coin" == item.key {
                    let available = item.stack_size;

                    if available > copper_to_remove {
                        item.stack_size -= copper_to_remove;
                        copper_to_remove = 0;
                    }
                    else {
                        items_to_remove.push(item.id);
                        copper_to_remove -= available;
                    }
                }
            }

            if copper_to_remove == 0 { break; }
        }

        // now, pay the rest in silver

        for (_key, item) in &mut self.bag {
            if item.kind == ItemKind::Currency {

                if "silver_coin" == item.key {
                    let available = item.stack_size;

                    if available > silver_to_remove {
                        item.stack_size -= silver_to_remove;
                        silver_to_remove = 0;
                    }
                    else {
                        items_to_remove.push(item.id);
                        silver_to_remove -= available;
                    }
                }
            }

            if silver_to_remove == 0 { break; }
        }

        // clean up empty coin stacks
        for id in items_to_remove {
            self.remove_item(id);
        }        
    }


    fn split_one_silver_coin(&mut self, item_factory: &mut ItemFactory) 
    {
        let mut items_to_remove = Vec::new();

        for (_key, item) in &mut self.bag {
            if item.kind == ItemKind::Currency {

                if "silver_coin" == item.key {
                    let available = item.stack_size;

                    if available > 1 {
                        item.stack_size -= 1;
                    }
                    else {
                        items_to_remove.push(item.id);
                    }
                }
            }
        }
        
        // clean up empty coin stacks
        for id in items_to_remove {
            self.remove_item(id);
        }        

        // and add the 100 copper coins
        let mut copper_coins = item_factory.create_base("copper_coin");
        copper_coins.stack_size = 100;
        self.put_item(copper_coins, Slot::Bag);
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