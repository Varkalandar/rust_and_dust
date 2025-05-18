use vecmath::{Vector2, vec2_sub, vec2_add, vec2_scale, vec2_len, vec2_square_len};
use geo::Polygon;
use geo::LineString;
use geo::Contains;
use geo::coord;

use std::f32::consts::PI;
use std::io::prelude::*;
use std::io::{Result, BufWriter};
use std::fs::File;
use std::path::PathBuf;
use std::collections::HashMap;
use std::boxed::Box;

use crate::shop::Shop;
use crate::item::Item;
use crate::ItemFactory;
use crate::creature::Creature;
use crate::creature::CreatureFactory;
use crate::projectile::ProjectileBuilder;
use crate::inventory::Inventory;
use crate::particle_driver::ParticleDriver;
use crate::animation::*;
use crate::sound::Sound;
use crate::SoundPlayer;
use crate::mob_group::MobGroup;
use crate::CREATURE_TILESET;
use crate::parse_rgba;
use crate::gfx::gl_support::BlendMode;
use crate::Slot;
use crate::WHITE;
use crate::RngReceiver;

pub const MAP_GROUND_LAYER:usize = 0;
pub const MAP_OBJECT_LAYER:usize = 1;
pub const MAP_CLOUD_LAYER:usize = 2;


pub struct Map {

    pub layers: [HashMap<u64, MapObject>; 7],
    pub animations: HashMap<u64, Box<dyn Animated>>,
    pub transitions: Vec<MapTransition>,
    
    // the map area which can be walked. New areas must be merged into this
    pub walkable: Vec<Polygon<f32>>,
    pub blocked: Vec<Polygon<f32>>,  // non-walkable areas inside the walkable area

    // 'AI' controlled objects
    pub mob_groups: Vec<MobGroup>,

    pub shops: Vec<Shop>,

    pub has_selection: bool,
    pub selected_item: u64,
    pub selected_layer: usize,

    pub name: String,
    pub map_image_name: String,
    pub backdrop_image_name: String,

    pub factory: MapObjectFactory,
    pub item_factory: ItemFactory,
    pub creature_factory: CreatureFactory,
    pub projectile_builder: ProjectileBuilder,
    pub player_id: u64, 
}


impl Map {
    pub fn new(name: &str, 
               map_image_name: &str, backdrop_image_name: &str,
               item_factory: ItemFactory) -> Map {
        let mut layers = [HashMap::new(), HashMap::new(), HashMap::new(), HashMap::new(), HashMap::new(), HashMap::new(), HashMap::new(),];
/*        
        let player_visual = Visual {
            base_image_id: 39,
            tileset_id: 4,
            current_image_id: 39,
            directions: 16,
            phases: 1, 
            height: 24.0,
            scale: 0.75,
            color: WHITE,
            glow: WHITE,
            blend: BlendMode::Blend,
            particles: ParticleDriver::new(),       
        };
*/
        let player_visual = Visual {
            base_image_id: 1,
            tileset_id: 4,
            current_image_id: 1,
            directions: 8,
            phases: 1, 
            z_off: 0.0,
            scale: 1.5,
            color: WHITE,
            glow: WHITE,
            blend: BlendMode::Blend,
            particles: ParticleDriver::new(),       
        };

        let mut factory = MapObjectFactory {
            next_id: 1,
        };

        let creature_factory = CreatureFactory::new();
        let projectile_builder = ProjectileBuilder::new();

        let mut player = factory.create_mob(39, 4, [1000.0, 1000.0], 24.0, 1.0);
        let player_id = player.uid;
        player.visual = player_visual;
        player.update_action = UpdateAction::EmitDriveParticles;
        player.creature = Some(creature_factory.create("Player"));
        player.move_end_action = MoveEndAction::PickItemsUp;

        layers[MAP_OBJECT_LAYER].insert(player.uid, player);


        // testing

        // let area = Polygon::new(LineString::from(vec![(100.0, 100.0), (1000.0, 100.0), (1000.0, 500.0), (100.0, 500.0)]), vec![]);
        let walkable = Vec::new();
        // walkable.push(area);

        Map {
            layers,

            animations: HashMap::new(),
            transitions: Vec::new(),
            mob_groups: Vec::new(),

            walkable,
            blocked: Vec::new(),

            shops: Vec::new(),
            has_selection: false,
            selected_item: 0,
            selected_layer: 0,

            name: name.to_string(),
            map_image_name: map_image_name.to_string(),
            backdrop_image_name: backdrop_image_name.to_string(),
        
            factory,
            item_factory,
            creature_factory,
            projectile_builder,
            player_id,
        }
    }


    pub fn clear(&mut self) {
    
        for layer in &mut self.layers {
            layer.clear();
        }

        self.transitions.clear();

        self.walkable.clear();
        self.blocked.clear();
    }


    pub fn get_player_position(&self) -> Vector2<f32> {

        let mob: &MapObject = self.layers[MAP_OBJECT_LAYER].get(&self.player_id).unwrap();
        return mob.position;
    }


    pub fn set_player_position(&mut self, position: Vector2<f32>) {

        let mob: &mut MapObject = self.layers[MAP_OBJECT_LAYER].get_mut(&self.player_id).unwrap();
        mob.position = position;
    }


    pub fn find_nearest_object(layer: &HashMap<u64, MapObject>, position: &Vector2<f32>, search_radius: f32, ignore_uid: u64) -> Option<u64> {
        let mut distance = search_radius * search_radius;
        let mut best_id = 0;

        for (_key, object) in layer {
            let dx = object.position[0] - position[0];
            let dy = object.position[1] - position[1];
            let d2 = dx * dx + dy * dy;

            // println!("object {} has distance {}", object.uid, d2);

            if d2 < distance && object.uid != ignore_uid {
                distance = d2;
                best_id = object.uid;
            }
        }

        let mut result:Option<u64> = None;

        if distance < search_radius * search_radius {
            result = Some(best_id);
            // println!("  best object is {}", best_id);
        }

        result
    }


    pub fn is_walkable(&self, position: [f32; 2]) -> bool {
        let p = coord! { x: position[0], y: position[1] };
        
        let mut ok = false;
        
        for polygon in &self.walkable {
            if polygon.contains(&p) {
                ok = true;
                break;
            }
        }

        println!("Location {}, {} is walkable={}", position[0], position[1], ok);

        ok
    }


    /**
     * Move all map objects, handle projectile hits, clean up data structures.
     
     * @return A list of all MapObjects which had been killed or destroyed during this update
     */
    pub fn update(&mut self, dt: f32, 
                  inventory: &mut Inventory, rng: &RngReceiver, speaker: &mut SoundPlayer) 
        -> (Vec<MapObject>, Option<usize>)
    {
        let mut transition = None;
        let mut kill_list = Vec::new();
        let mut phit_list = Vec::new();

        {
            let groups = &mut self.mob_groups;
            let mobs = &mut self.layers[MAP_OBJECT_LAYER];
            let factory = &mut self.factory;
            let projectile_builder = &mut self.projectile_builder;

            for group in groups {
                group.update(self.player_id, dt, mobs, rng, factory, projectile_builder, speaker);
            }
        }

        let mut pickup_position = None;

        for (_key, mob) in &mut self.layers[MAP_OBJECT_LAYER] {
            let before = mob.move_time_left;
            mob.move_dt(dt);
            let after = mob.move_time_left;

            // did the move just end?
            if before > 0.0 && after <= 0.0 {
                mob.visual.particles.clear();
                mob.stop_moving();

                if mob.move_end_action == MoveEndAction::PickItemsUp {
                    pickup_position = Some(mob.position);
                }

                if mob.move_end_action == MoveEndAction::RemoveFromMap {
                    kill_list.push(mob.uid);
                }
            }

            // particle stuff
            {
                let particles = &mut mob.visual.particles;
                let len = particles.spawn_ids.len();

                if len > 0 {
                    let chance = particles.spawn_chance * dt;
                    if rng.random() < chance {
                        let spark = particles.spawn_ids[rng.random_urange(0, len)];
                        
                        particles.add_particle(0.0, -400.0, 0.0, 0.0, 0.0, 0.0, 
                                               0.1, spark, [0.7, 0.75, 0.9]);
                    }
                }

                particles.drive(dt);
            }


            let animation_opt = self.animations.get(&mob.uid);
            match animation_opt {
                None => {
                },
                Some(animation) => {
                    animation.update(dt, mob);
                }
            }

            // must this mob be removed from the map?
            if mob.update_action == UpdateAction::RemoveFromMap {
                kill_list.push(mob.uid);
            }
            else if mob.update_action == UpdateAction::EmitDriveParticles && after > 0.0 {
                emit_drive_particles(mob, dt, rng);
            }
        }

        // player might have picked something up
        if pickup_position.is_some() {
            Self::check_pickup(&mut self.layers[MAP_OBJECT_LAYER], &pickup_position.unwrap(), 
                               inventory, self.player_id, speaker);

            transition = self.check_player_transition();
        }

        for (_key, mob) in &self.layers[MAP_OBJECT_LAYER] {

            let mob_type = mob.mob_type;
            if mob_type == MobType::PlayerProjectile || mob_type == MobType::CreatureProjectile {

                // projectiles may have hit something in the move
                let target = Self::find_nearest_object(&self.layers[MAP_OBJECT_LAYER], &mob.position, 80.0, mob.uid);
                match target {
                    None => {}
                    Some(uid) => {
                        phit_list.push((mob.uid, uid));                        
                    }
                }
            }
        }

        for (projectile, target) in phit_list {

            // some projectiles can only hit certain targets, check if the hit was valid
            let valid = self.handle_projectile_hit(projectile, target, rng, speaker);

            if valid {
                kill_list.push(projectile);
                
                let start_time = self.layers[MAP_OBJECT_LAYER].get(&target).unwrap().animation_timer;
                self.animations.insert(target, Box::new(RemovalAnimation::new(start_time, 0.3)));
            }
        }

        let mut killed_mob_list = Vec::with_capacity(kill_list.len()); 

        for id in kill_list {
            self.animations.remove(&id);
            let creature_opt = self.layers[MAP_OBJECT_LAYER].remove(&id);

            match creature_opt {
                None => {},
                Some(creature) => {
                    if creature.creature.is_some() {
                        killed_mob_list.push(creature);
                    }    
                }
            }
        }

        (killed_mob_list, transition)
    }


    fn check_pickup(layer: &mut HashMap<u64, MapObject>, position: &Vector2<f32>,
                    inventory: &mut Inventory, player_id: u64,
                    speaker: &mut SoundPlayer) 
    {
        let option = Map::find_nearest_object(layer, position, 100.0, player_id);
        match option {
            None => {},
            Some(key) => {
                let mob_option = layer.get(&key);
                match mob_option {
                    None => {},
                    Some(mob) => {

                        println!("Found a map object: {}, item option is {:?}", mob.uid, mob.item);

                        if mob.item.is_some() {
                            speaker.play(Sound::Click, 0.5);
                            let mob = layer.remove(&key);
                            let item = mob.unwrap().item;
                            inventory.put_item(item.unwrap(), Slot::Bag);
                        }
                    }
                }
            }
        }

        // inventory.print_contents();
    }


    pub fn check_player_transition(&mut self) -> Option<usize> 
    {
        let player_pos = self.layers[MAP_OBJECT_LAYER].get(&self.player_id).unwrap().position;
        let mut best_index = None;
        
        for i in 0 .. self.transitions.len() {
            let transit = &self.transitions[i];
            let v = vec2_sub(player_pos, transit.from);
            let d = vec2_square_len(v);
            if d < transit.rad * transit.rad {
                best_index = Some(i);
            }
        }

        // println!("Checked {} transitions, best is {:?}", self.transitions.len(), best_index);

        best_index
    }


    fn handle_projectile_hit(&mut self, projectile_uid: u64, target_uid: u64, rng: &RngReceiver, speaker: &mut SoundPlayer) -> bool {

        let projectile_type = self.layers[MAP_OBJECT_LAYER].get_mut(&projectile_uid).unwrap().mob_type;
        let target = self.layers[MAP_OBJECT_LAYER].get_mut(&target_uid).unwrap();

        // projectiles can only hit "the enemy" or obstacles on the map
        if projectile_type == MobType::PlayerProjectile &&
            target.mob_type == MobType::Player {
            // no, we do not shoot ourselves into the foot
            return false;
        }

        if projectile_type == MobType::CreatureProjectile && 
           target.mob_type == MobType::Creature {
            return false;
        }

        // what to do about projectile-projectile hits?
        if (projectile_type == MobType::CreatureProjectile ||
            projectile_type == MobType::PlayerProjectile) && 
           (target.mob_type == MobType::CreatureProjectile ||
            target.mob_type == MobType::PlayerProjectile) {
            return false;
        } 
    

        println!("Handle projectile hit on {}", target.uid);
        let sparks = [403, 404, 1993, 1994, 1995, 1996, 1997];

        let z_off = target.visual.z_off * target.visual.scale * 0.5;
        let creature_opt = &mut target.creature;

        if creature_opt.is_some() {
           let creature = creature_opt.as_mut().unwrap();

           if projectile_type == MobType::PlayerProjectile && 
              target.mob_type == MobType::Creature &&
              creature.hit_points > 0 {

                speaker.play(Sound::FireballHit, 0.5);

                for _i in 0..10 {
                    let xv = rng.random() * 2.0 - 1.0;
                    let yv = rng.random() * 2.0 - 1.0;
                    let zv = rng.random();

                    let color = [0.8 + rng.random() * 0.4, 0.5 + rng.random() * 0.4, 0.1 + rng.random() * 0.4];
                    let tile = sparks[rng.random_urange(0, sparks.len())];

                    let speed = if tile == 403 {100.0} else {100.0 + rng.random_frange(1.0, 50.0)};

                    target.visual.particles.add_particle(0.0, 0.0, z_off, xv * speed, yv * speed, zv * speed, 0.7, tile, color);
                    target.visual.color = [0.0, 0.0, 0.0, 0.0];
                    
                    let damage = 10; // todo
                    creature.hit_points -= damage;
                }
            
                return true;
            }
        }

        // don't kill the player (yet)
        false
    }


    pub fn place_item(&mut self, item: Item, position: Vector2<f32>) -> u64{

        // first we need a map object to anchor the item
        let layer = MAP_OBJECT_LAYER;
        let scale = 1.0;
        let height = 0.0;
        let mut mob = self.factory.create_mob(item.map_tile_id, layer, position, height, scale);
        let mob_id = mob.uid;

        // now add the item to the map object
        mob.visual.tileset_id = 6;
        mob.visual.base_image_id = item.map_tile_id;  
        mob.visual.current_image_id = item.map_tile_id + 
                                      Item::calc_image_offset_for_stack_size(item.stack_size);
        mob.visual.scale = item.map_scale;
        mob.visual.color = item.color;
        mob.item = Some(item);

        self.layers[layer].insert(mob_id, mob);

        mob_id
    }
    

    pub fn populate(&mut self, _filename: &str, rng: &RngReceiver, position: Vector2<f32>) {

        // let group = self.make_creature_group("Targetting Drone", 5, 9, position, 40.0, rng);
        let group = self.make_creature_group("generated_creature", 5, 9, position, 40.0, rng);
        self.mob_groups.push(group);
    }


    pub fn load(&mut self, filename: &str) {

        // preserve player
        let mut player = self.layers[MAP_OBJECT_LAYER].remove(&self.player_id).unwrap();

        self.clear();

        let mut path = PathBuf::new();
        path.push("resources/maps/");
        path.push(filename);

        println!("Loading map {}", path.display());

        let content = std::fs::read_to_string(path.as_path()).unwrap();
        let mut lines = content.lines();

        lines.next(); // version

        lines.next(); // header start
        self.name = lines.next().unwrap().to_string();
        self.map_image_name = lines.next().unwrap().to_string();
        self.backdrop_image_name = lines.next().unwrap().to_string();
        println!("map name={} image={} backdrop={}", self.name, self.map_image_name, self.backdrop_image_name);
        lines.next(); // header end

        lines.next(); // objects start
        let mut line = lines.next().unwrap();

        let object_end_marker = "end map objects".to_string();
        while object_end_marker != line {
            // println!("line='{}'", line);
            self.load_mob(line);
            line = lines.next().unwrap();
        }

        lines.next(); // transitions start
        line = lines.next().unwrap();

        let transition_end_marker = "end map transitions".to_string();
        while transition_end_marker != line {
            // println!("line='{}'", line);
            self.load_transition(line);
            line = lines.next().unwrap();
        }

        println!("player_id={}", self.player_id);

        // stop player movement
        player.stop_moving();

        self.layers[MAP_OBJECT_LAYER].insert(self.player_id, player);

        // store walkable area
        let p1 = [1250.0, 100.0];
        let p2 = [2500.0, 1250.0];
        let p3 = [1250.0, 2500.0];
        let p4 = [-50.0, 1250.0];

        let area = Polygon::new(LineString::from(vec![(p1[0], p1[1] + 108.0), 
                                                    (p2[0], p2[1] + 108.0), 
                                                    (p3[0], p3[1] + 108.0), 
                                                    (p4[0], p4[1] + 108.0)]),
                                                    vec![]);

        self.walkable.push(area);
    }


    fn load_mob(&mut self, line: &str) {
        let parts: Vec<&str> = line.split(",").collect();

        let layer = parts[0].parse::<usize>().unwrap();
        let tile_id = parts[1].parse::<usize>().unwrap();
        let directions = parts[2].parse::<usize>().unwrap();

        let x = parts[3].parse::<f32>().unwrap();
        let y = parts[4].parse::<f32>().unwrap();
        let z_off = parts[5].parse::<f32>().unwrap();
        let scale = parts[6].parse::<f32>().unwrap();

        // parts[7] is an RGBA tuple
        let color = parse_rgba(parts[7]);
        let blend = key_to_blend(parts[8]);

        println!("{}, {}, {}, {}, {}, {}, {:?}, {:?}", layer, tile_id, x, y, z_off, scale, color, blend);

        let mut mob = self.factory.create_mob(tile_id, layer, [x, y], z_off, scale);
        mob.visual.color = color;
        mob.visual.blend = blend;
        mob.visual.directions = directions;

        self.layers[layer].insert(mob.uid, mob);
    }


    fn load_transition(&mut self, line: &str) 
    {
        let mut parts = line.split(",");

        let x = parts.next().unwrap().parse::<f32>().unwrap();
        let y = parts.next().unwrap().parse::<f32>().unwrap();
        let r = parts.next().unwrap().parse::<f32>().unwrap();
        
        let dest_str = parts.next().unwrap();
        let c = dest_str.chars().next().unwrap();
        let destination;

        if c.is_ascii_digit() {
            let map_id = dest_str.parse::<i32>().unwrap();
            let dest_x = parts.next().unwrap().parse::<f32>().unwrap();
            let dest_y = parts.next().unwrap().parse::<f32>().unwrap();
            let to_location = [dest_x, dest_y];

            destination = TransitionDestination::Map {to_map: map_id, to_location};
        }
        else {
            let shop = Shop::new();
            let index = self.shops.len();
            destination = TransitionDestination::Shop {index};
            self.shops.push(shop);
        }

        self.add_transition([x, y], r, destination);
    }


    pub fn add_transition(&mut self, from: [f32; 2], catchment: f32, 
                          destination: TransitionDestination) {

        self.transitions.push(MapTransition {
            from,
            rad: catchment,
            destination,
        });
    }


    pub fn save(&self, filename: &str) -> Result<()> {
        let mut path = PathBuf::new();
        path.push("resources/maps");
        path.push(filename);

        let f = File::create(path.as_path())?;
        {        
            let mut writer = BufWriter::new(f);

            writer.write("v10\n".as_bytes())?;
            
            writer.write("begin map header\n".as_bytes())?;
            let name = self.name.to_string()  + "\n";
            writer.write(name.as_bytes())?;
            let map_image_name = self.map_image_name.to_string() + "\n";
            writer.write(map_image_name.as_bytes())?;
            let backdrop_image_name = self.backdrop_image_name.to_string() + "\n";
            writer.write(backdrop_image_name.as_bytes())?;
            writer.write("end map header\n".as_bytes())?;

            writer.write("begin map objects\n".as_bytes())?;
            self.save_layer(&mut writer, MAP_GROUND_LAYER)?;
            self.save_layer(&mut writer, MAP_OBJECT_LAYER)?;
            self.save_layer(&mut writer, MAP_CLOUD_LAYER)?;
            writer.write("end map objects\n".as_bytes())?;

            self.save_map_transitions(&mut writer)?
        }

        Ok(())
    }
    
    
    fn save_layer(&self, writer: &mut BufWriter<File>, layer: usize) -> Result<()> {
        let objects = &self.layers[layer];

        for (_key, object) in objects {

            if object.uid != self.player_id {

                let color = object.visual.color; 

                let line = 
                layer.to_string() + "," +
                &object.visual.base_image_id.to_string() + "," +
                &object.visual.directions.to_string() + "," +
                &object.position[0].to_string() + "," +
                &object.position[1].to_string() + "," +
                &object.visual.z_off.to_string() + "," +
                &object.visual.scale.to_string() + "," +
                &color[0].to_string() + " " +
                &color[1].to_string() + " " +
                &color[2].to_string() + " " +
                &color[3].to_string() + "," +            
                &blend_to_key(&object.visual.blend) +
                "\n";
                
                writer.write(line.as_bytes())?;
            }
        }

        Ok(())
    }

    
    fn save_map_transitions(&self, writer: &mut BufWriter<File>) -> Result<()> {
        writer.write("begin map transitions\n".as_bytes())?;

        for transit in &self.transitions {

            let destination =
                match &transit.destination  {
                    TransitionDestination::Map { to_map, to_location } => {
                        to_map.to_string() + "," +
                        &to_location[0].to_string() + "," +
                        &to_location[1].to_string()
                    },
                    TransitionDestination::Shop { index } => {
                        index.to_string()
                    }
                };


            let line = 
                transit.from[0].to_string() + "," +
                &transit.from[1].to_string() + "," +
                &transit.rad.to_string() + "," +
                &destination + "\n";

            writer.write(line.as_bytes())?;
        }

        writer.write("end map transitions\n".as_bytes())?;

        Ok(())
    }

    
    pub fn move_selected_object(&mut self, dx: f32, dy: f32) {        
        if self.has_selection {
            let object = self.layers[self.selected_layer].get_mut(&self.selected_item).unwrap();
            object.position[0] += dx;
            object.position[1] += dy;
        }
    }

    pub fn apply_to_selected_mob<F>(&mut self, func: F)
        where F: FnOnce(&mut MapObject) {
        let mob = self.layers[self.selected_layer].get_mut(&self.selected_item);

        match mob {
            None => {}
            Some(mob) => { func(mob); }
        }
    }


    pub fn make_creatures(&mut self, id: &str, min_count: i32, max_count: i32, center: Vector2<f32>, spacing: f32, scale: f32, rng: &RngReceiver) -> Vec<MapObject> 
    {
        let count = rng.random_irange(min_count, max_count + 1) as usize;

        let mut list: Vec<MapObject> = Vec::with_capacity(count);
    
        for _i in 0 .. count {

            let creature = self.creature_factory.create(id);
            let mut tries = 0;
            
            // don't place mobs in the same spot if possible
            // 10 tries will be made to find a clear spot
            loop {
                let x = center[0] + spacing * (rng.random() * 10.0 - 5.0);
                let y = center[1] + spacing * (rng.random() * 10.0 - 5.0);
    
                let mut ok = true;
                for mob in &list {

                    let dx = mob.position[0] - x;
                    let dy = mob.position[1] - y;
                    let d = dx * dx + dy * dy;
                    
                    // must be at least 20 units from each other
                    ok = d > 20.0 * 20.0;
                }

                tries += 1;

                if ok {
                    let mut mob = self.factory.create_mob(creature.base_tile_id, CREATURE_TILESET, [x, y], 32.0, scale);
                    mob.visual.directions = creature.frames;
                    mob.visual.blend = creature.blend_mode;
                    mob.mob_type = MobType::Creature;
                    mob.creature = Some(creature);
                    mob.animation_timer = rng.random(); // otherwise all start with the very same frame
                    list.push(mob);

                    break; 
                }
                
                if tries > 10 { 
                    break; 
                }
            }
        }
    
        list
    }

    
    pub fn make_creature_group(&mut self, id: &str, min_count: i32, max_count: i32, center: Vector2<f32>, spacing: f32, rng: &RngReceiver) -> MobGroup {
        
        println!("Placing creatures at {}, {}", center[0], center[1]);

        let mut mobs = self.make_creatures(id, min_count, max_count, center, spacing, 0.5, rng);
        let mut list = Vec::new();

        for i in (0..mobs.len()).rev() {
            let mob = mobs.remove(i);
            let id = mob.uid;

            self.layers[MAP_OBJECT_LAYER].insert(id, mob);
            list.push(id);
        
            self.animations.insert(id, Box::new(SpinAnimation::new(12.0)));
        }

        MobGroup::new(list, center, true, rng)
    }
}


fn emit_drive_particles(mob: &mut MapObject, dt: f32, rng: &RngReceiver) {

    let direction = vec2_scale(mob.velocity, -1.0);
    let rad = 0.05;

    let chance_per_second = 20.0;
    let chance = chance_per_second * dt;

    if rng.random() < chance {
                
        let xp = direction[0] * rad + direction[1] * (rng.random() * 2.0 - 1.0) * 0.05;
        let yp = direction[1] * rad + direction[0] * (rng.random() * 2.0 - 1.0) * 0.05;                

        let xv = direction[0] + rng.random() * 2.0 - 1.0;
        let yv = direction[1] + rng.random() * 2.0 - 1.0;
        
        let zv = (rng.random() *2.0 - 1.0) * 0.15;
        let speed = 1.0;

        let spark = 1993 + (rng.random() * 5.0) as usize;

        mob.visual.particles.add_particle(xp, yp, 25.0, xv * speed, yv * speed, zv * speed, 1.0, spark, [0.5, 0.8, 1.0]);
    }
}


pub fn move_mob(mob: &mut MapObject, destination: Vector2<f32>, base_speed: f32) {

    let direction = vec2_sub(destination, mob.position);

    println!("creature {} moves in direction {:?}", mob.uid, direction);

    let distance = vec2_len(direction);
    let time = distance / base_speed; // pixel per second

    mob.move_time_left = time;
    mob.velocity = vec2_scale(direction, 1.0 / time);

    // let d = mob.visual.orient(direction);
    // mob.visual.current_image_id = mob.visual.base_image_id + d;
    mob.visual.orient_in_direction(direction);
}


fn blend_to_key(blend: &BlendMode) -> String {
    let key =
        match blend {
            BlendMode::Blend => {"n"}, 
            BlendMode::Add => {"a"},
            // _ => {panic!("Unsupported blend mode")},
        };

    key.to_string()
}


fn key_to_blend(key: &str) -> BlendMode {

    println!("key='{}'", key);

    if key == "n" {
        BlendMode::Blend
    } else if key == "a" {
        BlendMode::Add
    } else {
        BlendMode::Blend
    }
}


pub struct MapObject {

    pub mob_type: MobType,
    pub uid: u64,
    pub visual: Visual,
    pub creature: Option<Creature>,
    pub item: Option<Item>,

    // world coordinates of this object. Note that screen coordinates are different
    pub position: Vector2<f32>,
    pub velocity: Vector2<f32>,

    pub move_time_total: f32,
    pub move_time_left: f32,

    pub move_end_action: MoveEndAction,
    pub update_action: UpdateAction,
    pub animation_timer: f32,
}


impl MapObject 
{   
    pub fn move_dt(&mut self, dt: f32) 
    {
        if self.move_time_left > 0.0 {
            let distance = vec2_scale(self.velocity, dt);
            let z_off;
            if self.creature.is_some() {
                let creature = self.creature.as_ref().unwrap();
                z_off = (creature.movement_function)(self.move_time_total - self.move_time_left);
            }
            else {
                z_off = 0.0;
            }
            
            self.position = vec2_add(self.position, distance);
            self.move_time_left -= dt;
            self.visual.z_off = z_off;

            // println!("z_off={}", self.visual.z_off);
        }
    }


    pub fn stop_moving(&mut self)
    {
        self.move_time_total = 0.0;
        self.move_time_left = 0.0;
        self.visual.z_off = 0.0;
    }
}


pub struct MapObjectFactory {
    next_id: u64,
}


impl MapObjectFactory 
{
    pub fn create_mob(&mut self, tile_id: usize, tileset_id: usize, position: Vector2<f32>, z_off: f32, scale: f32) -> MapObject 
    {
        let visual = Visual {
            base_image_id: tile_id,
            current_image_id: tile_id,
            directions: 8,
            phases: 1,
            tileset_id,
            z_off,
            scale,
            color: WHITE,
            glow: WHITE,
            blend: BlendMode::Blend,
            particles: ParticleDriver::new(),
        };

        let uid = self.next_id;
        self.next_id += 1;

        // println!("MapObjectFactory: next id will be {}", self.next_id);

        MapObject {
            mob_type: MobType::MapObject,
            uid,
            visual,
            creature: None,
            item: None,

            position, 
            velocity: [0.0, 0.0],
            move_time_total: 0.0,
            move_time_left: 0.0,

            move_end_action: MoveEndAction::None,
            update_action: UpdateAction::None,
            animation_timer: 0.0,
        }
    }
}


#[derive(PartialEq)]
pub enum MoveEndAction {
    None,
    RemoveFromMap,
    PickItemsUp,
}


#[derive(PartialEq)]
pub enum UpdateAction {
    None,
    RemoveFromMap,
    EmitDriveParticles,
}


pub struct Visual {
    pub base_image_id: usize,
    pub current_image_id: usize,
    pub directions: usize,
    pub phases: usize, // animation phases per direction 
    pub tileset_id: usize,
    pub z_off: f32,
    pub scale: f32,
    pub color: [f32; 4],
    pub glow: [f32; 4], // ground illumination color
    pub blend: BlendMode,
    pub particles: ParticleDriver,
}


impl Visual {
    pub fn orient(&self, direction: Vector2<f32>) -> usize {
        let directions = self.directions as f32;
        let mut result = 0;

        if direction[0] != 0.0 && direction[1] != 0.0 {
            // calculate facing
            let mut r = direction[1].atan2(direction[0]);
            
            // round to a segment
            r = r + PI + PI * 2.0 / directions;
        
            // calculate tile offsets from 0 to directions-1

            let f = (r * directions)  / (PI * 2.0) - 0.5;

            result = self.directions/2 + f.floor() as usize;

            if result >= self.directions {
                result = result - self.directions;
            }

            // println!("dx={} dy={} r={} directions={}", direction[0], direction[1], result, directions);
        } 
        else {
            // error case, zero length move
            println!("Error: Cannot orient mob by zero length direction");
        }

        result
    }


    pub fn orient_in_direction(&mut self, direction: Vector2<f32>) {
        let offset = self.orient(direction);
        self.current_image_id = self.base_image_id + offset;

        println!("orient_in_direction(): dx={} dy={} base={} current={} offset={} directions={}", direction[0], direction[1], self.base_image_id, self.current_image_id, offset, self.directions);
    }
}


#[derive(PartialEq, Clone, Copy)]
pub enum MobType 
{
    MapObject,
    Player,
    Creature,
    PlayerProjectile,
    CreatureProjectile,
}

pub enum TransitionDestination 
{
    Map {
        // destination map
        to_map: i32,

        to_location: Vector2<f32>,
    },
    Shop {
        index: usize,
    }, 
}

pub struct MapTransition 
{

    // entrance location
    from: Vector2<f32>,
    
    // catchment area
    rad: f32,

    pub destination: TransitionDestination,
}