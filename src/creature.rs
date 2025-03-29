use std::collections::HashMap;

use crate::read_lines;

pub struct CreatureFactory {
    prototypes: HashMap <String, CreaturePrototype>
}


struct CreaturePrototype {
    pub base_tile_id: usize,
    pub frames: usize,
    pub speed: f64,
    pub min_hp: i32,
    pub max_hp: i32,
}


pub struct Creature {
    pub base_tile_id: usize,
    pub frames: usize,
    pub base_speed: f64,
    pub hit_points: i32,
}


impl CreatureFactory {

    pub fn new() -> CreatureFactory {
        let prototypes = read_creature_prototypes();

        CreatureFactory {
            prototypes,
        }
    }


    pub fn create(&self, key: &str) -> Creature {
        let proto = self.prototypes.get(&key.to_string()).unwrap();

        Creature {
            base_tile_id: proto.base_tile_id,
            frames: proto.frames,
            base_speed: proto.speed,
            hit_points: (proto.max_hp + proto.min_hp) / 2,
        }
    }
}


fn read_creature_prototypes() -> HashMap <String, CreaturePrototype> {

    let lines = read_lines("resources/creatures/creatures.csv");
    let mut prototypes = HashMap::new();

    for i in 1..lines.len() {
        let mut parts = lines[i].split(",");

        let name = parts.next().unwrap().to_string();

        prototypes.insert(name, 
            CreaturePrototype {
                base_tile_id: parts.next().unwrap().parse::<usize>().unwrap(),
                frames: parts.next().unwrap().parse::<usize>().unwrap(),
                speed: parts.next().unwrap().parse::<f64>().unwrap(),
                min_hp: parts.next().unwrap().parse::<i32>().unwrap(),
                max_hp: parts.next().unwrap().parse::<i32>().unwrap(),
            });
    }

    prototypes
}
