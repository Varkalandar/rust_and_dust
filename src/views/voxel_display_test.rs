use std::f32::consts::PI;

use rand::Rng;

use image::RgbaImage;

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
use crate::gfx::gl_support::load_image;
use crate::gfx::c_mul;
use crate::gfx::c_add;


pub struct VoxelDisplayTest
{
    pub result: Texture2d,
    pub vector_ball: Framebuffer,
    pub soft_pen: Framebuffer,
}


impl VoxelDisplayTest
{
    pub fn new<T: SurfaceTypeTrait + ResizeableSurface>(display: &Display<T>) -> VoxelDisplayTest
    {
        let soft_pen = Framebuffer::from_image("resources/gfx/ui/soft_pen.png");
        let vector_ball = Framebuffer::from_image("resources/gfx/ui/vector_ball.png");
        let fb = generate_fb_image(&soft_pen);

        VoxelDisplayTest {
            result: fb.to_texture(display),
            vector_ball,
            soft_pen,
        }
    }
}


pub fn generate_creature<T: SurfaceTypeTrait + ResizeableSurface>(display: &Display<T>, 
                                                                  tileset: &mut TileSet,
                                                                  pen: &Framebuffer) -> CreaturePrototype
{
    let fb = generate_fb_image(pen);
    let tile_id = tileset.get_new_id();

    let tile = Tile {
        id: tile_id,
        size: [fb.width as f32, fb.height as f32],
        foot: [fb.width as f32 * 0.5, fb.height as f32 * 0.75],
        tex: fb.to_texture(display),
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


pub fn generate_fb_image(pen: &Framebuffer) -> Framebuffer
{
    let fb_size = 128;
    let mut fb = Framebuffer::new(fb_size, fb_size);

    // fb.fill_box(0, 0, 256, 256, [255, 255, 255, 255]);
    // fb.fill_box(10, 10, 236, 236, [0, 0, 0, 255]);

    // let mut voxels = generate_sphere(0.0, 64.0, 0.0, 50.0, 24, [192, 192, 128, 255]);
    let mut voxels = generate_tendrils(fb_size, 0.5 * 4.0, 0.7 * 4.0, 7..9, 23, [128, 160, 64], [2, 10, 1]);
    voxels.merge(generate_tendrils(fb_size, 1.0 * 4.0, 0.1 * 4.0, 15..17, 9, [192, 192, 128], [3, 3, 2]));

    voxels.sort_depth_first();

    // scan for max and min depth
    let mut max_depth = 0.0;
    let mut min_depth = 100.0;

    for voxel in &voxels.voxels {
        if voxel.z < min_depth {min_depth = voxel.z;}
        if voxel.z > max_depth {max_depth = voxel.z;}
    }

    // for the dot sizes, norm the depth to 0..1
    let z_scale = 1.0 / (max_depth - min_depth);

    println!("min_z={} max_z={} z_scale={}", min_depth, max_depth, z_scale);

    for voxel in &voxels.voxels {

        // voxels are around x=0 and z=0, we need to shift them to half the framebuffer width
        let xp = voxel.x as i32 + fb.width / 2;
        let yp = (voxel.y - voxel.z * 0.5) as i32;
        let size = std::cmp::min(((voxel.z - min_depth) * z_scale) as i32 + 1, 7);
        
        // vballs have some size, do some bounds checking here
        if xp > 2 && yp > 2 && xp < fb.width - 2 && yp < fb.height - 2 {
            // pen_at_size(&mut fb, xp, yp, pen, size + 2, voxel.color);
            pen.draw_scaled(&mut fb, xp, yp, size + 2, size + 2, voxel.color);
        }
        else {
            println!("pen position is out of bounds: xp={} yp={} size={}", xp, yp, size);
        }
    }


    // debug
    /*
    for y in 0 .. 128 {
        for x in 0 .. 128 {
            let source_x = x;
            let source_y = y;

            let pixel = vector_ball.get_pixel(source_x, source_y);

            println!("sx={} sy={} pix={:?}", source_x, source_y, pixel);

            fb.set_pix(x as i32, y as i32, pixel.0);
        }    
    }
    */

    fb
}


fn generate_sphere(xc: f32, yc: f32, zc: f32, rad: f32, steps: i32, color: [u8; 4]) -> Voxelstack
{
    let mut voxels = Voxelstack::new();

    for i in -steps/2 .. steps/2 {

        let fi = (i * 2) as f32 / steps as f32;
        let p = 1.0 - fi * fi;
        let ir = p.sqrt() * rad;

        for n in 0 .. steps {
            let p = PI * 2.0 * n as f32 / steps as f32 + 0.1;

            let x = p.cos() * ir;
            let z = p.sin() * ir;

            voxels.add(Voxel::new(
                xc + x, 
                yc + fi * rad, 
                zc + z + rad,
                1.0, 
                color
            ));
        }
    }

    voxels
}


fn generate_tendrils(fb_size: i32, dy: f32, dx: f32, steps_range: std::ops::Range<u32>, tendrils: i32,
                     start_color: [u8; 3], cv_range_max: [u8; 3]) -> Voxelstack
{
    let mut rng = rand::rng();
    let mut voxels = Voxelstack::new();

    let steps = rng.random_range(steps_range);
    
    let size = fb_size as f32;

    for rotation in 0 .. tendrils {

        let mut x = 0.0;
        let mut y = size * 0.75;
        let mut z = 0.0;
        let mut color = [start_color[0], start_color[1], start_color[2], 255];
        let cv = [rng.random_range(0 .. cv_range_max[0]), rng.random_range(0 .. cv_range_max[1]), rng.random_range(0 .. cv_range_max[2]), 255];

        for _i in 0 .. steps {

            let xv = rng.random::<f32>() * dx;
            let yv = -rng.random::<f32>() * dy;
            let zv = rng.random::<f32>() * dx;
            let steps = rng.random_range(2 .. 4);

            // println!("x={} y={} z={}, steps={}", x, y, z, steps);
            
            (x, y, z, color) = line(&mut voxels, x, y, z, xv, yv, zv, steps, color, cv, 5.0);
        }
        
        voxels.merge(generate_sphere(x, y, z, 3.0, 8, [rng.random_range(color[0]..=255), rng.random_range(color[1]..=255), rng.random_range(color[2]..=255), 255]));

        // spin before adding next tendril
        voxels.rotate_y(PI * 2.0 / tendrils as f32);
    }

    voxels
}


fn line(voxels: &mut Voxelstack,
        x: f32, y: f32, z: f32, xv: f32, yv: f32, zv: f32, steps: i32,
        mut color: [u8; 4], cv: [u8; 4], size: f32) -> (f32, f32, f32, [u8; 4])
{
    let mut xp = x;
    let mut yp = y;
    let mut zp = z;

    for _i in 0 .. steps {
        xp += xv;
        yp += yv;
        zp += zv;

        color[0] = c_add(color[0], cv[0]);
        color[1] = c_add(color[1], cv[1]);
        color[2] = c_add(color[2], cv[2]);

        voxels.add(Voxel::new(xp, yp, zp, 1.0, color));
    }

    (xp, yp, zp, color)
}


fn pen_at_size(fb: &mut Framebuffer, xp: i32, yp: i32, pen: &RgbaImage, size: u32, color: [u8; 4])
{
    for y in 0 .. size {
        for x in 0 .. size {
            let source_x = (x * 128 + 63) / size;
            let source_y = (y * 128 + 63) / size;

            let pixel = pen.get_pixel(source_x, source_y).0;

            // println!("sx={} sy={} pix={:?} size={}", source_x, source_y, pixel, size);

            fb.blend_pix(xp + x as i32, yp + y as i32, 
                         [c_mul(color[0], pixel[0]), 
                          c_mul(color[1], pixel[1]), 
                          c_mul(color[2], pixel[2]), 
                          c_mul(color[3], pixel[3])]);
        }    
    }
}
