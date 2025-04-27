use std::collections::HashMap;

use glutin::surface::WindowSurface;
use glium::Display;
use glium::Texture2d;
use glium::Program;
use glium::Frame;
use glium::VertexBuffer;

use crate::gfx::Vertex;
use crate::gfx::BlendMode;
use crate::gfx::RectF32;
use crate::gfx::texture_from_data;
use crate::gfx::build_dynamic_quad_buffer;
use crate::gfx::draw_tex_area_wb;

const PITCH: u32 = 1024;


#[allow(dead_code)]
struct UiGlyph {
    pub metrics: freetype::GlyphMetrics,
    tex_x: u32,
    tex_y: u32,
    advance: f32,
    top:  f32, // pixels above the baseline
    left: f32, // left-right shift
    bm_w: f32,
    bm_h: f32,
}


pub struct UiFont {
    face: freetype::Face,
    pub lineheight: i32,
    
    glyphs: HashMap<usize, UiGlyph>,
    texture: Texture2d,

    vertex_buffer: VertexBuffer<Vertex>,
}


impl UiFont {

    pub fn new(display: &Display<WindowSurface>, size: u32) -> UiFont 
    {
        let ft = freetype::Library::init().unwrap();
        let font = "resources/font/FiraSans-Regular.ttf";
        // let font = "resources/font/Humanistic.ttf";
        // let font = "resources/font/HumanisticCursive.ttf";
        let face = ft.new_face(font, 0).unwrap();
        face.set_pixel_sizes(0, size).unwrap();

        let char_height = ((face.ascender() - face.descender()) / 64) as f64; 
        let lineheight = (char_height * 1.2) as i32;

        // println!("Ascend {} descend {}", face.ascender(), face.descender());

        let mut glyphs = HashMap::new();
        let texture = create_glyphs(display, &face, &mut glyphs, lineheight as u32);

        let vertex_buffer = build_dynamic_quad_buffer(display);

        UiFont {
            face,
            lineheight,
            glyphs,
            texture,
            vertex_buffer,
        }        
    }


    pub fn calc_string_width(&self, text: &str) -> f32
    {
        let mut w = 0.0;
        
        for ch in text.chars() {
            let idx = ch as usize;
            let glyph = self.glyphs.get(&idx).unwrap();
            w += glyph.advance;                
        }

        w
    }


    pub fn draw(&self,
                display: &Display<WindowSurface>, target: &mut Frame, program: &Program,
                x: i32, y: i32, text: &str, color: &[f32; 4])
    {
        let (d_width, d_height) = display.get_framebuffer_dimensions();

        let mut xp = x as f32;
        let yp = (y as f32) + (self.face.ascender() / 64) as f32;
        
        for ch in text.chars() {
            
            // println!("char {} usize {}", ch, ch);
            let idx = ch as usize;
            let glyph = self.glyphs.get(&idx).unwrap();

            draw_tex_area_wb(target, program, &self.vertex_buffer,
                BlendMode::Blend,
                d_width, d_height,
                &self.texture,
                RectF32::new(glyph.tex_x as f32, glyph.tex_y as f32, glyph.bm_w, glyph.bm_h),
                RectF32::new(xp + glyph.left, yp - glyph.top, glyph.bm_w, glyph.bm_h),
                color);

            xp += glyph.advance;                
        }
    }

    pub fn draw_multiline(&self,
                          display: &Display<WindowSurface>, target: &mut Frame, program: &Program,
                          x: i32, y: i32, width: i32, text: &str, color: &[f32; 4], 
                          actually_draw: bool) -> i32
    {
        let (d_width, d_height) = display.get_framebuffer_dimensions();

        let mut run_x = 0.0;
        let xp = x as f32;
        let mut yp = (y as f32) + (self.face.ascender() / 64) as f32;
        let mut lines = 1;
        let mut iter = text.chars();
        let mut eof = false;
        let mut word = Vec::with_capacity(16);

        loop {
            // println!("char {} usize {}", ch, ch);

            // scan one word ahead, we don't know yet if this word will fit on the current line
            let mut word_width = 0.0;
            word.clear();
                        
            loop {
                let ch_opt = iter.next();
                
                if ch_opt.is_some() {
                    let ch = ch_opt.unwrap();
                    let idx = ch as usize;
                    let glyph = self.glyphs.get(&idx).unwrap();
        
                    word_width += glyph.advance;
                    word.push(glyph);
    
                    if ch.is_whitespace() {
                        // the word ends here, exit loop
                        break;
                    } 
                }
                else {
                    // end of text
                    eof = true;
                    break;
                }
            }

            // we now have a word of glyphs in the vector.
            // -> see if it will fit on the current line or if we need
            // to start a new line

            if run_x + word_width > width as f32 {
                // we need to start a new line for this word.
                run_x = 0.0;
                yp += self.lineheight as f32;
                lines += 1;
            }

            if actually_draw {
                // display the word
                for glyph in &word {
                    draw_tex_area_wb(target, program, &self.vertex_buffer,
                        BlendMode::Blend,
                        d_width, d_height,
                        &self.texture,
                        RectF32::new(glyph.tex_x as f32, glyph.tex_y as f32, glyph.bm_w, glyph.bm_h),
                        RectF32::new(xp + run_x +  glyph.left, yp - glyph.top, glyph.bm_w, glyph.bm_h),
                        color);

                    run_x += glyph.advance;
                }
            }
            else {
                run_x += word_width;
            }

            if eof {
                break;
            }
        }

        // println!("Text '{}' requires {} lines", text, lines);

        return lines;
    }
}


fn create_glyphs(display: &Display<WindowSurface>, face: &freetype::Face, glyphs: &mut HashMap<usize, UiGlyph>, lineheight: u32) -> Texture2d {
    
    let mut num_glyphs = 0;

    for glyph_nr in 0..0xFFFF {
        let idx_result = face.get_char_index(glyph_nr);
        if idx_result.is_some() { 
            num_glyphs += 1;
        }
    }

    println!("Found {} glyphs in font, lineheight={}, has_kerning={}", num_glyphs, lineheight, face.has_kerning());

    let b_width = PITCH;
    let b_height = (num_glyphs / 32) * lineheight;

    let mut buffer = vec![0_u8; (b_width * b_height * 4) as usize];

    // cursor to write glyphs into the texture buffer
    let mut cursor: (u32, u32) = (0, 0);

    for glyph_nr in 0..0xFFFF {

        let idx_result = face.get_char_index(glyph_nr);
        if idx_result.is_some() {
            let idx = idx_result.unwrap();
            face.load_glyph(idx, freetype::face::LoadFlag::RENDER).unwrap();
    
            let gs = face.glyph();
            let bitmap = gs.bitmap();
            let m = gs.metrics();
            
            // let ascend = face.ascender() as i32 / 64;

            let ug = UiGlyph {
                metrics: m,
                tex_x: cursor.0,
                tex_y: cursor.1,
                advance: m.horiAdvance as f32 / 64.0,
                top: gs.bitmap_top() as f32,
                left: gs.bitmap_left() as f32,
                bm_w: bitmap.width() as f32,
                bm_h: bitmap.rows() as f32,
            };

            // let left = gs.bitmap_left();
            // println!("glyph {} has advance={}, ascend={}, left={}", idx, ug.advance / 64.0, ascend, left);
            
            cursor = convert_bitmap(&mut buffer, &bitmap, cursor, lineheight);

            glyphs.insert(glyph_nr, ug);
        }
    }
    
    texture_from_data(display, buffer, b_width, b_height)
}

    
fn convert_bitmap(buffer: &mut Vec<u8>, bitmap: &freetype::Bitmap,cursor: (u32, u32), lineheight:u32) -> (u32, u32) {
    
    let bb = bitmap.buffer();
    let bw = bitmap.width() as u32;
    let bh = bitmap.rows() as u32;
    let bp = bitmap.pitch() as u32;
    
    let mut xp = cursor.0; 
    let mut yp = cursor.1;
    
    // println!("placing glyph at {}, {}", xp, yp);

    for y in 0..bh {
        for x in 0..bw {
            let idx = (y * bp + x) as usize;
            let alpha = (bb[idx] as f64 / 255.0).powf(0.75) * 255.0;
            buffer_setpix(buffer, xp + x, yp + y, alpha as u8)                
        }
    }

    // debug, print glyph on stdout
    /*
    for y in 0..bh {
        for x in 0..bw {
            let idx = (y * bp + x) as usize;
            if bb[idx] > 127 {
                print!("#");
            } else {
                print!(" ");                
            }
        }
        println!("");                
    }
    */
    
    xp += bw + 1;
    if xp >= PITCH {
         xp = 0;
         yp += lineheight;
    }

    (xp, yp)
}


fn buffer_setpix(buffer: &mut Vec<u8>, x: u32, y: u32, alpha: u8) {
    let dpos = ((y * PITCH + x) * 4) as usize;
    buffer[dpos] = 255;
    buffer[dpos+1] = 255;
    buffer[dpos+2] = 255;
    buffer[dpos+3] = alpha;
}
