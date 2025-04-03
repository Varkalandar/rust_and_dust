use vecmath::Vector2;

mod tileset;
mod font;

use std::rc::Rc;
use std::cmp::{min, max};

use glutin::surface::WindowSurface;
use glium::Display;
use glium::winit::window::Window;
use glium::winit::keyboard::Key;
use glium::winit::keyboard::NamedKey;
use glium::Program;
use glium::Frame;
use glium::Texture2d;
use glium::VertexBuffer;

pub use tileset::*;
pub use font::UiFont;
use crate::BlendMode;
use crate::gl_support::draw_texture;
use crate::gl_support::texture_from_data;
use crate::gl_support::draw_texture_wb;
use crate::gl_support::draw_tex_area_wb;
use crate::gl_support::draw_texture_clip_wb;
use crate::gl_support::build_dynamic_quad_buffer;
use crate::gl_support::Vertex;
use crate::gl_support::RectF32;


#[derive(PartialEq, Clone, Debug)]
pub enum MouseButton {
    Left,
    Right,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Button {
    Keyboard(glium::winit::keyboard::Key),
    Mouse(MouseButton),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ButtonState {
    Press,
    Release, 
}

#[derive(Debug, PartialEq, Clone)]
pub struct ButtonArgs {
    pub state: ButtonState,
    pub button: Button,
    pub scancode: Option<i32>,
}


#[derive(Debug, Clone)]
pub struct ButtonEvent {
    pub args: ButtonArgs,
    pub mx: f32,
    pub my: f32,
}

impl ButtonEvent {
    fn translate(&self, x: i32, y: i32) -> ButtonEvent {
        ButtonEvent {
            args: self.args.clone(),
            mx: self.mx + x as f32,
            my: self.my + y as f32,
        }
    }
}


pub struct MouseState {
    pub position: Vector2<f32>,
    drag_start: Vector2<f32>,
    pub left_pressed: bool,    
}


impl MouseState {
    pub fn record_drag_start(&mut self) -> Vector2<f32> {
        self.drag_start = self.position;
        self.drag_start
    }
}


pub struct KeyboardState {
    pub shift_pressed: bool,
    pub ctrl_pressed: bool,
}


pub trait UiController {
    type Appdata;

    /**
     * @return true if this controller could handle the event, false to pass the event to other controllers
     */
    fn handle_button_event(&mut self, _ui: &mut UI,_event: &ButtonEvent, _appdata: &mut Self::Appdata) -> bool {
        false
    }


    fn handle_mouse_move_event(&mut self, _ui: &mut UI, _event: &MouseMoveEvent, _appdata: &mut Self::Appdata) -> bool {
        false
    }


    /**
     * @return true if this controller could handle the event, false to pass the event to other controllers
     */
    fn handle_scroll_event(&mut self, _ui: &mut UI, _event: &ScrollEvent, _appdata: &mut Self::Appdata) -> bool {
        false
    }

    fn draw(&mut self, _target: &mut Frame,
            _ui: &mut UI, _appdata: &mut Self::Appdata) {

    }
    
    fn draw_overlay(&mut self, _target: &mut Frame,
                    _ui: &mut UI, _appdata: &mut Self::Appdata) {

    }

    fn update(&mut self, _appdata: &mut Self::Appdata, _dt: f32) -> bool {
        false
    }
}


#[derive(Debug, Clone)]
pub struct UiArea {
    pub x: i32, 
    pub y: i32,
    pub w: i32,
    pub h: i32,
}


impl UiArea {
    pub fn contains(&self, x: i32, y:i32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.w && y < self.y + self.h  
    }
}


pub struct UiComponent {
    pub head: Box<dyn UiHead>,    
}


pub struct UiContext
{
    pub font_10: Rc<UiFont>,
    pub font_14: Rc<UiFont>,
    pub tex_white: Rc<Texture2d>,

    pub vertex_buffer: VertexBuffer<Vertex>,
    
    pub window_size: [u32; 2],
    pub scissors: Option<UiArea>,
    pub mouse_state: MouseState,
    pub keyboard_state: KeyboardState,
}


pub struct UI
{
    pub root: UiComponent,
    pub context: UiContext,

    pub window: Window,
    pub display: Display<WindowSurface>,
    pub program: Program,
}


impl UI {

    pub fn new(window: Window, display: Display<WindowSurface>, program: Program, window_size: [u32; 2]) -> UI {
        
        let pixels = vec![255_u8; 1024];
        let tex_white = texture_from_data(&display, pixels, 16, 16);

        let context = UiContext { 
            window_size,
            scissors: None,

            font_10: Rc::new(UiFont::new(&display, 10)),
            font_14: Rc::new(UiFont::new(&display, 14)),
            tex_white: Rc::new(tex_white),

            vertex_buffer: build_dynamic_quad_buffer(&display),

            mouse_state: MouseState{position: [0.0, 0.0], drag_start: [0.0, 0.0], left_pressed: false,},
            keyboard_state: KeyboardState{shift_pressed: false, ctrl_pressed: false},
        };

        UI { 
            root: UI::make_container_intern(0, 0, window_size[0] as i32, window_size[1] as i32),
            window,
            display,
            program,
            context,
        }
    }

    
    pub fn window_center(&self) -> Vector2<f32> {
        [(self.context.window_size[0] / 2) as f32, (self.context.window_size[1] / 2) as f32]
    }


    pub fn make_container(&self, x: i32, y: i32, w: i32, h: i32) -> UiComponent {
        UI::make_container_intern(x, y, w, h)
    }


    fn make_container_intern(x: i32, y: i32, w: i32, h: i32) -> UiComponent {
        let container = UiContainer {
            area: UiArea {
                x, 
                y,
                w,
                h,                
            }, 
            children: Vec::new(),
        };

        UiComponent {
            head: Box::new(container),
        }        
    }
    

    pub fn make_button(&self, x: i32, y: i32, w: i32, h: i32, label: &str, _id: usize) -> UiComponent {
        let button = UiButton {
            area: UiArea {
                x, 
                y,
                w,
                h,                
            }, 
            font: self.context.font_14.clone(),
            label: label.to_string(),    
        };
        
        UiComponent {
            head: Box::new(button),
        }        
    }


    pub fn make_icon(&self, x: i32, y: i32, w: i32, h: i32, 
                     tile: &Rc<Tile>, label: &str, id: usize) -> UiComponent {
        let icon = UiIcon {
            area: UiArea {
                x, 
                y,
                w,
                h,                
            }, 
            font: self.context.font_10.clone(),
            label: label.to_string(),
            tile: tile.clone(),
            id,
        };
        
        UiComponent {
            head: Box::new(icon),
        }        
    }

    
    pub fn make_scrollpane(&self, x: i32, y: i32, w: i32, h: i32, 
                           child: UiComponent, scroll_step_x: f32, scroll_step_y: f32) -> UiComponent {
        let scrollpane = UiScrollpane {
            area: UiArea {
                x, 
                y,
                w,
                h,                
            }, 
            child,
            offset_x: 0,
            offset_y: 0,
            scroll_step_x,
            scroll_step_y,
        };
        
        UiComponent {
            head: Box::new(scrollpane),
        }        
    }


    pub fn make_color_choice(&self, x: i32, y: i32, w: i32, h: i32, id: usize, color: [f32;4]) -> UiComponent {
        let colorchoice = UiColorchoice::new(&self.display, x, y, w, h, id, color); 

        UiComponent {
            head: Box::new(colorchoice),
        }        
    }


    pub fn draw(&mut self, target: &mut Frame) {
        let context = &mut self.context;
        let head = &self.root.head;
        head.draw(&self.display, target, &self.program, context, 0, 0);
    }


    pub fn draw_hline(&self, target: &mut Frame, x: i32, y: i32, width: i32, color: &[f32; 4]) {
        let context = &self.context;

        draw_tex_area_wb(target, &self.program, &context.vertex_buffer,
            BlendMode::Blend, 
            context.window_size[0], context.window_size[1],
            &context.tex_white, 
            RectF32::new(0.0, 0.0, 1.0, 1.0),
            RectF32::new(x as f32, y as f32, width as f32, 1.0),
            color);
    }


    pub fn draw_box(&self, target: &mut Frame, x: i32, y: i32, width: i32, height: i32, color: &[f32; 4]) {
        let context = &self.context;
        self.draw_hline(target, x, y, width, color);
        self.draw_hline(target, x, y + height - 1, width, color);

        // vertical sides
        self.fill_box(target, x, y + 1, 1, height - 2, color);
        self.fill_box(target, x + width - 1, y + 1, 1, height - 2, color);
    }
    

    pub fn fill_box(&self, target: &mut Frame, x: i32, y: i32, width: i32, height: i32, color: &[f32; 4]) {
        let context = &self.context;

        draw_tex_area_wb(target, &self.program, &context.vertex_buffer,
            BlendMode::Blend, 
            context.window_size[0], context.window_size[1],
            &context.tex_white, 
            RectF32::new(0.0, 0.0, 1.0, 1.0),
            RectF32::new(x as f32, y as f32, width as f32, height as f32),
            color);
    }


    pub fn handle_button_event(&mut self, event: &ButtonEvent) -> Option<&dyn UiHead> {
        if event.args.state == ButtonState::Press {
            if event.args.button == Button::Keyboard(Key::Named(NamedKey::Shift)) {
                println!("Shift pressed");
                self.context.keyboard_state.shift_pressed = true;
            }
            
            if event.args.button == Button::Mouse(MouseButton::Left) {
                self.context.mouse_state.left_pressed = true;
            }
        }

        if event.args.state == ButtonState::Release {
            if event.args.button == Button::Keyboard(Key::Named(NamedKey::Shift)) {
                println!("Shift released");
                self.context.keyboard_state.shift_pressed = false;
            }    

            if event.args.button == Button::Mouse(MouseButton::Left) {
                self.context.mouse_state.left_pressed = false;
            }
        }

        self.root.head.handle_button_event(event)
    }


    pub fn handle_mouse_move_event(&mut self, event: &MouseMoveEvent) -> Option<&dyn UiHead> {
        self.context.mouse_state.position = [event.mx as f32, event.my as f32];
        self.root.head.handle_mouse_move_event(event, &self.context.mouse_state)
    }


    pub fn handle_scroll_event(&mut self, event: &ScrollEvent) -> Option<&dyn UiHead> {
        self.root.head.handle_scroll_event(event)
    }
}

pub struct MouseMoveEvent {
    pub mx: f32,
    pub my: f32,
}


pub struct ScrollEvent {
    pub dx: f32,
    pub dy: f32,
    pub mx: f32,
    pub my: f32,
}


pub trait UiHead {

    fn area(&self) -> &UiArea {
        &UiArea { x: 0, y: 0, w: 0, h: 0}
    }

    fn set_position(&mut self, _x: i32, _y: i32) {
    }

    fn draw(&self, _display: &Display<WindowSurface>, _target: &mut Frame, _program: &Program,
            _context: &mut UiContext, _x: i32, _y: i32) {
    } 

    fn handle_button_event(&mut self, _event: &ButtonEvent) -> Option<&dyn UiHead> {
        println!("This component cannot handle button events.");
        None
    }

    fn handle_mouse_move_event(&mut self, _event: &MouseMoveEvent, _mouse: &MouseState) -> Option<&dyn UiHead> {
        None
    }

    fn handle_scroll_event(&mut self, _event: &ScrollEvent) -> Option<&dyn UiHead> {
        None
    }

    /**
     * Because children have to be mutable. e.g. to change their looks upon
     * mouse cursor pointing to them, the UI must own the components.
     */
    fn add_child(&mut self, _child: UiComponent) {
        println!("This component cannot store children.");
    }

    fn clear(&mut self) {
    }

    fn get_id(&self) -> usize {
        0
    }

    fn get_numeric_result(&self) -> Vec<u32> {
        vec![0]
    }
}


pub struct UiContainer {
    pub area: UiArea,
    pub children: Vec<UiComponent>,
}


impl UiContainer {

    /**
     * Returns the index of the child at the given position or None
     */ 
    fn find_child_at(&mut self, x: i32, y: i32) -> Option<usize> {

        let rel_x = x - self.area.x;
        let rel_y = y - self.area.y;

        // println!("Checking relative to container origin {}, {}", rel_x, rel_y);

        for i in 0 .. self.children.len() {
            let area = &self.children[i].head.area();

            // println!("Area {}, {}, {}, {}", area.x, area.y, area.w, area.h);

            if area.contains(rel_x, rel_y) {
                // println!("Found a child at {}, {}", x, y);

                return Some(i);
            }
        }

        None
    }
}


impl UiHead for UiContainer {
    
    fn area(&self) -> &UiArea {
        &self.area
    }

    fn draw(&self, display: &Display<WindowSurface>, target: &mut Frame, program: &Program,
            context: &mut UiContext, x: i32, y: i32) {
        // draw only children which are inside visible area

        let xp = x + self.area.x;
        let yp = y + self.area.y;
        let scissors = 
            match &context.scissors {
                Some(area) => area.clone(),
                None => UiArea{x: 0, y: 0, w: context.window_size[0] as i32, h: context.window_size[1] as i32},
            };

        // println!("Scissors = {:?}", scissors);

        for i in 0..self.children.len() {
            let child = &self.children[i];    
            let a = child.head.area();

            if xp + a.x + a.w >= scissors.x && yp + a.y + a.h >= scissors.y &&
               xp + a.x <= scissors.x + scissors.w && yp + a.y <= scissors.y + scissors.h {

                child.head.draw(display, target, program, context, xp, yp);
            }
        }
    }


    fn handle_button_event(&mut self, event: &ButtonEvent) -> Option<&dyn UiHead> {

        let option = self.find_child_at(event.mx as i32, event.my as i32);
                
        println!("event received at {}, {} -> child={}", event.mx, event.my, option.is_some());

        match option {
            None => {
            },
            Some(child) => {
                let c = &mut self.children[child];
                return c.head.handle_button_event(event);
            }
        }

        None
    }


    fn handle_mouse_move_event(&mut self, event: &MouseMoveEvent, mouse: &MouseState) -> Option<&dyn UiHead> {
        let option = self.find_child_at(event.mx as i32, event.my as i32);
                
        match option {
            None => {
            },
            Some(child) => {
                // println!("Mouse moved to {}, {}", event.mx, event.my);
                let c = &mut self.children[child];
                return c.head.handle_mouse_move_event(event, mouse);
            }
        }

        None
    }


    fn handle_scroll_event(&mut self, event: &ScrollEvent) -> Option<&dyn UiHead> {

        let option = self.find_child_at(event.mx as i32, event.my as i32);
                
        match option {
            None => {
            },
            Some(child) => {
                let c = &mut self.children[child];
                return c.head.handle_scroll_event(event);
            }
        }

        None
    }


    fn add_child(&mut self, child: UiComponent) {
        self.children.push(child);
    }


    fn clear(&mut self) {
        self.children.clear();
    }
}


pub struct UiButton {
    pub area: UiArea,
    pub font: Rc<UiFont>,
    pub label: String,
}


impl UiHead for UiButton {
    
    fn area(&self) -> &UiArea {
        &self.area
    }

    fn draw(&self, display: &Display<WindowSurface>, target: &mut Frame, program: &Program, 
            context: &mut UiContext, x: i32, y: i32) {

        let area = self.area();
        
        draw_texture_clip_wb(target, program, &context.vertex_buffer,
            BlendMode::Blend,
            context.window_size[0], context.window_size[1],
            &context.tex_white,
            (area.x + x) as f32,
            (area.y + y) as f32, 
            area.w as f32 / 16.0, 
            area.h as f32 / 16.0,
            &[0.1, 0.1, 0.1, 1.0],
            &context.scissors);

        let label_width = self.font.calc_string_width(&self.label) as i32;
        let label_x = x + (area.w - label_width) / 2;
        let label_y = y + (area.h - self.font.lineheight) / 2;

        self.font.draw(display, target, program, label_x, label_y, &self.label, &[1.0, 1.0, 1.0, 1.0]);
    } 
}


pub struct UiIcon
{
    pub area: UiArea,
    pub font: Rc<UiFont>,
    pub label: String,
    pub tile: Rc<Tile>,
    pub id: usize,
}


impl UiHead for UiIcon
{
    fn area(&self) -> &UiArea {
        &self.area
    }

    fn draw(&self, display: &Display<WindowSurface>, target: &mut Frame, program: &Program, 
            context: &mut UiContext, x: i32, y: i32) {
        let area = self.area();
        let xp = (x + area.x) as f32;
        let yp = (y + area.y) as f32;

        draw_texture_clip_wb(target, program, &context.vertex_buffer,
            BlendMode::Blend,
            context.window_size[0], context.window_size[1],
            &context.tex_white,
            xp,
            yp, 
            area.w as f32 / 16.0, 
            area.h as f32 / 16.0,
            &[0.1, 0.1, 0.1, 1.0],
            &context.scissors);    

        let tw = self.tile.tex.width() as f32 * 0.25;
        let th = self.tile.tex.height() as f32 * 0.25;

        let y_base = yp + area.h as f32 - 26.0; // space for a label below the icon image

        let image_x = xp + (area.w as f32 - tw) / 2.0;
        let image_y = y_base - th;

        draw_texture_clip_wb(target, program, &context.vertex_buffer,
            BlendMode::Blend,
            context.window_size[0], context.window_size[1],
            &self.tile.tex,
            image_x,
            image_y, 
            0.25, 
            0.25,
            &[1.0, 1.0, 1.0, 1.0], 
            &context.scissors);    

        let label_width = self.font.calc_string_width(&self.label) as i32;
        let label_x = xp as i32 + (area.w - label_width) / 2;
        let label_y = yp as i32 + area.h - self.font.lineheight;

        self.font.draw(display, target, program, label_x, label_y, &self.label, &[0.4, 0.6, 0.7, 1.0]);
    } 


    fn handle_button_event(&mut self, event: &ButtonEvent) -> Option<&dyn UiHead> {

        // which buttons should trigger this icon?
        if event.args.button == Button::Mouse(MouseButton::Left) {
            return Some(self);
        }

        None
    }


    fn get_id(&self) -> usize {
        self.id
    }
}


pub struct UiScrollpane
{
    pub area: UiArea,
    child: UiComponent,
    offset_x: i32,
    offset_y: i32,
    scroll_step_x: f32,
    scroll_step_y: f32
}


impl UiHead for UiScrollpane
{
    fn area(&self) -> &UiArea {
        &self.area
    }


    fn draw(&self, display: &Display<WindowSurface>, target: &mut Frame, program: &Program,
            context: &mut UiContext, x: i32, y: i32) {
        let area = self.area();
        let xp = x + area.x;
        let yp = y + area.y;

        draw_texture_wb(target, program, &context.vertex_buffer,
            BlendMode::Blend,
            context.window_size[0], context.window_size[1],
            &context.tex_white,
            xp as f32,
            yp as f32, 
            area.w as f32 / 16.0, 
            area.h as f32 / 16.0,
            &[0.3, 0.2, 0.1, 0.5]);

        context.scissors = Some(UiArea {x: xp, y: yp, w: area.w, h: area.h});

        self.child.head.draw(display, target, program, 
                             context, xp + self.offset_x, yp + self.offset_y);

        context.scissors = None;
    }


    fn handle_scroll_event(&mut self, event: &ScrollEvent) -> Option<&dyn UiHead> {
        self.offset_x += (event.dx * self.scroll_step_x) as i32;
        self.offset_y += (event.dy * self.scroll_step_y) as i32;

        println!("Scrollpane, new scroll offset is {}, {}", self.offset_x, self.offset_y);

        Some(self)
    }

    fn handle_button_event(&mut self, event: &ButtonEvent) -> Option<&dyn UiHead> {

        if event.args.state == ButtonState::Press {
            // paging keys
            if event.args.button == Button::Keyboard(Key::Named(NamedKey::PageDown)) {
                self.offset_y -= self.area.h;
                return Some(self);
            }

            if event.args.button == Button::Keyboard(Key::Named(NamedKey::PageUp)) {
                self.offset_y += self.area.h;
                return Some(self);
            }
        }

        self.child.head.handle_button_event(&event.translate(-self.area.x-self.offset_x, -self.area.y-self.offset_y))
    }
}


pub struct UiColorchoice {
    pub area: UiArea,
    bandwidth: i32,
    pub id: usize,
    tex: Texture2d,
    light: Texture2d,
    trans: Texture2d,
    r: u32,
    g: u32,
    b: u32,
    a: u32,
    lightness: u32,
}


impl UiColorchoice {
    pub fn new(display: &Display<WindowSurface>,
               x: i32, y: i32, w: i32, h: i32, id: usize, color: [f32;4]) -> UiColorchoice {

        println!("make UiColorchoice at {} {} {} {}", x, y, w, h);

        let tw = 28;

        UiColorchoice {
            area: UiArea {
                x, 
                y,
                w,
                h,                
            }, 
            bandwidth: tw,
            id,
            r: (color[0] * 255.0) as u32,
            g: (color[1] * 255.0) as u32,
            b: (color[2] * 255.0) as u32,
            a: (color[3] * 255.0) as u32,
            lightness: 255,
            tex: UiColorchoice::make_color_tex(display, (w - tw) as u32, (h - tw) as u32),
            light: UiColorchoice::make_light_tex(display, (w - tw) as u32, (tw-4) as u32),
            trans: UiColorchoice::make_trans_tex(display, (tw - 4) as u32, (h - tw) as u32),
        }
    }


    fn make_color_tex(display: &Display<WindowSurface>, w: u32, h: u32) -> Texture2d {
        
        let mut img = Vec::with_capacity((w * h) as usize);

        // color field
        for j in 0..h {
            for i in 0..w {
                // normalize input
                let y = 128;
                let u = (i * 255) / h;
                let v = (j * 255) / w;

                let (r, g, b) = Self::yuv_to_rgb(y, u as i32, v as i32);

                img.push(r);
                img.push(g);
                img.push(b);
                img.push(255);
            }
        }

        let tex = texture_from_data(display, img, w, h);        

        tex
    }

    
    fn yuv_to_rgb(y: i32, u: i32, v: i32) -> (u8, u8, u8) {
        // R = (y + 1.4075 * (v - 128));
        // G = (y - 0.3455 * (u - 128) - (0.7169 * (v - 128)));
        // B = (y + 1.7790 * (u - 128));
      
        let r = y + (v - 128);
        let g = y - (u - 128)/2 - (v - 128)/2;
        let b = y + (u - 128);
      
        // println!("RGB {} {} {}", r, g, b);

        (max(min(r, 255), 0) as u8, max(min(g, 255), 0) as u8, max(min(b, 255), 0) as u8)
    }


    fn make_light_tex(display: &Display<WindowSurface>, w: u32, h: u32) -> Texture2d {
        
        let mut img = Vec::with_capacity((w * h) as usize);

        // color field
        for _j in 0..h {
            for i in 0..w {
                let y = (i * 255 / w) as u8;
                img.push(y);
                img.push(y);
                img.push(y);
                img.push(255);
            }
        }

        let tex = texture_from_data(display, img, w, h);        

        tex
    }


    fn make_trans_tex(display: &Display<WindowSurface>, w: u32, h: u32) -> Texture2d {
        
        let mut img = Vec::with_capacity((w * h) as usize);

        // color field
        for j in 0..h {
            for _i in 0..w {
                let y = (j * 255 / h) as u8;
                img.push(255);
                img.push(255);
                img.push(255);
                img.push(255 - y);
            }
        }

        let tex = texture_from_data(display, img, w, h);        

        tex
    }
}



impl UiHead for UiColorchoice
{
    fn area(&self) -> &UiArea {
        &self.area
    }

    fn set_position(&mut self, x: i32, y:i32) {
        self.area.x = x;
        self.area.y = y;
    }

    fn draw(&self, display: &Display<WindowSurface>, target: &mut Frame, program: &Program, 
            context: &mut UiContext, x: i32, y: i32) {
        let area = &self.area;
        let xp = x + area.x;
        let yp = y + area.y;

        let bw = self.bandwidth;

        // println!("Drawing at {} {} {} {}", xp, yp, area.w, area.h);


        // white "reset" area
        draw_texture_clip_wb(target, program, &context.vertex_buffer,
            BlendMode::Blend,
            context.window_size[0], context.window_size[1],
            &context.tex_white,
            (xp + area.w - bw + 4) as f32,
            yp as f32, 
            (bw - 4) as f32 / 16.0, 
            (bw - 4) as f32 / 16.0, 
            &[1.0, 1.0, 1.0, 1.0], 
            &context.scissors);    

        // lightness
        draw_texture_clip_wb(target, program, &context.vertex_buffer,
            BlendMode::Blend,
            context.window_size[0], context.window_size[1],
            &self.light,
            xp as f32,
            yp as f32, 
            1.0, 
            1.0, 
            &[1.0, 1.0, 1.0, 1.0], 
            &context.scissors);
            
        // transparency
        draw_texture_clip_wb(target, program, &context.vertex_buffer,
            BlendMode::Blend,
            context.window_size[0], context.window_size[1],
            &self.trans,
            (xp + area.w - bw + 4) as f32,
            (yp + bw) as f32, 
            1.0, 
            1.0, 
            &[1.0, 1.0, 1.0, 1.0], 
            &context.scissors);

        // color
        draw_texture_clip_wb(target, program, &context.vertex_buffer,
            BlendMode::Blend,
            context.window_size[0], context.window_size[1],
            &self.tex,
            xp as f32,
            (yp + bw) as f32, 
            1.0, 
            1.0, 
            &[1.0, 1.0, 1.0, 1.0], 
            &context.scissors);

        /*
        gl.draw(viewport, |c, gl| {
            
            // white "reset" area
            let rect = Rectangle::new([1.0, 1.0, 1.0, 1.0]); 
            rect.draw([(xp + area.w - bw + 4) as f64, yp as f64, (bw - 4) as f64, (bw - 4) as f64], draw_state, c.transform, gl);
            

            let image_l = 
                Image::new()
                    .rect([xp as f64, yp as f64, (area.w - bw) as f64, (bw - 4) as f64])
                    .color([1.0, 1.0, 1.0, 1.0]);
            image_l.draw(&self.light, draw_state, c.transform, gl);


            let image_t = 
                Image::new()
                    .rect([(xp + area.w - bw + 4) as f64, (yp + bw) as f64, (bw - 4) as f64, (area.h - bw) as f64])
                    .color([1.0, 1.0, 1.0, 1.0]);
            image_t.draw(&self.trans, draw_state, c.transform, gl);


            let image   = 
                Image::new()
                    .rect([xp as f64, (yp + bw) as f64, (area.w - bw) as f64, (area.h - bw) as f64])
                    .color([1.0, 1.0, 1.0, 1.0]);
            image.draw(&self.tex, draw_state, c.transform, gl);
        });
        */
    } 

    fn handle_button_event(&mut self, event: &ButtonEvent) -> Option<&dyn UiHead> {

        let mx = event.mx as i32;
        let my = event.my as i32;

        if my < self.area.y + self.bandwidth {
            // light choice or reset area
            if mx < self.area.x + self.area.w - self.bandwidth {
                // light
                let lightness = ((mx - self.area.x) * 255 / (self.area.w-self.bandwidth)) as u32;

                self.r = min(255, self.r * lightness / self.lightness);
                self.g = min(255, self.g * lightness / self.lightness);
                self.b = min(255, self.b * lightness / self.lightness);        

                self.lightness = lightness;

                println!("New lightness {}", self.lightness);
            }
            else {
                // reset
                self.r = 255;
                self.g = 255;
                self.b = 255;
                self.a = 255;
                self.lightness = 255;
                println!("Reset to white");
            }
        }
        else {
            // color choice or transparency
            if mx < self.area.x + self.area.w - self.bandwidth {
                // color
                let i = mx - self.area.x;
                let j = my - self.area.y;
        
                let y = 64;
                let u = (i * 255) / self.area.h;
                let v = (j * 255) / self.area.w;
        
                let (ur, ug, ub) = Self::yuv_to_rgb(y, u as i32, v as i32);
        
                self.r = min(255, (ur as u32) * self.lightness * 8 / 255);
                self.g = min(255, (ug as u32) * self.lightness * 8 / 255);
                self.b = min(255, (ub as u32) * self.lightness * 8 / 255);        
            }
            else {
                // transp
                let alpha = 255 - (event.my as i32 -self.area.y-self.bandwidth) * 255 / (self.area.h-self.bandwidth);
                self.a = alpha as u32;
                println!("New alpha {}", self.a);
            }
        }

        Some(self)
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn get_numeric_result(&self) -> Vec<u32> {
        vec![self.r, self.g, self.b, self.a]
    }

}
