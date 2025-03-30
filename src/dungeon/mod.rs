use std::collections::HashMap;
use std::cmp;

use vecmath::Vector2;
use rand::prelude::*;
use geo::LineString;
use geo::Polygon;

use crate::MAP_GROUND_LAYER;
use crate::MAP_OBJECT_LAYER;
use crate::Map;
use crate::ItemFactory;
use crate::item::Item;


pub struct Room {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}


pub struct Dungeon {
    pub start_position: [f32; 2],

    pub rooms: Vec<Room>,
    pub corridors: Vec<HashMap<i32, [i32; 2]>>,
}


pub fn generate_dungeon(map: &mut Map, factory: &mut ItemFactory) -> Dungeon {

/*
    let layer = MAP_GROUND_LAYER;
    let height = 0.0;
    let id = 50;
    let mut pos = [1000.0, 1000.0];

    // let height = world.layer_tileset[layer].tiles_by_id.get(&id).unwrap().foot[1];
    
    create_mob(map, id, layer, pos, height, 1.0);

    pos[0] += 108.0;
    pos[1] += 108.0;

    create_mob(map, id, layer, pos, height, 1.0);
*/

    let mut rng = rand::rng();

    // place_floor_tile(map, -5 + 5, 5 + 5);
    // build_winded_corridor(map, &mut rng, 0, 0, 10, 10);

    let dungeon = rooms_and_corridors(map, factory, &mut rng);

    furnish_dungeon(&dungeon, map, factory, &mut rng);
    dungeon
}


fn rooms_and_corridors<R: Rng + ?Sized>(map: &mut Map, factory: &mut ItemFactory, rng: &mut R) -> Dungeon {

    let mut corridors: Vec<HashMap<i32, [i32; 2]>> = Vec::new();
    let mut rooms: Vec<Room> = Vec::new();

    let mut entrances: [i32; 16 * 8] = [0; 16 * 8];

    for ry in 0 .. 4 {
        for rx in 0 .. 4 {
            let x = rx * 13 + rng.random_range(-3..3);
            let y = ry * 13 + rng.random_range(-3..3);

            let x1 = x - rng.random_range(1..3);
            let y1 = y - rng.random_range(1..3); 
            let x2 = x + rng.random_range(1..3);
            let y2 = y + rng.random_range(1..3);

            // keep track of entrances

            // start index of the room data in the array. 4 coordinates, 2 values each
            let room: usize = ((ry * 4 + rx) * 8) as usize;

            entrances[room + 0] = x;
            entrances[room + 1] = y1;

            entrances[room + 2] = x2;
            entrances[room + 3] = y;

            entrances[room + 4] = x;
            entrances[room + 5] = y2;

            entrances[room + 6] = x1;
            entrances[room + 7] = y;

            build_room(map, factory, rng, x1, y1, x2, y2, &entrances[room .. room + 8]);

            rooms.push(Room {
                x1, x2, y1, y2,
            })
        }
    }

    // to avoid double tiles and easier wall placement in corridors we first collect all
    // floor coordinates and then actually build the corridor

    let mut floors: HashMap<i32, [i32; 2]>; 

    for ry in 0 .. 4 {
        for rx in 0 .. 4 {

            let room = (ry * 4 + rx) * 8;
            
            // "down right" corridors

            floors = HashMap::new();

            if ry < 3 {
                // straight starting stubs
                build_straight_corridor(&mut floors,
                    entrances[room + 4], //  = x;
                    entrances[room + 5] + 1, //  = b;
                    entrances[room + 4], //  = x;
                    entrances[room + 5] + 2, //  = b;
                    0
                );

                build_straight_corridor(&mut floors,
                    entrances[room + 8 * 4 + 0], //  = x;
                    entrances[room + 8 * 4 + 1] - 1, //  = b;
                    entrances[room + 8 * 4 + 0], //  = x;
                    entrances[room + 8 * 4 + 1] - 2, //  = b;
                    0
                );

                // now the windy connection
                let wriggle_prob = rng.random_range(0.1 .. 1.0);
                build_winded_corridor(map, &mut floors, rng, 
                    entrances[room + 4], //  = x;
                    entrances[room + 5] + 2, //  = b;
        
                    entrances[room + 8 * 4], //  = x;
                    entrances[room + 8 * 4 + 1] - 2, //  = t;

                    wriggle_prob);
            }
           
            build_tunnel_from_coordinates(map, &floors);
            corridors.push(floors);

            // "up right" corridors

            floors = HashMap::new();

            if rx < 3 {
                // straight starting stubs
                
                build_straight_corridor(&mut floors,
                    entrances[room + 2] + 1,
                    entrances[room + 3],
                    entrances[room + 2] + 2,
                    entrances[room + 3],
                    0,
                );

                build_straight_corridor(&mut floors,
                    entrances[room + 8 + 6] - 1,
                    entrances[room + 8 + 7],
                    entrances[room + 8 + 6] - 2,
                    entrances[room + 8 + 7],
                    0,
                );
    
                let wriggle_prob = rng.random_range(0.1 .. 1.0);
                build_winded_corridor(map, &mut floors, rng, 
                    entrances[room + 2] + 2,
                    entrances[room + 3],
                    entrances[room + 8 + 6] - 2,
                    entrances[room + 8 + 7],

                    wriggle_prob);
            }

            // build_tunnel_from_coordinates(map, &floors);
            build_corridor_from_coordinates(map, &floors);
            corridors.push(floors);
        }
    }

    Dungeon {
        start_position: map_pos(rooms[0].x2 - 1, rooms[0].y1 + 1, 0, 1.0),
        rooms, 
        corridors,
    }
}


fn furnish_dungeon<R: Rng + ?Sized>(dungeon: &Dungeon, 
                                    map: &mut Map, factory: &mut ItemFactory, rng: &mut R) {

    // place entrance stairs
    place_wall_tile(map, dungeon.rooms[0].x2, dungeon.rooms[0].y1, 
                    0, 248, [1.0, 1.0, 1.0, 1.0]);

    for i in 1 .. dungeon.rooms.len() {
        place_coins(map, factory, 
                    rng.random_range(dungeon.rooms[i].x1 .. dungeon.rooms[i].x2), 
                    rng.random_range(dungeon.rooms[i].y1 .. dungeon.rooms[i].y2), 
                    "copper_coin", rng.random_range(1 .. 6));
    }

    for room in &dungeon.rooms {
    
        let p1 = map_pos(room.x1, room.y1, 0, 1.0);
        let p2 = map_pos(room.x2, room.y1, 0, 1.0);
        let p3 = map_pos(room.x2, room.y2, 0, 1.0);
        let p4 = map_pos(room.x1, room.y2, 0, 1.0);
    
        let area = Polygon::new(LineString::from(vec![(p1[0], p1[1]), (p2[0], p2[1]), (p3[0], p3[1]), (p4[0], p4[1])]), vec![]);

        map.walkable.push(area);
    }
}


fn build_room<R: Rng + ?Sized>(map: &mut Map, factory: &mut ItemFactory, rng: &mut R, 
                               sx: i32, sy: i32, dx: i32, dy: i32,
                               entrances: &[i32]) {

    let wall_color = [1.0, 1.0, 1.0, 1.0];

    for y in sy .. dy + 1 {
        for x in sx .. dx + 1 {
            place_floor_tile(map, x, y, rng.random_range(51..=53), [0.97, 0.92, 0.9, 1.0]);
        }
    }

    // tall back walls

    // left
    for x in sx .. dx + 1 {
        if sy < 3 || entrances[4] != x {
            place_wall_tile(map, x-1, sy, -226, 494, wall_color);
        }
        else {
            // the walls need end pieces ...

            // left corner
            place_wall_tile(map, x, sy, -118, 498, wall_color);

            // right corner
            place_wall_tile(map, x-1, sy, -118, 501, wall_color);
        }
    }

    // right
    for y in sy .. dy + 1 {
        if dx > 12 * 3 - 6 || entrances[3] != y {
            place_wall_tile(map, dx, y+1, -226, 495, wall_color);
        }
        else {
            // the walls need end pieces ...

            // left corner
            place_wall_tile(map, dx, y+1, -118, 498, wall_color);

            // right corner
            place_wall_tile(map, dx, y, -118, 501, wall_color);
        }
    }

    // short front walls
    
    // right
    for x in sx .. dx + 1 {
        if dy > 12 * 3 - 6 || entrances[0] != x {
            place_wall_tile(map, x+1, dy-1, 100, 497, wall_color);
        }
    }

    // left
    for y in sy .. dy + 1 {
        if sx < 3 || entrances[7] != y {
            place_wall_tile(map, sx, y-1, -6, 496, wall_color);
        }
    }

    // left room corner
    place_wall_tile(map, sx, sy-1, 98, 498, wall_color);

    // right room corner
    place_wall_tile(map, dx+1, dy, 98, 501, wall_color);
}


fn build_winded_corridor<R: Rng + ?Sized>(map: &mut Map, floors: &mut HashMap<i32, [i32; 2]>, rng: &mut R, 
                                          sx: i32, sy: i32, dx: i32, dy: i32,
                                          wriggle_prob: f64) {
    // is this straight?

    if sx == dx || sy == dy {
        // straight corridor

        subdivide_corridor(map, floors, rng, sx, sy, dx, dy, wriggle_prob);
    }
    else {
        // L-shaped corridor, split it into two straight parts

        // two options to chose

        if rng.random() {
            subdivide_corridor(map, floors, rng, sx, sy, sx, dy, wriggle_prob);
            subdivide_corridor(map, floors, rng, sx, dy, dx, dy, wriggle_prob);
        }
        else {
            subdivide_corridor(map, floors, rng, sx, sy, dx, sy, wriggle_prob);
            subdivide_corridor(map, floors, rng, dx, sy, dx, dy, wriggle_prob);    
        }
    }
}


fn subdivide_corridor<R: Rng + ?Sized>(map: &mut Map, floors: &mut HashMap<i32, [i32; 2]>, rng: &mut R,
                                       sx: i32, sy: i32, dx: i32, dy: i32,
                                       wriggle_prob: f64) {
    let vx = (dx - sx).signum();
    let vy = (dy - sy).signum();

    if vx != 0 && vy != 0 {
        panic!("Diagonal corridor {}, {}", vx, vy);
    }

    let n = cmp::max((dx - sx).abs(), (dy - sy).abs());
    let p: f64 = rng.random();

    if n < 6 || p > wriggle_prob{
        // too short to be wriggled. Build straight, include end piece
        build_straight_corridor(floors, sx, sy, dx, dy, 1);
    }
    else {
        let min = 2;
        let max = n - 2;

        // start piece
        build_straight_corridor(floors, sx, sy, sx + min * vx, sy + min * vy, 0);

        // depth of turn
        let d:i32 = rng.random_range(-n/2 .. n/2);

        // U turn

        subdivide_corridor(map, floors, rng, 
                           sx + min * vx, sy + min * vy, 
                           sx + min * vx + d * vy, sy + min * vy + d * -vx,
                           wriggle_prob);

        subdivide_corridor(map, floors, rng, 
                           sx + min * vx + d * vy, sy + min * vy + d * -vx, 
                           sx + max * vx + d * vy, sy + max * vy + d * -vx,
                           wriggle_prob);

        subdivide_corridor(map, floors, rng, 
                           sx + max * vx + d * vy, sy + max * vy + d * -vx, 
                           sx + max * vx, sy + max * vy,
                           wriggle_prob);

        // end piece
        build_straight_corridor(floors, sx + max * vx, sy + max * vy, dx, dy, 1);
    }
}


/**
 * Builds a straight corridor.
 *
 * @param end The end piece of the corridor will be omitted of end is zero. Pass one to place the end piece too
 */
fn build_straight_corridor(floors: &mut HashMap<i32, [i32; 2]>,
                           sx: i32, sy: i32, dx: i32, dy: i32, end: i32) {
    let vx = (dx - sx).signum();
    let vy = (dy - sy).signum();

    let n = cmp::max((dx - sx).abs(), (dy - sy).abs()) + end;
    let mut x = sx;
    let mut y = sy;

    for _i in 0..n {
        add_floor_coordinate(floors, x, y);

        x += vx;
        y += vy;
    }
}


fn floor_key(x: i32, y: i32) -> i32 {
    y * 1000 + x
}


fn add_floor_coordinate(floors: &mut HashMap<i32, [i32; 2]>, x: i32, y: i32) {
    let key = floor_key(x, y);
    floors.insert(key, [x, y]);
}


fn build_tunnel_from_coordinates(map: &mut Map, floors: &HashMap<i32, [i32; 2]>)
{
    // let wall_color = [0.63, 0.64, 0.65, 1.0];
    // let floor_color = [0.38, 0.36, 0.33, 1.0];
    let wall_color = [0.53, 0.54, 0.55, 1.0];
    let floor_color = [0.32, 0.30, 0.27, 1.0];

    for (_key, value) in floors {
        let x = value[0];
        let y = value[1];

        place_floor_tile(map, x, y, 47, floor_color);

        // check connections to neighboring floor tiles
        let north = floors.get(&floor_key(x+1, y)).is_some();
        let south = floors.get(&floor_key(x-1, y)).is_some();

        let east = floors.get(&floor_key(x, y+1)).is_some();
        let west = floors.get(&floor_key(x, y-1)).is_some();

        // count connections, we need to leave end pieces without wall into a room
        let mut connections = 0;
        if north { connections += 1};
        if south { connections += 1};
        if east { connections += 1};
        if west { connections += 1};

        let end_piece = connections == 1;

        if end_piece {
            println!("End piece detected at {}, {}", x, y);
        }

        // placement helper
        // place_wall_tile(map, x, y+1, 0, 692, [1.0, 1.0, 1.0, 1.0]);

        // place walls if there is no connection

        // back walls
        // left
        if !west && !(end_piece && east) {             
            place_wall_tile(map, x-1, y, -202, 512, wall_color);  
        }
        
        // right
        if !north && !(end_piece && south) {
            place_wall_tile(map, x+1, y, 20, 513, wall_color);
        }

        // front walls

        // left
        if !south && !(end_piece && north) {
            place_wall_tile(map, x, y, 60, 515, wall_color);
        }

        // right
        if !east && !(end_piece && west) {
            place_wall_tile(map, x, y, 110, 514, wall_color);  
        }
    }
}

fn build_corridor_from_coordinates(map: &mut Map, floors: &HashMap<i32, [i32; 2]>)
{
    let floor_color = [0.68, 0.66, 0.63, 1.0];
    let wall_color = [0.53, 0.54, 0.55, 1.0];

    for (_key, value) in floors {
        let x = value[0];
        let y = value[1];

        place_floor_tile(map, x, y, 48, floor_color);

        // check connections to neighboring floor tiles
        let north = floors.get(&floor_key(x+1, y)).is_some();
        let south = floors.get(&floor_key(x-1, y)).is_some();

        let east = floors.get(&floor_key(x, y+1)).is_some();
        let west = floors.get(&floor_key(x, y-1)).is_some();

        // count connections, we need to leave end pieces without wall into a room
        let mut connections = 0;
        if north { connections += 1};
        if south { connections += 1};
        if east { connections += 1};
        if west { connections += 1};

        let end_piece = connections == 1;

        if end_piece {
            println!("End piece detected at {}, {}", x, y);
        }

        // place walls if there is no connection

        // back walls
        // left
        if !west && !(end_piece && east) {             
            place_wall_tile(map, x-1, y, -202, 519, wall_color);  
        }
        
        // right
        if !north && !(end_piece && south) {
            place_wall_tile(map, x+1, y, 18, 518, wall_color);
        }

        // front walls

        // left
        if !south && !(end_piece && north) {
            place_wall_tile(map, x, y, 30, 503, wall_color);
        }

        // right
        if !east && !(end_piece && west) {
            place_wall_tile(map, x, y, 94, 509, wall_color);  
        }
    }
}

fn place_floor_tile(map: &mut Map, x: i32, y: i32, id: usize, color: [f32; 4]) {
    let layer = MAP_GROUND_LAYER;
    let height = 0.0;
    let scale = 1.0;
    let pos = map_pos(x, y, 0, scale);

    let mob_id = create_mob(map, id, layer, pos, height, scale);

    let mob = map.layers[layer].get_mut(&mob_id).unwrap();

    mob.visual.color = color;
}


fn place_wall_tile(map: &mut Map, x: i32, y: i32, z_off: i32, id: usize, color: [f32; 4]) {
    let layer = MAP_OBJECT_LAYER;
    let height = 0.0;
    let scale = 1.0;
    let pos = map_pos(x, y, z_off, scale);

    let mob_id = create_mob(map, id, layer, pos, height, scale);
    let mob = map.layers[layer].get_mut(&mob_id).unwrap();

    mob.visual.color = color;
}


fn place_coins(map: &mut Map, factory: &mut ItemFactory,
               x: i32, y: i32, id: &str, count: u32) -> u64 {
    let layer = MAP_OBJECT_LAYER;
    let height = 0.0;
    let scale = 1.0;
    let pos = map_pos(x, y, 0, scale);

    let mob_id = create_mob(map, 0, layer, pos, height, scale);

    let mob = map.layers[layer].get_mut(&mob_id).unwrap();

    let mut item = factory.create(id);
    item.stack_size = count;

    mob.visual.tileset_id = 6;
    mob.visual.base_image_id = item.map_tile_id;  
    mob.visual.current_image_id = item.map_tile_id + Item::calc_image_offset_for_stack_size(count);
    mob.visual.scale = item.map_scale;
    mob.item = Some(item);
    mob_id
}


fn create_mob(map: &mut Map, tile_id: usize, layer: usize, position: Vector2<f32>, height: f32, scale: f32) -> u64 {
    let mob = map.factory.create_mob(tile_id, layer, position, height, scale);
    let mob_id = mob.uid;
    map.layers[layer].insert(mob_id, mob);

    mob_id
}


fn map_pos(x: i32, y: i32, z_off: i32, scale: f32) -> [f32; 2] {

    let fx = ((y + x) * 108) as f32; 
    let fy = ((y - x) * 108 + z_off) as f32;

    // println!("{}, {} -> {}, {}", x, y, fx, fy);

    [fx * scale, fy * scale]
}