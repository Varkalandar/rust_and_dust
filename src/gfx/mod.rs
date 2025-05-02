pub mod gl_support;
pub mod voxel;

use std::collections::HashSet;

use glutin::surface::ResizeableSurface;
use glutin::surface::SurfaceTypeTrait;
use glium::Display;
use glium::Texture2d;
use gl_support::texture_from_data;


pub struct Framebuffer 
{
    pub width: i32,
    pub height: i32,
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


    pub fn blend_pix(&mut self, x: i32, y: i32, color: [u8; 4]) 
    {
        let dpos = ((y * self.width + x) * 4) as usize;
        let r1 = self.buffer[dpos] as i32;
        let g1 = self.buffer[dpos+1] as i32;
        let b1 = self.buffer[dpos+2] as i32;
        let a1 = self.buffer[dpos+3] as i32;

        let r2 = color[0] as i32;
        let g2 = color[1] as i32;
        let b2 = color[2] as i32;
        let a2 = color[3] as i32;

        // we round the result down always, so we must add 255 to each fractional value to get proper rounding
        let r = r2 * a2 + r1 * (255 - a2) + 255;
        let g = g2 * a2 + g1 * (255 - a2) + 255;
        let b = b2 * a2 + b1 * (255 - a2) + 255;
        
        // what is right here?
        // let a = b2 * a2 + b1 * (255 - a2) + 255;
        let a =std::cmp::max(a1, a2) << 8;

        self.buffer[dpos] = (r >> 8) as u8;
        self.buffer[dpos+1] = (g >> 8) as u8;
        self.buffer[dpos+2] = (b >> 8) as u8;
        self.buffer[dpos+3] = (a >> 8) as u8;
    }


    pub fn set_pix(&mut self, x: i32, y: i32, color: [u8; 4]) 
    {
        let dpos = ((y * self.width + x) * 4) as usize;
        self.buffer[dpos] = color[0];
        self.buffer[dpos+1] = color[1];
        self.buffer[dpos+2] = color[2];
        self.buffer[dpos+3] = color[3];
    }


    pub fn vball(&mut self, x: i32, y: i32, size: i32, color: [u8; 4])
    {
        match size {
            0 => {
                // nothing to draw
            },
            1 => {
                // darkened pixel
                self.set_pix(x, y, shade(color, 128));
            },
            2 => {
                // darkened pixel
                self.set_pix(x, y, shade(color, 192));
            },
            3 => {
                // one pixel
                self.set_pix(x, y, color);
            },
            4 => {
                // half star
                self.set_pix(x, y, color);
                let c2 = shade(color, 192);
                self.set_pix(x+1, y, c2);
                self.set_pix(x, y+1, c2);
            },
            5 => {
                // star shape, center is brightest
                self.set_pix(x, y, color);

                let c1 = shade(color, 224);
                self.set_pix(x-1, y, c1);
                self.set_pix(x, y-1, c1);

                let c2 = shade(color, 192);
                self.set_pix(x+1, y, c2);
                self.set_pix(x, y+1, c2);
            },
            6 => {
                // 3x3 box shape, center is brightest
                self.set_pix(x, y, color);

                let c1 = shade(color, 224);
                self.set_pix(x-1, y, c1);
                self.set_pix(x, y-1, c1);

                let c2 = shade(color, 192);
                self.set_pix(x+1, y, c2);
                self.set_pix(x, y+1, c2);
                self.set_pix(x-1, y-1, c2);

                let c3 = shade(color, 160);
                self.set_pix(x+1, y-1, c3);
                self.set_pix(x-1, y+1, c3);
                self.set_pix(x+1, y+1, c3);
            },
            7 => {
                // a case 6 base, with 4 more star ray dots
                self.vball(x, y, 6, color);

                let c1 = shade(color, 128);
                self.set_pix(x-2, y, c1);
                self.set_pix(x+2, y, c1);
                self.set_pix(x, y-2, c1);
                self.set_pix(x, y+2, c1);
            },

            _ => {
                panic!("Unknown vball size {}.", size);
            }
        }    
    }


    pub fn to_texture<T: SurfaceTypeTrait + ResizeableSurface>(self, display: &Display<T>) -> Texture2d
    {
        texture_from_data(display, self.buffer, self.width as u32, self.height as u32)
    }
}


pub fn shade(color: [u8; 4], shade: i32) -> [u8; 4]
{
    let r = (color[0] as i32 * shade) >> 8;
    let g = (color[1] as i32 * shade) >> 8;
    let b = (color[2] as i32 * shade) >> 8;

    [r as u8, g as u8, b as u8, color[3]]
}
