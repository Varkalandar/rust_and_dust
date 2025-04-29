use std::collections::HashMap;

use crate::read_lines;

pub struct CreatureFactory {
    prototypes: HashMap <String, CreaturePrototype>
}


pub struct CreaturePrototype {
    pub base_tile_id: usize,
    pub frames: usize,
    pub speed: f32,
    pub min_hp: i32,
    pub max_hp: i32,
    pub projectile_spawn_distance: f32,
}


#[allow(dead_code)]
pub struct Creature {
    pub base_tile_id: usize,
    pub frames: usize,
    pub base_speed: f32,
    pub hit_points: i32,
    pub projectile_spawn_distance: f32,
}


impl CreatureFactory {

    pub fn new() -> CreatureFactory 
    {
        let prototypes = read_creature_prototypes();

        CreatureFactory {
            prototypes,
        }
    }


    pub fn create(&self, key: &str) -> Creature 
    {
        let proto = self.prototypes.get(&key.to_string()).unwrap();

        Creature {
            base_tile_id: proto.base_tile_id,
            frames: proto.frames,
            base_speed: proto.speed,
            hit_points: (proto.max_hp + proto.min_hp) / 2,
            projectile_spawn_distance: proto.projectile_spawn_distance,
        }
    }
    

    pub fn add(&mut self, name: &str, creature: CreaturePrototype)
    {
        self.prototypes.insert(name.to_string(), creature);
    }
}


fn read_creature_prototypes() -> HashMap <String, CreaturePrototype> 
{
    let lines = read_lines("resources/creatures/creatures.csv");
    let mut prototypes = HashMap::new();

    for i in 1..lines.len() {
        let mut parts = lines[i].split(",");

        let name = parts.next().unwrap().to_string();

        prototypes.insert(name, 
            CreaturePrototype {
                base_tile_id: parts.next().unwrap().parse::<usize>().unwrap(),
                frames: parts.next().unwrap().parse::<usize>().unwrap(),
                speed: parts.next().unwrap().parse::<f32>().unwrap(),
                min_hp: parts.next().unwrap().parse::<i32>().unwrap(),
                max_hp: parts.next().unwrap().parse::<i32>().unwrap(),
                projectile_spawn_distance: parts.next().unwrap().parse::<f32>().unwrap(),
            });
    }

    prototypes
}
