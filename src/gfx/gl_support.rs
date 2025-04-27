use std::io::BufReader;
use std::fs::File;

use glutin::surface::SurfaceTypeTrait;
use glutin::surface::ResizeableSurface;
use glutin::surface::WindowSurface;

use glium::Display;
use glium::Texture2d;
use glium::Program;
use glium::Surface;
use glium::Frame;
use glium::BlendingFunction;
use glium::LinearBlendingFactor;
use glium::Blend;
use glium::Rect;
use glium::implement_vertex;
use glium::uniform;
use glium::VertexBuffer;

use geo::Coord;
use geo::Polygon;
use geo::CoordsIter;

use crate::ui::*;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);


#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BlendMode {
    Blend,
    Add,
}

pub struct RectF32 {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RectF32 {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        RectF32 {x, y, width, height }
    }
}

pub fn load_texture<T: SurfaceTypeTrait + ResizeableSurface>(display: &Display<T>, filename: &str) -> glium::Texture2d {

    let file_try = File::open(filename);

    if !file_try.is_ok() {
        panic!("Failed to open texture {}", filename);
    }
    let file = file_try.unwrap();

    let reader = BufReader::new(file);

    let image = image::load(reader, image::ImageFormat::Png).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions);
    let texture = glium::Texture2d::new(display, image).unwrap();
    
    texture
}

pub fn texture_from_data<T: SurfaceTypeTrait + ResizeableSurface>(display: &Display<T>, data: Vec<u8>, width: u32, height: u32) -> glium::Texture2d {

    let image = glium::texture::RawImage2d::from_raw_rgba(data, (width, height));
    let texture = glium::Texture2d::new(display, image).unwrap();
    
    texture
}

pub fn build_program(display: &Display<WindowSurface>) -> glium::Program {
    let vertex_shader_src = r#"
    #version 140
    
    in vec2 position;
    in vec2 tex_coords;
    out vec2 v_tex_coords;
    
    uniform mat4 matrix;
    
    void main() {
        v_tex_coords = tex_coords;
        gl_Position = matrix * vec4(position, 0.0, 1.0);
    }
    "#;
    
    let fragment_shader_src = r#"
    #version 140
    
    in vec2 v_tex_coords;
    out vec4 color;

    uniform vec4 col;
    uniform sampler2D tex;
    
    void main() {
        color = texture(tex, v_tex_coords) * col;
    }
    "#;
    
    let program = glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap();

    program
}

pub fn build_dynamic_quad_buffer<T: SurfaceTypeTrait + ResizeableSurface>(display: &Display<T>) 
    -> VertexBuffer<Vertex> 
{
    let xp = 0.0;
    let yp = 0.0;
    let fw = 1.0;
    let fh = 1.0;

    let shape = vec![
        Vertex { position: [xp + 0.0, yp + 0.0], tex_coords: [0.0, 0.0] },
        Vertex { position: [xp +  fw, yp + 0.0], tex_coords: [1.0, 0.0] },
        Vertex { position: [xp +  fw, yp +  fh], tex_coords: [1.0, 1.0] },

        Vertex { position: [xp +  fw,  yp + fh], tex_coords: [1.0, 1.0] },
        Vertex { position: [xp + 0.0,  yp + fh], tex_coords: [0.0, 1.0] },
        Vertex { position: [xp + 0.0, yp + 0.0], tex_coords: [0.0, 0.0] },
    ];

    let vertex_buffer = glium::VertexBuffer::dynamic(display, &shape).unwrap();

    return vertex_buffer;
}


pub fn draw_texture<T: SurfaceTypeTrait + ResizeableSurface>(display: &Display<T>,
                                                             target: &mut Frame,   
                                                             program: &Program,  
                                                             blend: BlendMode,
                                                             texture: &Texture2d,
                                                             xp: f32,
                                                             yp: f32, 
                                                             sx: f32, 
                                                             sy: f32,
                                                             color: &[f32; 4]) {

    let fw = texture.width() as f32 * sx;
    let fh = texture.height() as f32 * sy;

    let shape = vec![
        Vertex { position: [xp + 0.0, yp + 0.0], tex_coords: [0.0, 0.0] },
        Vertex { position: [xp +  fw, yp + 0.0], tex_coords: [1.0, 0.0] },
        Vertex { position: [xp +  fw, yp +  fh], tex_coords: [1.0, 1.0] },

        Vertex { position: [xp +  fw,  yp + fh], tex_coords: [1.0, 1.0] },
        Vertex { position: [xp + 0.0,  yp + fh], tex_coords: [0.0, 1.0] },
        Vertex { position: [xp + 0.0, yp + 0.0], tex_coords: [0.0, 0.0] },
    ];

    draw_shape(display, target, program, blend, &shape, texture, color, None);
}


pub fn draw_texture_clip_wb(target: &mut Frame,   
                            program: &Program,
                            buffer: &VertexBuffer<Vertex>,
                            blend: BlendMode,
                            display_width: u32,
                            display_height: u32,
                            texture: &Texture2d,
                            xp: f32,
                            yp: f32, 
                            sx: f32, 
                            sy: f32,
                            color: &[f32; 4],
                            scissors: &Option<UiArea>) {

    let fw = texture.width() as f32 * sx;
    let fh = texture.height() as f32 * sy;

    let shape = vec![
        Vertex { position: [xp + 0.0, yp + 0.0], tex_coords: [0.0, 0.0] },
        Vertex { position: [xp +  fw, yp + 0.0], tex_coords: [1.0, 0.0] },
        Vertex { position: [xp +  fw, yp +  fh], tex_coords: [1.0, 1.0] },

        Vertex { position: [xp +  fw,  yp + fh], tex_coords: [1.0, 1.0] },
        Vertex { position: [xp + 0.0,  yp + fh], tex_coords: [0.0, 1.0] },
        Vertex { position: [xp + 0.0, yp + 0.0], tex_coords: [0.0, 0.0] },
    ];

    let clip = 
        match scissors {
            Some(scissors) => Some(Rect {
                left: scissors.x as u32,
                bottom: scissors.y as u32,
                width: scissors.w as u32,
                height: scissors.h as u32,
            }),
            None => None,
        };

    draw_shape_wb(target, program, buffer, blend, 
                  display_width, display_height, &shape, texture, color, clip);
}

/*
pub fn draw_tex_area<T: SurfaceTypeTrait + ResizeableSurface>(display: &Display<T>,
                                                             target: &mut Frame,   
                                                             program: &Program,  
                                                             blend: BlendMode,
                                                             texture: &Texture2d,
                                                             src_rect: RectF32, 
                                                             dst_rect: RectF32,
                                                             color: &[f32; 4]) {

    let tw = texture.width() as f32;
    let th = texture.height() as f32;

    let xp = dst_rect.x;
    let yp = dst_rect.y;
    let fw = dst_rect.width;
    let fh = dst_rect.height;

    let tcx = src_rect.x / tw;
    let tcy = src_rect.y / th;
    let tcw = src_rect.width / tw;
    let tch = src_rect.height / th;

    // println!("tex coords = {}, {}, {}, {}", tcx, tcy, tcw, tch);
    // println!("vertex coords = {}, {}, {}, {}", xp, yp, fw, fh);

    let shape = vec![
        Vertex { position: [xp + 0.0, yp + 0.0], tex_coords: [tcx      , tcy] },
        Vertex { position: [xp +  fw, yp + 0.0], tex_coords: [tcx + tcw, tcy] },
        Vertex { position: [xp +  fw, yp +  fh], tex_coords: [tcx + tcw, tcy + tch] },

        Vertex { position: [xp +  fw,  yp + fh], tex_coords: [tcx + tcw, tcy + tch] },
        Vertex { position: [xp + 0.0,  yp + fh], tex_coords: [tcx      , tcy + tch] },
        Vertex { position: [xp + 0.0, yp + 0.0], tex_coords: [tcx      , tcy] },
    ];

    draw_shape(display, target, program, blend, &shape, texture, color, None);
}
*/

pub fn draw_polygon<T: SurfaceTypeTrait + ResizeableSurface>(display: &Display<T>,
    target: &mut Frame,   
    program: &Program,  
    blend: BlendMode,
    texture: &Texture2d,
    polygon: &Polygon<f32>,
    color: &[f32; 4]) {

    let mut iter = polygon.coords_iter();

    let p1: Coord<f32> = iter.next().unwrap();
    let p2: Coord<f32> = iter.next().unwrap();
    let p3: Coord<f32> = iter.next().unwrap();
    let p4: Coord<f32> = iter.next().unwrap();

    let shape = vec![
        Vertex { position: [p1.x, p1.y], tex_coords: [0.0, 0.0] },
        Vertex { position: [p2.x, p2.y], tex_coords: [1.0, 0.0] },
        Vertex { position: [p3.x, p3.y], tex_coords: [1.0, 1.0] },

        // second triangle

        Vertex { position: [p3.x, p3.y], tex_coords: [1.0, 1.0] },
        Vertex { position: [p4.x, p4.y], tex_coords: [0.0, 1.0] },
        Vertex { position: [p1.x, p1.y], tex_coords: [0.0, 0.0] },
    ];

    draw_shape(display, target, program, blend, &shape, texture, color, None);
}


pub fn draw_shape<T: SurfaceTypeTrait + ResizeableSurface>(
    display: &Display<T>,
    target: &mut Frame,   
    program: &Program,  
    blend: BlendMode,
    shape: &Vec<Vertex>,
    texture: &Texture2d,
    color: &[f32; 4],
    scissor: Option<Rect>) {

    let vertex_buffer = glium::VertexBuffer::new(display, shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let (d_width, d_height) = display.get_framebuffer_dimensions();
    let fdw = d_width as f32;
    let fdh = d_height as f32;

    let xf: f32 = 2.0 / fdw; 
    let yf: f32 = 2.0 / fdh; 

    let uniforms = uniform! {
        matrix: [
            [  xf,  0.0,  0.0,  0.0],
            [ 0.0,  -yf,  0.0,  0.0],
            [ 0.0,  0.0,  1.0,  0.0],
            [-1.0,  1.0,  0.0,  1.0],
        ],                        
        tex: texture,
        col: *color,
    };

    let gl_blend = if blend == BlendMode::Blend {
        glium::Blend::alpha_blending()
    }
    else {
        Blend {
            color: BlendingFunction::Addition {
                source: LinearBlendingFactor::SourceAlpha,
                destination: LinearBlendingFactor::One,
            },
                alpha: BlendingFunction::Addition {
                source: LinearBlendingFactor::One,
                destination: LinearBlendingFactor::One
            },
            constant_value: (0.0, 0.0, 0.0, 0.0)
        }
    };

    let params = glium::DrawParameters {
        blend: gl_blend,
        scissor,
        .. Default::default()
    };

    target.draw(&vertex_buffer, &indices, program, &uniforms, &params).unwrap();
}


pub fn draw_texture_wb(
    target: &mut Frame,   
    program: &Program,
    buffer: &VertexBuffer<Vertex>,
    blend: BlendMode,
    display_width: u32,
    display_height: u32,
    texture: &Texture2d,
    xp: f32,
    yp: f32, 
    sx: f32, 
    sy: f32,
    color: &[f32; 4]) {

    let fw = texture.width() as f32 * sx;
    let fh = texture.height() as f32 * sy;

    let shape = vec![
        Vertex { position: [xp + 0.0, yp + 0.0], tex_coords: [0.0, 0.0] },
        Vertex { position: [xp +  fw, yp + 0.0], tex_coords: [1.0, 0.0] },
        Vertex { position: [xp +  fw, yp +  fh], tex_coords: [1.0, 1.0] },

        Vertex { position: [xp +  fw,  yp + fh], tex_coords: [1.0, 1.0] },
        Vertex { position: [xp + 0.0,  yp + fh], tex_coords: [0.0, 1.0] },
        Vertex { position: [xp + 0.0, yp + 0.0], tex_coords: [0.0, 0.0] },
    ];

    draw_shape_wb(target, program, buffer, blend, 
                  display_width, display_height,
                  &shape, texture, color, None);
}


pub fn draw_tex_area_wb(
    target: &mut Frame,   
    program: &Program,  
    buffer: &VertexBuffer<Vertex>,
    blend: BlendMode,
    display_width: u32,
    display_height: u32,
    texture: &Texture2d,
    src_rect: RectF32, 
    dst_rect: RectF32,
    color: &[f32; 4]) {

    let tw = texture.width() as f32;
    let th = texture.height() as f32;

    let xp = dst_rect.x;
    let yp = dst_rect.y;
    let fw = dst_rect.width;
    let fh = dst_rect.height;

    let tcx = src_rect.x / tw;
    let tcy = src_rect.y / th;
    let tcw = src_rect.width / tw;
    let tch = src_rect.height / th;

    // println!("tex coords = {}, {}, {}, {}", tcx, tcy, tcw, tch);
    // println!("vertex coords = {}, {}, {}, {}", xp, yp, fw, fh);

    let shape = vec![
    Vertex { position: [xp + 0.0, yp + 0.0], tex_coords: [tcx      , tcy] },
    Vertex { position: [xp +  fw, yp + 0.0], tex_coords: [tcx + tcw, tcy] },
    Vertex { position: [xp +  fw, yp +  fh], tex_coords: [tcx + tcw, tcy + tch] },

    Vertex { position: [xp +  fw,  yp + fh], tex_coords: [tcx + tcw, tcy + tch] },
    Vertex { position: [xp + 0.0,  yp + fh], tex_coords: [tcx      , tcy + tch] },
    Vertex { position: [xp + 0.0, yp + 0.0], tex_coords: [tcx      , tcy] },
    ];

    draw_shape_wb(target, program, buffer, blend, 
                  display_width, display_height,
                  &shape, texture, color, None);
}


pub fn draw_shape_wb(
    target: &mut Frame,   
    program: &Program,  
    buffer: &VertexBuffer<Vertex>,
    blend: BlendMode,
    display_width: u32,
    display_height: u32,
    shape: &Vec<Vertex>,
    texture: &Texture2d,
    color: &[f32; 4],
    scissor: Option<Rect>) {

    
    buffer.write(shape);
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let fdw = display_width as f32;
    let fdh = display_height as f32;

    let xf: f32 = 2.0 / fdw; 
    let yf: f32 = 2.0 / fdh; 

    let uniforms = uniform! {
        matrix: [
            [  xf,  0.0,  0.0,  0.0],
            [ 0.0,  -yf,  0.0,  0.0],
            [ 0.0,  0.0,  1.0,  0.0],
            [-1.0,  1.0,  0.0,  1.0],
        ],                        
        tex: texture,
        col: *color,
    };

    let gl_blend = if blend == BlendMode::Blend {
        glium::Blend::alpha_blending()
    }
    else {
        Blend {
            color: BlendingFunction::Addition {
                source: LinearBlendingFactor::SourceAlpha,
                destination: LinearBlendingFactor::One,
            },
                alpha: BlendingFunction::Addition {
                source: LinearBlendingFactor::One,
                destination: LinearBlendingFactor::One
            },
            constant_value: (0.0, 0.0, 0.0, 0.0)
        }
    };

    let params = glium::DrawParameters {
        blend: gl_blend,
        scissor,
        /*
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        */

        .. Default::default()
    };

    target.draw(buffer, &indices, program, &uniforms, &params).unwrap();
}
