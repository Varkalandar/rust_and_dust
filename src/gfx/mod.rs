use std::collections::HashSet;

mod gl_support;
pub use gl_support::*;

struct Framebuffer 
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
            buffer: Vec::with_capacity((width * height * 4) as usize),   // rgba
        }
    }


    pub fn fillbox(&mut self, x: i32, y: i32, w: i32, h: i32, color: [u8; 4])
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


    pub fn fillcircle(&mut self, xc: i32, yc: i32, radius: i32, color: [u8; 4])
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
}