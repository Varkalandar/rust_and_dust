use std::f32::consts::PI;

use rand::Rng;

use glutin::surface::ResizeableSurface;
use glutin::surface::SurfaceTypeTrait;
use glium::Display;

use crate::Texture2d;
use crate::creature::CreaturePrototype;
use crate::creature::movement_glide;
use crate::creature::CreatureAnimation;
use crate::TileSet;
use crate::Tile;
use crate::gfx::Framebuffer;
use crate::gfx::voxel::Voxel;
use crate::gfx::voxel::Voxelstack;
use crate::gfx::gl_support::BlendMode;
use crate::gfx::c_add;


pub struct VoxelImageGenerator
{
    pub result: Texture2d,
    pub vector_ball: Framebuffer,
    pub soft_pen: Framebuffer,
}


impl VoxelImageGenerator
{
    pub fn new<T: SurfaceTypeTrait + ResizeableSurface>(display: &Display<T>) -> VoxelImageGenerator
    {
        let soft_pen = Framebuffer::from_image("resources/gfx/ui/soft_pen.png");
        let vector_ball = Framebuffer::from_image("resources/gfx/ui/vector_ball.png");

        // let generator = generate_goblet;
        let generator = generate_scorpion;

        let fb = generate_image(&vector_ball, 1.0, generator);

        VoxelImageGenerator {
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
    let mut tile_id = 0;

    // let generator = generate_goblet;
    let generator = generate_scorpion;

    // create n directions
    let n = 16;
    for i in 0 .. n {
        let fb = generate_image(pen, ((i + n/2) as f32 / n as f32) * PI * -2.0, generator);
        tile_id = tileset.get_new_id();
    
        let tile = Tile {
            id: tile_id,
            size: [fb.width as f32, fb.height as f32],
            foot: [fb.width as f32 * 0.5, fb.height as f32 * 0.75],
            tex: fb.to_texture(display),
            name: "generated_creature".to_string(),
        };

        println!("Adding tile {}", tile_id);
        tileset.add_tile(tile);
    }

    CreaturePrototype {
        base_tile_id: tile_id - n + 1,
        frames: n,
        speed: 100.0,
        min_hp: 1,
        max_hp: 2,
        projectile_spawn_distance: 25.0,

        // blend_mode: BlendMode::Add,
        blend_mode: BlendMode::Blend,
        movement_function: movement_glide,
        animation_type: CreatureAnimation::NONE,
    }
}


pub fn generate_image(pen: &Framebuffer, rot: f32, generator: fn() -> Voxelstack) -> Framebuffer
{
    let fb_size = 128;
    let mut fb = Framebuffer::new(fb_size, fb_size);

    let mut voxels = (generator)();

    voxels.rotate_y(rot);
    voxels.sort_depth_first();

    // scan for max and min depth
    let mut max_depth = 0.0;
    let mut min_depth = 100.0;

    for voxel in &voxels.voxels {
        if voxel.z < min_depth {min_depth = voxel.z;}
        if voxel.z > max_depth {max_depth = voxel.z;}
    }

    // for the dot sizes, norm the depth to 0..1
    let z_scale = if max_depth - min_depth != 0.0 {1.0 / (max_depth - min_depth)} else {1.0};

    println!("min_z={} max_z={} z_scale={}", min_depth, max_depth, z_scale);

    for voxel in &voxels.voxels {

        // voxels are around x=0 and z=0, we need to shift them to half the framebuffer width
        let xp = voxel.x as i32 + fb.width / 2;
        let yp = (voxel.y - voxel.z * 0.5) as i32;

        // norm depth size scale to 0.9 .. 1.1
        let size_f = 1.1 - ((voxel.z - min_depth) * z_scale) * 0.2;
        let size = (size_f * voxel.size + 0.5) as i32;

        // voxels have some size, do some bounds checking here
        if xp > size && yp > size && xp < fb.width - size && yp < fb.height - size {
            pen.draw_scaled(&mut fb, xp - size/2, yp - size/2, size, size, 
                            voxel.color, 
                            |c| -> u8 {
                                let base = c as f32 * PI * 0.5 / 255.0;
                                (base.sin() * base.sin() * 255.0) as u8
                            });
        }
        else {
            println!("pen position is out of bounds: xp={} yp={} size={}", xp, yp, size);
        }
    }

    // debug
    /*
    pen.draw_scaled(&mut fb, 0, 0, pen.width, pen.height, 
        [192, 224, 255, 255], 
        |c| -> u8 {c});
    */
    
    fb
}


fn generate_goblet() -> Voxelstack
{
    let mut voxels = generate_goblet_aux(7, 60.0, 40.0, [128, 160, 192, 255]);
    voxels.merge(generate_goblet_aux(5, 30.0, 64.0, [192, 224, 255, 255]));

    voxels
}


fn generate_goblet_aux(steps: i32, rad: f32, height: f32, color: [u8; 4]) -> Voxelstack
{
    let mut voxels = Voxelstack::new();

    for i in 0 .. steps {

        let fi = i as f32 * 1.2 / steps as f32;
        let r = fi.sin() * rad;

        let y = 96.0 - (i as f32 * height) / steps as f32;

        for n in 0 .. steps * 2 {
            let start_angle = fi * - 1.0;
            let angle = PI * n as f32 / steps as f32 + start_angle;

            let x = angle.cos() * r;
            let z = angle.sin() * r;

            voxels.add(Voxel::new(
                x, 
                y, 
                z,
                8.0, 
                color
            ));
        }
    }

    voxels
}


fn generate_scorpion() -> Voxelstack
{
    generate_scorpion_aux([224, 160, 128, 255])
}


fn generate_scorpion_aux(color: [u8; 4]) -> Voxelstack
{
    let mut voxels = Voxelstack::new();

    // head/eyes
    voxels.add(Voxel::new(-36.0, 94.0, 4.0, 9.0, [128, 160, 192, 255]));
    voxels.add(Voxel::new(-36.0, 94.0, -4.0, 9.0, [128, 160, 192, 255]));

    // body
    for n in -4 .. -0 {

        let x = n as f32 * 8.0;
        let y = 96.0;
        let z = 0.0;

        voxels.add(Voxel::new(x, y, z, 16.0, color));

        // feet
        voxels.add(Voxel::new(x + 10.0, y + 6.0, 8.0, 8.0, [255, 224, 160, 255]));
        voxels.add(Voxel::new(x + 10.0, y + 6.0, -8.0, 8.0, [255, 224, 160, 255]));
    }

    // arc
    let mut ac = color;
    for n in -6 .. 10 {
        let size = 16.0 - (n + 6) as f32 * 8.0 / 12.0; 
        let angle = PI * n as f32 / 12.0;

        let x = angle.cos() * 30.0;
        let y = 96.0 - 30.0 - angle.sin() * 30.0;
        let z = 0.0;

        ac[0] += 1;
        ac[1] += 4;

        voxels.add(Voxel::new(x, y, z, size, ac));
    }

    // stinger
    for n in 0 .. 5 {

        let x = -24.0 + n as f32 * -3.0;
        let y = 45.0 + n as f32 * 2.0;
        let z = 0.0;

        voxels.add(Voxel::new(x, y, z, 5.0, [255, 255, 160, 255]));
    }


    voxels
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

    for _rotation in 0 .. tendrils {

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

        voxels.add(Voxel::new(xp, yp, zp, size, color));
    }

    (xp, yp, zp, color)
}
