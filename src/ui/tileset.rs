use vecmath::Vector2;
use std::{rc::Rc, collections::HashMap, fs::read_to_string, path::{Path, PathBuf}};

use glutin::surface::WindowSurface;
use glium::Display;
use glium::Texture2d;

use crate::load_texture;

pub struct Tile {
    pub id: usize,
    pub size: Vector2<f32>,
    pub foot: Vector2<f32>,
    pub tex: Texture2d,
    pub name: String,
}


pub struct TileSet {
     pub tiles_by_id: HashMap<usize, Rc<Tile>>,
     pub tiles_order_to_id: HashMap<usize, usize>,    
}


impl TileSet {

    /**
     * creates an empty tile set
     */
     /*
    pub fn new() -> TileSet {
        TileSet {
            tiles_by_id: HashMap::new(),
            tiles_order_to_id: HashMap::new(),
        }
    }
    */
    
    pub fn load(display: &Display<WindowSurface>, path_str: &str, file_str: &str) -> TileSet {
        
        let mut fullpath = PathBuf::new();
        fullpath.push(path_str);
        fullpath.push(file_str);
        
        let path = Path::new(fullpath.as_path());    
        let rs = read_to_string(path).unwrap();
        let mut line_vec = Vec::new();
        
        for line in rs.lines() {
            line_vec.push(line);
        }

        println!("Read {} lines from {:?}", line_vec.len(), path);
        
        let mut tileset = TileSet {
            tiles_by_id: HashMap::new(),
            tiles_order_to_id: HashMap::new(),
        };
        
        let mut ordinal = 0;    
        for i in 0..line_vec.len() {
            let line = line_vec[i];
            
            if line.starts_with("Tile Description") {
                
                let tile_opt = load_tile(display, path_str, &line_vec, i);
                
                if tile_opt.is_some() {
                    let tile = tile_opt.unwrap();
                    let id = tile.id;

                    tileset.tiles_by_id.insert(id, Rc::new(tile));
                    tileset.tiles_order_to_id.insert(ordinal, id);
                }
                
                ordinal += 1;
            }
        }

        tileset        
    }
    
    
    pub fn shallow_copy(&self) -> TileSet {
        let mut result = TileSet {
            tiles_by_id: HashMap::new(),
            tiles_order_to_id: HashMap::new(),
        };

        for key in &self.tiles_order_to_id {
            let id = *key.1;
            result.tiles_order_to_id.insert(*key.0, id);

            let tile = self.tiles_by_id.get(&id).unwrap();
            result.tiles_by_id.insert(id, tile.clone());
        }
        
        result
    }
}


fn load_tile(display: &Display<WindowSurface>, path_str: &str, lines: &Vec<&str>, start: usize) -> Option<Tile> {

    let id = lines[start + 2].parse::<usize>().unwrap();

    let mut size = lines[start + 3].split(" ");
    let width = size.next().unwrap().parse::<f32>().unwrap();
    let height = size.next().unwrap().parse::<f32>().unwrap();

    let mut foot = lines[start + 5].split(" ");
    let foot_x = foot.next().unwrap().parse::<f32>().unwrap();
    let foot_y = foot.next().unwrap().parse::<f32>().unwrap();

    let name = lines[start + 11];

    let mut result: Option<Tile> = None;

    if width > 1.0 || height > 1.0 {
        // println!("Item {} is {} size={}x{} foot={}x{}", id, name, width, height, foot_x, foot_y);

        let filename = 
            path_str.to_string() + "/" + &id.to_string() + "-" + name + ".png";
        
        let tex = load_texture(display, &filename);

        result = Some(Tile {
            id,
            size: [width, height],
            foot: [foot_x, foot_y],
            tex,
            name: name.to_string(),            
        });      
    }

    result
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_load_tileset() {
    }
}