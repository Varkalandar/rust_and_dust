use std::f32::consts::PI;

use glutin::surface::ResizeableSurface;
use glutin::surface::SurfaceTypeTrait;
use glium::Display;

use crate::Texture2d;
use crate::Framebuffer;
use crate::Voxel;
use crate::Voxelstack;


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
                let p = PI * 2.0 * n as f32 / 24.0;
    
                let x = p.cos() * ir;
                let z = p.sin() * ir;
    
                voxels.add(Voxel {x: x + 400.0, y: 200.0 + fi * rad, z: z + rad});
            }
        }


        for voxel in voxels.voxels {
            let xp = voxel.x as i32;
            let yp = (voxel.y + voxel.z * 0.5) as i32;
            fb.fill_box(xp, yp, 2, 2, [255, 255, 0, 255]);
        }

        VoxelDisplayTest {
            result: fb.to_texture(display),
        }
    }
}