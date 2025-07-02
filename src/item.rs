use core::str::Split;
use std::fmt::Formatter;
use std::collections::HashMap;
use rand::Rng;

use crate::inventory::Slot;
use crate::read_lines;
use crate::parse_rgba;


#[derive(Clone, Debug, PartialEq)]
pub enum Activation
{
    None,
    Fireball,
    FrostBolt,
    LightningStrike,
}

impl Activation
{
    pub fn info_str(&self) -> &str {
        match self {
            Activation::None => "",
            Activation::Fireball => "Activation: Fireball",
            Activation::FrostBolt => "Activation: Frost Bolt",
            Activation::LightningStrike => "Activation: Lightning Strike",
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
pub enum DropEffect
{
    None,
    EnchantFireball,
    EnchantFrostBolt,
    EnchantLightningStrike,
}


#[derive(Clone, Debug, PartialEq)]
pub enum ItemKind
{
    Misc,
    Wand,
    Bow,
    Ring,
    Amulet,
    Scroll,
    Currency,
    Plugin
}


impl ItemKind
{
    pub fn name_str(&self) -> &str
    {
        match self {
            ItemKind::Misc => "Miscellaneous",
            ItemKind::Wand => "Wand",
            ItemKind::Bow => "Bow",
            ItemKind::Ring => "Ring",
            ItemKind::Amulet => "Amulet",
            ItemKind::Scroll => "Scroll",
            ItemKind::Currency => "Currency",
            ItemKind::Plugin => "Plugin",
        }
    }
}


#[derive(Debug)]
pub struct ItemPrototype 
{   
    pub key: String,              // for prototype lookup
    pub singular: String,         // name for stack size == 1
    pub plural: String,           // name for stack size >= 2
    pub show_type: bool,          // display item type on a separate line?
    pub mods: Vec<ModPrototype>,
    
    pub inventory_tile_id: usize,
    pub inventory_w: i32,
    pub inventory_h: i32,
    pub inventory_scale: f32,
    pub map_tile_id: usize,
    pub map_scale: f32,
    pub color: [f32; 4],
    pub ilvl: u32,
    pub kind: ItemKind,
    pub max_stack_size: u32,       // some items can be stacked and must have a stack count
    pub base_price: u32,

    pub activation: Activation,
    pub drop_effect: DropEffect,

    pub description: String,
}


#[derive(Debug)]
pub struct Item 
{   
    // the ID must be unique in a game
    pub id: u64,

    pub key: String,              // for prototype lookup
    pub singular: String,         // name for stack size == 1
    pub plural: String,           // name for stack size >= 2
    pub show_type: bool,          // display item type on a separate line?
    pub mods: Vec<Mod>,
    
    pub inventory_tile_id: usize,
    pub inventory_w: i32,
    pub inventory_h: i32,
    pub inventory_scale: f32,
    pub map_tile_id: usize,
    pub map_scale: f32,
    pub color: [f32; 4],
    pub ilvl: u32,
    pub kind: ItemKind,
    pub stack_size: u32,         // some items can be stacked and must have a stack count
    pub max_stack_size: u32,
    pub base_price: u32,

    pub activation: Activation,
    pub drop_effect: DropEffect,

    pub description: String,
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
            200 .. 500 => 5 * 2,
            _ => 6 * 2,
        }
    }


    pub fn calc_price(&self) -> u32
    {
        // todo: factor in the mods
        return self.base_price;
    }


    pub fn has_mod_type(&self, attribute: Attribute) -> bool
    {
        for modifier in &self.mods {
            if modifier.attribute == attribute {
                return true;
            }
        }

        false
    }
}


pub struct ItemFactory
{
    next_id: u64,

    proto_items: HashMap<String, ItemPrototype>,
    proto_mods: HashMap<String, ModPrototype>,
}


impl ItemFactory 
{
    pub fn new() -> ItemFactory 
    {
        let proto_mods = read_proto_mods();
        let proto_items = read_proto_items(&proto_mods);

        ItemFactory {
            next_id: 0,
            proto_items,
            proto_mods,
        }
    }


    pub fn create_base(&mut self, key: &str) -> Item 
    {
        let proto_opt = self.proto_items.get(key);
        
        if proto_opt.is_none() {
            panic!("Unknown item '{}'", key);
        }
        else {
            let id = self.next_id;
            self.next_id += 1;
            let proto = proto_opt.unwrap();

            Item {
                id, 
                key: proto.key.to_string(),
                singular: proto.singular.to_string(),
                plural: proto.plural.to_string(),
                show_type: proto.show_type,
                mods: Vec::new(),

                inventory_tile_id: proto.inventory_tile_id,
                inventory_w: proto.inventory_w,
                inventory_h: proto.inventory_h,
                inventory_scale: proto.inventory_scale,
                map_scale: proto.map_scale,
                color: proto.color,
                ilvl: proto.ilvl,
                kind: proto.kind.clone(),
            
                map_tile_id: proto.map_tile_id,
                stack_size: 1,
                max_stack_size: proto.max_stack_size,
                base_price: proto.base_price,

                activation: proto.activation.clone(),
                drop_effect: proto.drop_effect.clone(),
                description: proto.description.to_string(),
            }
        }
    }


    pub fn create<R: Rng + ?Sized>(&mut self, key: &str, rng: &mut R) -> Item 
    {
        let mut item = self.create_base(key);
        let proto = self.proto_items.get(key).unwrap();
        item.mods = process_proto_mods(&proto.mods, rng); // proto.mods.clone()

        item
    }


    pub fn create_random<R: Rng + ?Sized>(&mut self, rng: &mut R, max_level: u32) -> Item
    {
        let mut matches = Vec::with_capacity(self.proto_items.len());

        for (_key, proto) in &self.proto_items {
            // ilvl 0 means "do not generate randomly"
            // these items can still be generated by their key
            if proto.ilvl > 0 && proto.ilvl >= max_level {
                matches.push(proto.key.clone());
            }
        }

        // pick a random one
        let index = rng.random_range(0 .. matches.len());

        let item = self.create(&matches[index], rng);

        item
    }


    /**
     * Items with a higher number of mods should be more rare than items
     * with a low number of mods. (Diminishing returns)
     *
     * @param max_level The max level of the generated item
     * @param max_mods The max number of mods generated
     * @param base_chance The basic chance to add a mod at all
     * @param mf_factor For each mod added the chance will be multiplied with this factor
     *                  so we can control how likely it is to create another mod after the one
     * @return The generated item.
     */
    pub fn create_random_item<R: Rng + ?Sized>(&mut self, rng: &mut R,
                                               max_level: u32, max_mods: u32, 
                                               base_chance: f32, mf_factor: f32) -> Item
    {
        let mut item = self.create_random(rng, max_level);        
        let mut chance = base_chance;
        let mut tries = 0;

        while tries < max_mods {
            if rng.random::<f32>() < chance {
                self.add_random_mods(&mut item, rng, 1, max_level);
            }      
            
            chance = chance * mf_factor;    // shrink the chance
            tries += 1;
        }

        item
    }


    fn add_random_mods<R: Rng + ?Sized>(&self, item: &mut Item, rng: &mut R, 
                                        mod_count: u32, max_level: u32)
    {
        let mut keys: Vec<(&String, &ModPrototype)> = 
            self.proto_mods.iter().filter( |value| -> bool 
                {
                    if item.kind == ItemKind::Scroll || 
                       item.kind == ItemKind::Currency {
                        return false
                    }
                    else {
                        // only accept mods which match the level

                        if value.1.ilvl <= max_level {
                            // this mod is acceptable
                            return true
                        } 

                        // we filter everything else
                        false
                    }
                }
            ).collect();
        

        for _i in 0 .. mod_count {
            
            if keys.len() == 0 {
                // there are no more suitable mods for this item
                break;
            }

            let n = rng.random_range(0 .. keys.len());
            let key = keys.remove(n);
            let proto_mod = self.proto_mods.get(key.0).unwrap();

            if item.has_mod_type(proto_mod.attribute.clone()) {
                // a mod of this type is already on the item, 
                // we don't ad another one of the same type.
            }
            else {
                let modifier = random_from_range(proto_mod, rng, ModKind::Echanted);
                item.mods.push(modifier);
            }
        }
    }
}


fn read_proto_items(proto_mods: &HashMap<String, ModPrototype>) -> HashMap<String, ItemPrototype> 
{
    let lines = read_lines("resources/items/items.csv");
    let mut proto_items: HashMap<String, ItemPrototype> = HashMap::new();

    for i in 1..lines.len() {
        let mut parts = lines[i].split(",");
        let key = parts.next().unwrap().to_string();
        
        // ignore empty lines, they are just to separate sections
        if key.len() > 0 {
            proto_items.insert(
                key.to_string(),
                ItemPrototype {
                    key,
                    singular: parts.next().unwrap().to_string(),
                    plural:  parts.next().unwrap().to_string(),
                    show_type: "yes" == parts.next().unwrap(), 
                    inventory_tile_id: parts.next().unwrap().parse::<usize>().unwrap(),
                    map_tile_id: parts.next().unwrap().parse::<usize>().unwrap(),
                    inventory_w: parts.next().unwrap().parse::<i32>().unwrap(),
                    inventory_h: parts.next().unwrap().parse::<i32>().unwrap(),
                    inventory_scale: parts.next().unwrap().parse::<f32>().unwrap(),
                    map_scale: parts.next().unwrap().parse::<f32>().unwrap(),
                    color: parse_rgba(parts.next().unwrap()),
                    ilvl: parts.next().unwrap().parse::<u32>().unwrap(),
                    kind: parse_item_type(parts.next().unwrap()),
                    max_stack_size: parts.next().unwrap().parse::<u32>().unwrap(),
                    base_price: parts.next().unwrap().parse::<u32>().unwrap(),

                    drop_effect: parse_drop_effect(parts.next().unwrap()),
                    mods: parse_mods(&mut parts, proto_mods),
                    activation: Activation::None,
                    description: parts.next().unwrap().to_string(),
                }
            );
        }
    }

    proto_items
}


fn read_proto_mods() -> HashMap<String, ModPrototype> 
{
    let lines = read_lines("resources/items/modifiers.csv");
    let mut proto_items: HashMap<String, ModPrototype> = HashMap::new();

    for i in 1..lines.len() {
        let mut parts = lines[i].split(",");
        let key = parts.next().unwrap().to_string();
        
        // ignore empty lines, they are just to separate sections
        if key.len() > 0 {
            proto_items.insert(
                key.to_string(),
                ModPrototype {
                    attribute: parse_attribute(parts.next().unwrap()),
                    min_value: parts.next().unwrap().parse::<i32>().unwrap(),
                    max_value: parts.next().unwrap().parse::<i32>().unwrap(),
                    unit: parse_unit(parts.next().unwrap()),
                    ilvl: parts.next().unwrap().parse::<u32>().unwrap(),
                }
            );
        }
    }

    proto_items
}


fn calc_slot(v: i32) -> Slot 
{
    match v {
        0 => Slot::OnCursor,
        1 => Slot::Bag,
        2 => Slot::Stash,
        3 => Slot::Head,
        4 => Slot::Body,
        5 => Slot::LHand,
        6 => Slot::RHand,
        7 => Slot::Amulet,
        8 => Slot::LRing,
        9 => Slot::RRing,
        _ => {
            println!("calc_slot: Cannot find slot for input value {}, using Slot::Bag.", v);
            Slot::Bag
        }
    }
}


fn parse_mods(parts: &mut Split<&str>, proto_mods: &HashMap<String, ModPrototype>) -> Vec<ModPrototype>
{
    let mut result = Vec::new();

    loop {
        let key = parts.next();
        if key.is_some() {
            let key = key.unwrap();

            match key {
                "info" => {
                    // info must be the last key and it's not a mod, 
                    // so we stop parsing mods here
                    break;
                }
                "" => {
                    // end of data
                    break;
                }
                _ => {
                    let mod_opt = proto_mods.get(key);
                    if mod_opt.is_some() {
                        result.push(mod_opt.unwrap().clone());
                    }
                    else {
                        panic!("parse_mods: unknown modifier key found: '{}'", key);
                    }
                } 
            }
        }
        else {
            // out of data
            break;
        }
    }

    result
}


fn parse_attribute(input: &str) -> Attribute
{
    match input {
        "res_fire" => Attribute::ResFire,
        "res_light" => Attribute::ResLight,
        "res_cold" => Attribute::ResCold,
        "spell_dam" => Attribute::SpellDamage,
        "phys_dam" => Attribute::PhysicalDamage,
        _ => panic!("parse_attribute: unknown attribute {}", input),
    }
}


fn parse_unit(input: &str) -> Unit
{
    match input {
        "%" => Unit::Percent,
        _ => Unit::Integer,
    }
}


fn parse_range(input: &str) -> (i32, i32) 
{
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


fn parse_drop_effect(input: &str) -> DropEffect
{
    if "enchant_fireball" == input {
        DropEffect::EnchantFireball
    }
    else if "enchant_frost_bolt" == input {
        DropEffect::EnchantFrostBolt
    }
    else if "enchant_lightning_strike" == input {
        DropEffect::EnchantLightningStrike
    }
    else if "" == input {
        DropEffect::None
    }
    else {
        panic!("Unknown drop effect '{}'", input);
    }
}


fn parse_item_type(input: &str) -> ItemKind
{
    if "wand" == input {
        return ItemKind::Wand;
    }
    else if "ring" == input {
        return ItemKind::Ring;
    }
    else if "bow" == input {
        return ItemKind::Bow;
    }
    else if "amulet" == input {
        return ItemKind::Amulet;
    }
    else if "scroll" == input {
        return ItemKind::Scroll;
    }
    else if "currency" == input {
        return ItemKind::Currency;
    }
    else if "plugin" == input {
        return ItemKind::Plugin;
    }
    else {
        println!("parse_item_type: Unknown item type '{}'", input);
        return ItemKind::Misc;
    }
}


#[allow(dead_code)]
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Attribute {

    Agility,
    Armor,
    Speed,
    PhysicalDamage,
    SpellDamage,
    ResFire,
    ResLight,
    ResCold,
}


impl std::fmt::Display for Attribute 
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {

        let name = match self {
            Attribute::Agility => "Agility",
            Attribute::Armor => "Armor",
            Attribute::Speed => "Speed",
            Attribute::PhysicalDamage => "Physical Damage",
            Attribute::SpellDamage => "Added Spell Damage",
            Attribute::ResFire => "Fire Resistance",
            Attribute::ResLight => "Lightning Resistance",
            Attribute::ResCold => "Cold Resistance",
        };

        write!(f, "{}", name)
    }
}


#[derive(PartialEq, Debug, Clone)]
pub enum Unit {
    Percent,
    Integer,
}


impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {

        let name = match self {
            Unit::Percent => "Percent",
            Unit::Integer => "Integer",
        };

        write!(f, "{}", name)
    }
}


#[derive(Clone, Debug, PartialEq)]
pub enum ModKind {
    Implicit,      // part of the item itself
    Echanted,      // These can be added/removed/modified 
}


#[derive(Debug, Clone)]
pub struct ModPrototype {
    pub attribute: Attribute,
    pub min_value: i32,
    pub max_value: i32,
    pub unit: Unit,
    pub ilvl: u32,
}


#[derive(Debug, Clone)]
pub struct Mod {
    pub attribute: Attribute,
    pub min_value: i32,
    pub max_value: i32,
    pub unit: Unit,
    pub kind: ModKind,
    pub ilvl: u32,
}


impl Mod 
{
    pub fn assemble_mod_line_text(&self) -> String 
    {
        let text;
        let min_value = self.min_value;
        let max_value = self.max_value;

        if max_value > 0 {
            let range = if min_value == max_value {
                min_value.to_string()
            } else {
                min_value.to_string() + "-" + &max_value.to_string()
            };

            let unit_sign = if self.unit == Unit::Percent {"%"} else {""};

            text = self.attribute.to_string() + ": " + &range + unit_sign;
        }
        else {
            text = "".to_string();
        }
        
        text
    }
}


fn process_proto_mods<R: Rng + ?Sized>(mods: &Vec<ModPrototype>, rng: &mut R) -> Vec<Mod>
{
    let mut result = Vec::with_capacity(mods.len());

    for modifier in mods {

        if modifier.attribute == Attribute::ResFire || 
           modifier.attribute == Attribute::ResLight ||
           modifier.attribute == Attribute::ResCold {
            // the proto mod is a range, we need to pick one value
            // from that range to produce a concrete mod for our item
            result.push(random_from_range(modifier, rng, ModKind::Implicit));
        }
        else {
            result.push(random_from_range(modifier, rng, ModKind::Implicit));
        }
    }

    result
}


fn random_from_range<R: Rng + ?Sized>(modifier: &ModPrototype, rng: &mut R, kind: ModKind) -> Mod 
{
    let actual_value = rng.random_range(modifier.min_value .. modifier.max_value);
    Mod {
        attribute: modifier.attribute.clone(),
        min_value: actual_value,
        max_value: actual_value,
        unit: modifier.unit.clone(),
        kind,
        ilvl: modifier.ilvl,
    }
}