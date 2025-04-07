use core::str::Split;
use std::fmt::Formatter;
use std::collections::HashMap;
use rand::Rng;

use crate::inventory::Slot;
use crate::read_lines;
use crate::parse_rgba;

#[derive(Debug)]
pub struct Item 
{   
    // the ID must be unique in a game
    pub id: u64,

    pub key: String,              // for prototype lookup
    pub singular: String,         // name for stack size == 1
    pub plural: String,           // name for stack size >= 2
    pub mods: Vec<Mod>,
    
    pub inventory_tile_id: usize,
    pub inventory_w: i32,
    pub inventory_h: i32,
    pub inventory_scale: f32,
    pub map_tile_id: usize,
    pub map_scale: f32,
    pub color: [f32; 4],
    pub slot: Slot,
    pub stack_size: u32,         // some items can be stacked and must have a stack count
    pub max_stack_size: u32,
}


impl Item 
{
    pub fn name(&self) -> String 
    {
        if self.stack_size == 1 {
            return self.singular.to_string();
        }
        else {
            if self.plural.len() > 0 {
                return self.stack_size.to_string() + " " + &self.plural;
            }
        }

        return self.singular.to_string();
    }

    
    pub fn get_attribute_total_mod(&self, attribute: Attribute) -> f32 
    {
        let mut sum: f32 = 0.0;

        for m in &self.mods {
            if m.attribute == attribute {
                sum = sum + m.min_value as f32;
            }            
        }
        
        sum
    }
    

    pub fn calc_image_offset_for_stack_size(stack_size: u32) -> usize 
    {
        match stack_size {
            0 => 0,    
            1 => 0,    
            2 => 1 * 2,
            3 => 2 * 2,
            4 .. 50 => 3 * 2,
            50 .. 200 => 4 * 2,
            100 .. 10000 => 5 * 2,
            
            _ => 0 * 2,
        }
    }

    pub fn print_debug(&self) 
    {
        println!("{}", self.name());
    }
}


pub struct ItemFactory
{
    next_id: u64,

    proto_items: HashMap<String, Item>,
}


impl ItemFactory {
    pub fn new() -> ItemFactory 
    {
        let mut proto_items = read_proto_items();
        let plugins = read_plugins();

        for plugin in plugins {
            let key = plugin.key.to_string();
            proto_items.insert(key, plugin);
        }
        
        ItemFactory {
            next_id: 0,
            proto_items,
        }
    }


    pub fn create(&mut self, key: &str) -> Item 
    {
        let id = self.next_id;
        self.next_id += 1;
        
        let proto = self.proto_items.get(key).unwrap();

        Item {
            id, 
            key: proto.key.to_string(),
            singular: proto.singular.to_string(),
            plural: proto.plural.to_string(),
            mods: proto.mods.clone(),

            inventory_tile_id: proto.inventory_tile_id,
            inventory_w: proto.inventory_w,
            inventory_h: proto.inventory_h,
            inventory_scale: proto.inventory_scale,
            map_scale: proto.map_scale,
            color: proto.color,
            slot: proto.slot,
        
            map_tile_id: proto.map_tile_id,
            stack_size: 1,
            max_stack_size: proto.max_stack_size,
        }
    }

    pub fn create_random<R: Rng + ?Sized>(&mut self, rng: &mut R, area_level: i32) -> Item
    {
        let mut matches = Vec::with_capacity(self.proto_items.len());

        // todo: filter by level
        for proto in &self.proto_items {
            matches.push(proto.1.key.clone());
        }

        // pick a random one
        let index = rng.random_range(0 .. matches.len());

        self.create(&matches[index])
    }
}


fn read_proto_items() -> HashMap<String, Item> 
{
    let lines = read_lines("resources/items/items.csv");
    let mut proto_items: HashMap<String, Item> = HashMap::new();

    for i in 1..lines.len() {
        let mut parts = lines[i].split(",");
        let key = parts.next().unwrap().to_string();
        
        proto_items.insert(
            key.to_string(),
            Item {
                id: 0,      // just a placeholder in case of item prototypes.
                key,
                singular: parts.next().unwrap().to_string(),
                plural:  parts.next().unwrap().to_string(),
                inventory_tile_id: parts.next().unwrap().parse::<usize>().unwrap(),
                map_tile_id: parts.next().unwrap().parse::<usize>().unwrap(),
                inventory_w: parts.next().unwrap().parse::<i32>().unwrap(),
                inventory_h: parts.next().unwrap().parse::<i32>().unwrap(),
                inventory_scale: parts.next().unwrap().parse::<f32>().unwrap(),
                map_scale: parts.next().unwrap().parse::<f32>().unwrap(),
                color: parse_rgba(parts.next().unwrap()),
                slot: calc_slot(parts.next().unwrap().parse::<i32>().unwrap()),
                stack_size: 1,
                max_stack_size: parts.next().unwrap().parse::<u32>().unwrap(),
                mods: parse_mods(&mut parts),
            }
        );
    }

    proto_items
}


fn read_plugins() -> Vec<Item> {

    let lines = read_lines("resources/items/plugins.csv");
    let mut plugins: Vec<Item> = Vec::new();

    for i in 1..lines.len() {
        let mut parts = lines[i].split(",");

        plugins.push(Item {
            id: 0,
            key: parts.next().unwrap().to_string(),
            singular: parts.next().unwrap().to_string(),
            plural: "".to_string(),
            inventory_tile_id: parts.next().unwrap().parse::<usize>().unwrap(),
            map_tile_id: parts.next().unwrap().parse::<usize>().unwrap(),
            inventory_w: parts.next().unwrap().parse::<i32>().unwrap(),
            inventory_h: parts.next().unwrap().parse::<i32>().unwrap(),
            inventory_scale: parts.next().unwrap().parse::<f32>().unwrap(),
            map_scale: parts.next().unwrap().parse::<f32>().unwrap(),
            color: [1.0, 1.0, 1.0, 1.0],
            slot: Slot::Bag,
            stack_size: 1,
            max_stack_size: 1,
            mods: Vec::new(),
        });
    }

    plugins
}


fn calc_slot(v: i32) -> Slot {
    match v {
        0 => Slot::Bag,
        1 => Slot::Stash,
        2 => Slot::Nose,
        3 => Slot::Body,
        4 => Slot::LWing,
        5 => Slot::RWing,
        6 => Slot::Engine,
        _ => Slot::Bag,
    }
}


fn parse_mods(parts: &mut Split<&str>) -> Vec<Mod> {
    let mut result = Vec::new();

    result.push(parse_mod(parts.next(), Attribute::Structure));
    result.push(parse_mod(parts.next(), Attribute::Agility));
    result.push(parse_mod(parts.next(), Attribute::Armor));
    result.push(parse_mod(parts.next(), Attribute::Computation));
    result.push(parse_mod(parts.next(), Attribute::Speed));
    result.push(parse_mod(parts.next(), Attribute::PhysicalDamage));
    result.push(parse_mod(parts.next(), Attribute::SpellDamage));

    result
}

fn parse_mod(input: Option<&str>, attribute: Attribute) -> Mod {

    let (min_value, max_value) = parse_range(input.unwrap());

    Mod { 
        attribute,
        min_value,
        max_value,
    }
}


fn parse_range(input: &str) -> (i32, i32) {
    // .parse::<i32>().unwrap();

    if input.contains("-") {
        let mut parts = input.split("-");
        let min_value = parts.next().unwrap().parse::<i32>().unwrap();
        let max_value = parts.next().unwrap().parse::<i32>().unwrap();
        (min_value, max_value)
    }
    else {
        let value = input.parse::<i32>().unwrap();
        (value, value)
    }
}


#[allow(dead_code)]
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Attribute {

    Structure,
    Agility,
    Armor,
    Computation,
    Speed,
    PhysicalDamage,
    SpellDamage,
    RadiationDamage,

    Integrity,
    Energy,
}


impl std::fmt::Display for Attribute {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {

        let name = match self {
            Attribute::Structure => "Structure",
            Attribute::Agility => "Agility",
            Attribute::Armor => "Armor",
            Attribute::Computation => "Computation",
            Attribute::Speed => "Speed",
            Attribute::PhysicalDamage => "Physical Damage",
            Attribute::SpellDamage => "Added Spell Damage",
            Attribute::RadiationDamage => "Radiation Damage",
                
            Attribute::Integrity => "Integrity",
            Attribute::Energy => "Energy",        
        };

        write!(f, "{}", name)
    }
}


#[derive(Debug, Clone)]
pub struct Mod {
    pub attribute: Attribute,
    pub min_value: i32,
    pub max_value: i32,
}