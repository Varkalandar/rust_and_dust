use std::collections::HashSet;
use std::cmp::Ordering;

use glutin::surface::ResizeableSurface;
use glutin::surface::SurfaceTypeTrait;
use glium::Display;
use glium::Texture2d;

mod gl_support;
pub use gl_support::*;

pub struct Framebuffer 
{
    width: i32,
    height: i32,
    buffer: Vec<u8>,
}


impl Framebuffer
{
    pub fn new(width: i32, height: i32) -> Framebuffer 
    {
        Framebuffer {
            width,
            height,
            buffer: vec![0_u8; (width * height * 4) as usize],   // rgba
        }
    }


    pub fn fill_box(&mut self, x: i32, y: i32, w: i32, h: i32, color: [u8; 4])
    {
        for yy in y .. y + h {
            for xx in x .. x + w {
                let dpos = ((yy * self.width + xx) * 4) as usize;
                self.buffer[dpos] = color[0];
                self.buffer[dpos+1] = color[1];
                self.buffer[dpos+2] = color[2];
                self.buffer[dpos+3] = color[3];
            }
        }
    }


    pub fn hline(&mut self, x: i32, y: i32, w: i32, color: [u8; 4])
    {
        for xx in x .. x + w {
            let dpos = ((y * self.width + xx) * 4) as usize;
            self.buffer[dpos] = color[0];
            self.buffer[dpos+1] = color[1];
            self.buffer[dpos+2] = color[2];
            self.buffer[dpos+3] = color[3];
        }
    }


    pub fn fill_circle(&mut self, xc: i32, yc: i32, radius: i32, color: [u8; 4])
    {
        let mut f = 1 - radius;
        let mut ddf_x = 1;
        let mut ddf_y = -2 * radius;
        let mut x = 0;
        let mut y = radius;
    
        let mut line_marks: HashSet<i32> = HashSet::new();
          
        self.hline(xc-radius, yc, radius + radius + 1, color);
          
        while x < y { 
            if f >= 0 {
                y = y - 1;
                ddf_y = ddf_y + 2;
                f = f + ddf_y;
            }
                    
            x = x + 1;
            ddf_x = ddf_x + 2;
            f = f + ddf_x;
  
            if line_marks.get(&y).is_none() {
                self.hline(xc-x, yc+y, x+x, color);
                self.hline(xc-x, yc-y, x+x, color);
                line_marks.insert(y);
            }
      
            if line_marks.get(&x).is_none() {
                self.hline(xc-y, yc+x, y+y, color);
                self.hline(xc-y, yc-x, y+y, color);
                line_marks.insert(x);
            }
        }
    }


    pub fn setpix(&mut self, x: i32, y: i32, color: [u8; 4]) 
    {
        let dpos = ((y * self.width + x) * 4) as usize;
        self.buffer[dpos] = color[0];
        self.buffer[dpos+1] = color[1];
        self.buffer[dpos+2] = color[2];
        self.buffer[dpos+3] = color[3];
    }

    pub fn to_texture<T: SurfaceTypeTrait + ResizeableSurface>(self, display: &Display<T>) -> Texture2d
    {
        texture_from_data(display, self.buffer, self.width as u32, self.height as u32)
    }
}

pub struct Voxel
{
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub struct Voxelstack 
{
    pub voxels: Vec<Voxel>,
}

impl Voxelstack
{
    pub fn new() -> Voxelstack
    {
        Voxelstack {
            voxels: Vec::new(),
        }
    }

    pub fn add(&mut self, voxel: Voxel)
    {
        self.voxels.push(voxel);
    }

    pub fn sort_depth_first(&mut self)
    {
        self.voxels.sort_unstable_by(|a, b| -> Ordering {
            if a.z > b.z {
                Ordering::Greater
            } else if a.z < b.z {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
    }
}