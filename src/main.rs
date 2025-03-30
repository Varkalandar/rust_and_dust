extern crate glium;
extern crate glutin;

extern crate freetype;
extern crate image;
extern crate rodio;
extern crate rand;

use glutin::surface::WindowSurface;

use glium::Display;
use glium::Frame;
use glium::Texture2d;
use glium::Program;
use glium::winit::application::ApplicationHandler;
use glium::winit::event_loop::ActiveEventLoop;
use glium::winit::event::WindowEvent;
use glium::winit::window::{Window, WindowId};
use glium::winit::keyboard::Key;
use glium::winit::event::MouseScrollDelta;
use glium::VertexBuffer;
use glium::Surface;

use vecmath::{vec2_add, vec2_len, vec2_scale, vec2_sub, Vector2};
use rand::SeedableRng;

use std::time::SystemTime;
use std::time::Duration;
use std::thread::sleep;

use std::fs::read_to_string;
use std::path::Path;
use std::cmp::Ordering;

mod item;
mod creature;
mod inventory;
mod projectile;
mod dungeon;
mod map;
mod editor;
mod game;
mod ui;
mod sound;
mod particle_driver;
mod animation;
mod mob_group;
mod player_inventory_view;
mod gl_support;

use dungeon::*;
use map::{Map, MAP_GROUND_LAYER, MAP_OBJECT_LAYER, MAP_CLOUD_LAYER, MoveEndAction};
use ui::{UI, UiController, TileSet, Button, ButtonState, ButtonArgs, MouseButton, ButtonEvent, MouseMoveEvent, ScrollEvent};
use editor::MapEditor;
use game::Game;
use item::ItemFactory;
use inventory::{Inventory, Slot};
use sound::SoundPlayer;

pub use gl_support::Vertex;
use gl_support::BlendMode;
use gl_support::load_texture;
use gl_support::build_program;
use gl_support::draw_texture;
use gl_support::draw_texture_wb;
use gl_support::build_dynamic_quad_buffer;
use gl_support::draw_polygon;

const MAP_RESOURCE_PATH: &str = "resources/gfx/map/";
const CREATURE_TILESET: usize = 3;
const PROJECTILE_TILESET: usize = 5;
const ANIMATION_TILESET: usize = 7;

// Game structures

pub struct GameWorld {
    map: Map,
    layer_tileset: [TileSet; 8],

    player_inventory: Inventory,

    speaker: SoundPlayer,

    rng: rand::rngs::StdRng,

    black_texture: Texture2d,
    map_texture: Texture2d,
    map_backdrop: Texture2d,
}


pub struct GameControllers {
    editor: MapEditor,
    game: Game,
    edit: bool,    
}


impl GameControllers {
    fn current(&mut self) -> &mut dyn UiController<Appdata = GameWorld> {
        if self.edit {
            &mut self.editor
        }
        else {
            &mut self.game
        }
    }
}


pub struct App {
    ui: UI,

    world: GameWorld,
    controllers: GameControllers,

    update_time: SystemTime,
    need_focus: bool,
}


impl App {
    
    fn new(window: Window, display: Display<WindowSurface>, program: Program, window_size: [u32; 2]) -> App {

        // let map_image_file = "map_wasteland.png";
        let map_image_file = "map_soft_grass.png";
        let map_backdrop_file = "backdrop_red_blue.png";

        let black_texture = load_texture(&display, &(MAP_RESOURCE_PATH.to_string() + "map_black.png"));
        let map_texture = load_texture(&display, &(MAP_RESOURCE_PATH.to_string() + map_image_file));
        let map_backdrop = load_texture(&display, &(MAP_RESOURCE_PATH.to_string() + map_backdrop_file));

        let ground_tiles = TileSet::load(&display, "resources/gfx/grounds", "map_objects.tica");
        let decoration_tiles = TileSet::load(&display, "resources/gfx/objects", "map_objects.tica");
        let item_tiles = TileSet::load(&display, "resources/gfx/items", "items.tica");
        let cloud_tiles = TileSet::load(&display, "resources/gfx/clouds", "map_objects.tica");
        let creature_tiles = TileSet::load(&display, "resources/gfx/creatures", "creatures.tica");
        let player_tiles = TileSet::load(&display, "resources/gfx/players", "players.tica");
        let projectile_tiles = TileSet::load(&display, "resources/gfx/projectiles", "projectiles.tica");
        let animation_tiles = TileSet::load(&display, "resources/gfx/animations", "animations.tica");

        let layer_tileset = [
            ground_tiles,
            decoration_tiles,
            cloud_tiles,
            creature_tiles,
            player_tiles,
            projectile_tiles,
            item_tiles,
            animation_tiles,
            ];        

        let mut factory = ItemFactory::new();
        let rng = rand::rngs::StdRng::seed_from_u64(12345678901);
        let mut map = Map::new("Demo Map", map_image_file, map_backdrop_file);
        // map.load("start.map");

        // Testing dungeon generation
        let dungeon = generate_dungeon(&mut map, &mut factory);
        map.set_player_position(dungeon.start_position);

        let ui = UI::new(window, display, program, window_size);
        
        let editor = MapEditor::new();

        let inventory_bg = load_texture(&ui.display, "resources/gfx/ui/inventory_bg.png");
        let game = Game::new(inventory_bg, &ui, &layer_tileset[6]);

        let mut inv = Inventory::new();

        // Some inventory contents for testing

        let demo_item = factory.create("wooden_wand");
        inv.put_item(demo_item, Slot::Bag);

        let laser = factory.create("engraved_wand");
        inv.put_item(laser, Slot::RWing);

        let mut coins = factory.create("copper_coin");
        coins.stack_size = 1000;
        inv.put_item(coins, Slot::Bag);

        App {        
            ui,

            world: GameWorld {
                map,
                layer_tileset,
                player_inventory: inv,
                speaker: SoundPlayer::new(),

                rng,

                black_texture,
                map_texture,
                map_backdrop,
            },

            controllers: GameControllers {
                editor,
                game,
                edit: true,
            },

            update_time: SystemTime::now(),
            need_focus: true,
        }
    }


    /**
     * Updates the game world using the time passed since the last update
     * to determine how much things have moved etc.
     *
     * @return true if the display must be updated too, false otherwise.
     */
    fn update(&mut self) -> bool {
        let world = &mut self.world;

        let now = SystemTime::now();
        let difference = now.duration_since(self.update_time);

        if difference.is_ok() {
            let secs = difference.unwrap().as_secs_f32();

            // try to limit to 100 updates per second
            if secs < 0.01 {
                // too early
                return false;
            }

            self.update_time = now;

            // println!("seconds: {}", secs);

            self.controllers.current().update(world, secs);
        }

        true
    }


    fn render(&mut self) {

        let t0 = SystemTime::now();

        let buffer = build_dynamic_quad_buffer(&self.ui.display);
        let world = &self.world;

        let width = self.ui.context.window_size[0] as f32;
        let height = self.ui.context.window_size[1] as f32;

        let player_position = &world.map.get_player_position();
        let player_x = player_position[0] as f32;
        let player_y = player_position[1] as f32;

        let offset_x = width / 2.0 - player_x;
        let offset_y = height / 2.0 - player_y / 2.0;

        // background image, parallax scrolling at 0.5 times map scroll amount
        let back_off_x = - player_x / 2.0;
        let back_off_y = - player_y / 4.0;

        let mut target = self.ui.display.draw();
                
        target.clear_color(0.0, 1.0, 1.0, 1.0);
        target.clear_depth(1.0);

        
        draw_texture(&self.ui.display, &mut target, &self.ui.program, BlendMode::Blend, &self.world.black_texture, 
                     0.0, 0.0, 1000.0, 1000.0, &[0.8, 0.8, 0.8, 1.0]);
        

        /*
        draw_texture(&self.ui.display, &mut target, &self.ui.program, BlendMode::Blend, &self.world.map_backdrop, 
                     back_off_x, back_off_y, 2.0, 2.0, &[0.8, 0.8, 0.8, 1.0]);

        draw_texture(&self.ui.display, &mut target, &self.ui.program, BlendMode::Blend, &self.world.map_texture, 
                     offset_x, offset_y, 2.0, 2.0, &[0.8, 0.8, 0.8, 1.0]);
        */

        let tex_white = &self.ui.context.tex_white;

        
        // draw ground decorations (flat)
        Self::render_layer(&self.ui.display, &mut target, &self.ui.program, &buffer, world, tex_white, MAP_GROUND_LAYER);

        // draw decorations (upright things)
        Self::render_layer(&self.ui.display, &mut target, &self.ui.program, &buffer, world, tex_white, MAP_OBJECT_LAYER);

        // draw clouds
        Self::render_layer(&self.ui.display, &mut target, &self.ui.program, &buffer, world, tex_white, MAP_CLOUD_LAYER);
        
        /*
        draw_polygon(&self.ui.display, &mut target, &self.ui.program,
            BlendMode::Blend,
            tex_white,
            &self.world.map.walkable[0],
            &[1.0, 1.0, 1.0, 1.0]);
        */


        {
            let world = &mut self.world;
            let ui = &mut self.ui;
            self.controllers.current().draw(&mut target, ui, world);    
            self.controllers.current().draw_overlay(&mut target, ui, world);    
        }

        
        let now = SystemTime::now();
        let difference = now.duration_since(t0);

        if difference.is_ok() {
            self.update_time = now;
            let secs = difference.unwrap().as_secs_f64();
            let s = format!("Time: {}s", secs);
            let font = &self.ui.context.font_14;
        
            font.draw(&self.ui.display, &mut target, &self.ui.program, 10, 600, &s, &[1.0, 1.0, 1.0, 1.0]);
        }
        


        target.finish().unwrap();
    }


    fn render_layer(display: &Display<WindowSurface>, target: &mut Frame, program: &Program,
                    buffer: &VertexBuffer<Vertex>,
                    world: &GameWorld, tex_white: &Texture2d, layer_id: usize) {

        let (display_width, display_height) = display.get_framebuffer_dimensions();
        let window_center = [display_width as f32 * 0.5, display_height as f32 * 0.5];

        let pos_frac = &world.map.get_player_position();
        let player_position = &[pos_frac[0].floor(), pos_frac[1].floor()];
        let mut objects = Vec::with_capacity(world.map.layers.len());

        for (_key, mob) in &world.map.layers[layer_id] {

            let xd = mob.position[0] - pos_frac[0];
            let yd = mob.position[1] - pos_frac[1];
            let distance = xd * xd + yd * yd; 
            let max = window_center[0] * window_center[0] + window_center[1] * window_center[1];

            if distance < max * 3.0 {
                objects.push(mob);
            }
        }

        
        objects.sort_unstable_by(|a, b| -> Ordering {
            let ap = a.position[0] + a.position[1] * 10000.0;
            let bp = b.position[0] + b.position[1] * 10000.0;

            if ap > bp {
                Ordering::Greater
            } else if ap < bp {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });        


        for mob in objects {
            let tileset_id = mob.visual.tileset_id;

            // println!("Accessing mob {} with tile {} from tileset {}", mob.uid, mob.visual.current_image_id, tileset_id);

            let set = &world.layer_tileset[tileset_id];                    
            let tile = set.tiles_by_id.get(&mob.visual.current_image_id).unwrap();

            let tpos = calc_tile_position(&mob.position, tile.foot, mob.visual.scale, player_position, &window_center);

            draw_texture_wb(target, program, &buffer,
                mob.visual.blend,
                display_width,
                display_height,
                &tile.tex,
                tpos[0],
                tpos[1], 
                mob.visual.scale as f32, 
                mob.visual.scale as f32,
                &mob.visual.color);

            // highlight selected item
            if world.map.has_selection && 
               layer_id == world.map.selected_layer &&
               mob.uid == world.map.selected_item {
                
                draw_texture_wb(target, program, buffer,
                    BlendMode::Add,
                    display_width,
                    display_height,
                    tex_white,
                    tpos[0],
                    tpos[1], 
                    (tile.size[0] as f32 * mob.visual.scale / 16.0), 
                    (tile.size[1] as f32 * mob.visual.scale / 16.0), 
                    &[0.15, 0.2, 0.1, 1.0]);
            }

            // fake shine for glowing projectiles
            if tileset_id == 5 {

                let glow_tile = &world.layer_tileset[2].tiles_by_id[&21]; // cloud set
                let tpos = calc_tile_position(&mob.position, glow_tile.foot, 0.9, player_position, &window_center);

                draw_texture_wb(target, program, buffer,
                    BlendMode::Add,
                    display_width,
                    display_height,
                    &glow_tile.tex,
                    tpos[0] - 170.0,
                    tpos[1] - 50.0, 
                    0.9, 
                    0.9,
                    &mob.visual.glow);    
            }

            // particle effects
            mob.visual.particles.for_each_particle(|particles, last_particle_mark| {
                
                for i in 0..last_particle_mark {
                    let p = &particles[i];

                    if p.active {
                        // println!("p.tex={} pos {}, {}", p.tex_id, p.xpos, p.ypos);

                        let set = mob.visual.particles.spawn_tile_set;
                        let tile = &world.layer_tileset[set].tiles_by_id.get(&p.tex_id).unwrap();
                        let tpos = calc_tile_position(&mob.position, tile.foot, mob.visual.scale, player_position, &window_center);

                        // world coordinates to screen coordinates
                        let xp = p.xpos as f32;
                        let yp = ((p.ypos - p.zpos) * 0.5)  as f32;

                        let fade = quadratic_fade(p.age / p.lifetime);

                        draw_texture_wb(target, program, buffer,
                            BlendMode::Add,
                            display_width,
                            display_height,
                            &tile.tex,
                            tpos[0] + xp,
                            tpos[1] + yp, 
                            1.0, 
                            1.0,
                            &[p.color[0]*fade, p.color[1]*fade, p.color[2]*fade, 1.0])
                    }
                }
            });
        }    
    }

    
    fn move_player(&mut self, window_center: Vector2<f32>) {
        
        let screen_direction = vec2_sub(self.ui.context.mouse_state.position, window_center);
        
        // world coordinates have y components double as large
        // as screen coordinates
        let direction = [screen_direction[0], screen_direction[1] * 2.0];
        
        let distance = vec2_len(direction);

        let player = self.world.map.layers[MAP_OBJECT_LAYER].get_mut(&self.world.map.player_id).unwrap();
        let attributes = player.creature.as_ref().unwrap();
        let time = distance / attributes.base_speed; // pixel per second

        player.move_time_left = time;
        player.velocity = vec2_scale(direction, 1.0/time);

        let dest = vec2_add(player.position, direction);

        let d = player.visual.orient(direction);
        player.visual.current_image_id = player.visual.base_image_id + d;

        println!("  moving {} pixels over {} seconds, destination is {:?}", distance, time, dest);        
    }


    pub fn handle_button_event(&mut self, event: &ButtonEvent) {

        // editor/game switch must be handled here, the button press is not handed down ATM

        println!("button event = {:?}", event);

        if event.args.state == ButtonState::Release {
            if event.args.button == Button::Keyboard(Key::Character("e".into())) {    
                self.controllers.edit = true;
                println!("Switching to editor mode.");

                let player = self.world.map.layers[MAP_OBJECT_LAYER].get_mut(&self.world.map.player_id).unwrap();
                player.move_end_action = MoveEndAction::None;
            }
            if event.args.button == Button::Keyboard(Key::Character("g".into())) {                        
                self.controllers.edit = false;
                println!("Switching to game mode.");
                
                let player = self.world.map.layers[MAP_OBJECT_LAYER].get_mut(&self.world.map.player_id).unwrap();
                player.move_end_action = MoveEndAction::PickItemsUp;
            }        
        }

        let window_center: Vector2<f32> = self.ui.window_center(); 
        let controller = &mut self.controllers.current();
        let world = &mut self.world;
        let ui = &mut self.ui;

        let consumed = controller.handle_button_event(ui, &event, world);

        if event.args.state == ButtonState::Release && !consumed {
            if event.args.button == Button::Mouse(MouseButton::Left) {
                self.move_player(window_center);            
            }
        }
    }


    fn handle_mouse_move_event(&mut self, event: &MouseMoveEvent) {
        
        let controller = &mut self.controllers.current();
        let world = &mut self.world;
        let ui = &mut self.ui;

        controller.handle_mouse_move_event(ui, &event, world);
    }


    fn handle_scroll_event(&mut self, event: &ScrollEvent) {

        let controller = &mut &mut self.controllers.current();
        let world = &mut self.world;
        let ui = &mut self.ui;

        controller.handle_scroll_event(ui, &event, world);
    }
}


pub fn read_lines(pathname: &str) -> Vec<String> {
    let path = Path::new(pathname);    
    let rs = read_to_string(path).unwrap();
    let mut lines = Vec::new();
    
    for line in rs.lines() {
        lines.push(line.to_string());
    }

    lines
}


pub fn parse_rgba(color_str: &str) -> [f32; 4] {
    let mut color_iter = color_str.split(" ");

    let mut color: [f32; 4] = [0.0; 4];
    for i in 0..4 {
        color[i] = color_iter.next().unwrap().parse::<f32>().unwrap();
    }

    color
}


pub fn screen_to_world_pos(ui: &UI, player_pos: &Vector2<f32>, screen_pos: &Vector2<f32>) -> Vector2<f32>
{
    let rel_mouse_x = screen_pos[0] - (ui.context.window_size[0] / 2) as f32;
    let rel_mouse_y = (screen_pos[1] - (ui.context.window_size[1] / 2) as f32) * 2.0;

    // transform to world coordinates
    // it is relatrive to player position
    let world_pos = [rel_mouse_x + player_pos[0], rel_mouse_y + player_pos[1]];

    world_pos
}


pub fn calc_tile_position(position: &Vector2<f32>, foot: Vector2<f32>, scale: f32, player_position: &Vector2<f32>, window_center: &Vector2<f32>) -> [f32; 2] {
        
    let mut pos_x = position[0] - player_position[0];
    let mut pos_y = (position[1] - player_position[1]) * 0.5;  
    
    pos_x += window_center[0];
    pos_y += window_center[1];
    pos_x += -foot[0] * scale;
    pos_y += -foot[1] * scale;
    
    [pos_x, pos_y]
}


fn quadratic_fade(x: f32) -> f32 {
    (1.0 - (x*x))
}


impl ApplicationHandler for App {

    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        // Why and how to do that?
        // self.window = Some(event_loop.create_window(glium::winit::window::Window::default_attributes()).unwrap());
    }
    
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {

        // we can only request focus when visible. So after application start we must wait till
        // the window becomes visible and only then can request focus. Maybe there is a better method?
        if self.need_focus {
            let visible = self.ui.window.is_visible();

            if visible.is_some() && visible.unwrap() {
                self.ui.window.focus_window();
                self.need_focus = false;
            }
        }

        match event {

            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },

            WindowEvent::RedrawRequested => {
                let redraw_required = self.update();
                if redraw_required {
                    self.render();
                }

                self.ui.window.request_redraw();                
            }

            // Because glium doesn't know about windows we need to resize the display
            // when the window's size has changed.
            glium::winit::event::WindowEvent::Resized(window_size) => {
                self.ui.display.resize(window_size.into());
                self.ui.context.window_size = [window_size.width, window_size.height];
            },

            glium::winit::event::WindowEvent::MouseInput { device_id: _, button, state } => {

                let button_event = ButtonEvent {
                    args: ButtonArgs {
                        state: if state == glium::winit::event::ElementState::Pressed { ButtonState::Press } else { ButtonState::Release },
                        button: if button == glium::winit::event::MouseButton::Left {Button::Mouse(MouseButton::Left)} else {Button::Mouse(MouseButton::Right)},
                        scancode: None,
                    },
                    mx: self.ui.context.mouse_state.position[0],
                    my: self.ui.context.mouse_state.position[1],
                };

                // println!("Button = {:?}, state = {:?}, button_event = {:?}", button, state, button_event);

                self.handle_button_event(&button_event);
            },

            glium::winit::event::WindowEvent::CursorMoved { device_id: _, position } => {
                // println!("mouse position = {:?}", position);

                let event = MouseMoveEvent {
                    mx: position.x as f32,
                    my: position.y as f32,
                };
                self.handle_mouse_move_event(&event);
            },

            glium::winit::event::WindowEvent::MouseWheel { device_id: _, delta, phase: _ } => {
                println!("wheel delta = {:?}", delta);

                match delta {
                    MouseScrollDelta::LineDelta(dx, dy) => {
                        let event = ScrollEvent {
                            dx: dx as f32,
                            dy: dy as f32,
                            mx: self.ui.context.mouse_state.position[0],
                            my: self.ui.context.mouse_state.position[1],
                        };
                        self.handle_scroll_event(&event);
                    },
                    _ => {

                    }
                }
            },

            glium::winit::event::WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                
                println!("key event = {:?}", event);
                // println!("Key = {:?} state = {:?} modifiers = {:?}", event.keycode, event.state, event.modifiers);

                let button_event = ButtonEvent {
                    args: ButtonArgs {
                        state: if event.state == glium::winit::event::ElementState::Pressed { ButtonState::Press } else { ButtonState::Release },
                        button: Button::Keyboard(event.logical_key),
                        scancode: None,
                    },
                    mx: self.ui.context.mouse_state.position[0],
                    my: self.ui.context.mouse_state.position[1],
                };

                self.handle_button_event(&button_event);
            }

            /*
            // By requesting a redraw in response to a RedrawEventsCleared event we get continuous rendering.
            // For applications that only change due to user input you could remove this handler.
            glium::winit::event::Event::AboutToWait => {
                window.request_redraw();
            },
            */

            _ => (),
        }
    }
}


fn main() {
    
    let window_size = [1200, 770];

    // We start by creating the EventLoop, this can only be done once per process.
    // This also needs to happen on the main thread to make the program portable.
    let event_loop = glium::winit::event_loop::EventLoop::builder()
        .build()
        .expect("event loop building");

    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("Rust and Dust Collector v0.0.1")
        .with_inner_size(window_size[0], window_size[1])
        .build(&event_loop);

    let program = build_program(&display);

    let mut app = App::new(window, display, program, window_size);

    event_loop.run_app(&mut app).unwrap();
}
