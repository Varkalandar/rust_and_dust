use std::f32::consts::PI;

use glutin::surface::ResizeableSurface;
use glutin::surface::SurfaceTypeTrait;
use glium::Display;

use crate::Texture2d;
use crate::gfx::Framebuffer;
use crate::gfx::voxel::Voxel;
use crate::gfx::voxel::Voxelstack;
use crate::creature::CreaturePrototype;
use crate::TileSet;
use crate::Tile;


pub struct VoxelDisplayTest
{
    pub result: Texture2d,
}


impl VoxelDisplayTest
{
    pub fn new<T: SurfaceTypeTrait + ResizeableSurface>(display: &Display<T>) -> VoxelDisplayTest
    {
        let mut fb = Framebuffer::new(800, 600);

        fb.fill_box(0, 0, 800, 600, [255, 255, 255, 255]);
        fb.fill_box(10, 10, 780, 580, [0, 0, 0, 255]);
    
        let mut voxels = Voxelstack::new();

        for i in -12 .. 12 {

            let rad = 150.0;
            let fi = i as f32 / 12.0;
            let p = 1.0 - fi * fi;
            let ir = p.sqrt() * rad;

            for n in 0 .. 24 {
                let p = PI * 2.0 * n as f32 / 24.0 + 0.1;
    
                let x = p.cos() * ir;
                let z = p.sin() * ir;
    
                voxels.add(Voxel::new(
                    x + 400.0, 200.0 + fi * rad, z + rad, [255, 255, 0, 255]
                ));
            }
        }

        voxels.sort_depth_first();

        for voxel in voxels.voxels {
            let xp = voxel.x as i32;
            let yp = (voxel.y + voxel.z * 0.5) as i32;
            let size = std::cmp::min((voxel.z * 0.022) as i32 + 1, 7); 
            fb.vball(xp, yp, size, voxel.color);
            // println!("z = {}", voxel.z)
        }

        VoxelDisplayTest {
            result: fb.to_texture(display),
        }
    }

}

pub fn generate_creature<T: SurfaceTypeTrait + ResizeableSurface>(display: &Display<T>, 
                                                                  tileset: &mut TileSet) -> CreaturePrototype
{
    let fb_size = 128;
    let mut fb = Framebuffer::new(fb_size, fb_size);

    // fb.fill_box(0, 0, 256, 256, [255, 255, 255, 255]);
    // fb.fill_box(10, 10, 236, 236, [0, 0, 0, 255]);

    let mut voxels = Voxelstack::new();
    let steps = 96;

    for i in -steps/2 .. steps/2 {

        let rad = 38.0;
        let fi = (i * 2) as f32 / steps as f32;
        let p = 1.0 - fi * fi;
        let ir = p.sqrt() * rad;

        for n in 0 .. steps {
            let p = PI * 2.0 * n as f32 / steps as f32 + 0.1;

            let x = p.cos() * ir;
            let z = p.sin() * ir;
            let fb_center = fb_size as f32 * 0.5;

            voxels.add(Voxel::new(
                x + fb_center, 
                fb_center + fi * rad, 
                z + rad,
                [255, 255, 128, 255]
            ));
        }
    }

    voxels.sort_depth_first();

    for voxel in voxels.voxels {
        let xp = voxel.x as i32;
        let yp = (voxel.y + voxel.z * 0.5) as i32;
        let size = std::cmp::min((voxel.z * 0.022) as i32 + 1, 7); 
        fb.vball(xp, yp, size, voxel.color);
        // println!("z = {}", voxel.z)
    }

    let texture = fb.to_texture(display);
    let tile_id = tileset.get_new_id();

    let tile = Tile {
        id: tile_id,
        size: [fb_size as f32, fb_size as f32],
        foot: [fb_size as f32 * 0.5, fb_size as f32],
        tex: texture,
        name: "generated_creature".to_string(),
    };

    tileset.add_tile(tile);

    CreaturePrototype {
        base_tile_id: tile_id,
        frames: 1,
        speed: 100.0,
        min_hp: 1,
        max_hp: 2,
        projectile_spawn_distance: 25.0,
    }
}
