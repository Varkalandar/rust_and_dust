use vecmath::{Vector2, vec2_sub, vec2_add, vec2_scale, vec2_normalized};

use glium::Texture2d;
use glium::winit::keyboard::Key;
use glium::Frame;

use crate::ui::{UI, UiController, MouseButton, Button, ButtonState, ButtonEvent, MouseMoveEvent, ScrollEvent};
use crate::GameWorld;
use crate::move_player;
use crate::screen_to_world_pos;
use crate::player_inventory_view::PlayerInventoryView;
use crate::TileSet;
use crate::Map;
use crate::map::MoveEndAction;
use crate::map::MapObject;
use crate::map::MapObjectFactory;
use crate::map::MobType;
use crate::MAP_OBJECT_LAYER;
use crate::PROJECTILE_TILESET;
use crate::gl_support::load_texture;
use crate::MAP_RESOURCE_PATH;
use crate::SoundPlayer;


pub struct Game {
    piv: PlayerInventoryView,
    show_inventory: bool,
}


impl UiController for Game {
    type Appdata = GameWorld;

    /**
     * @return true if this controller could handle the event, false to pass the event to other controllers
     */
     fn handle_button_event(&mut self, ui: &mut UI, event: &ButtonEvent, world: &mut Self::Appdata) -> bool {
        // first pass the even to the UI. if there is a component
        // trigered it will consume the event. Events which are not
        // consumed by the UI will be handed to the game core

        let comp = ui.handle_button_event(&event);

        if event.args.state == ButtonState::Release {

            match comp {
                None => {
                    // the click hit no UI element, so we look into handling it.

                    let pos = screen_to_world_pos(&ui, &world.map.get_player_position(), &ui.context.mouse_state.position);
                    
                    if event.args.button == Button::Mouse(MouseButton::Left) {
                        ui.root.head.clear();

                        let target_opt = Map::find_nearest_object(&world.map.layers[MAP_OBJECT_LAYER], &pos, 100.0, world.map.player_id);
                        match target_opt {
                            None => {},
                            Some(target_uid) => {
                                let target = world.map.layers[MAP_OBJECT_LAYER].get(&target_uid).unwrap();

                                if target.creature.is_some() {
                                    let target_pos = target.position;
                                    fire_projectile(&mut world.map, "Fireball", target_pos, &mut world.speaker);
                                    return true;
                                }
                            }
                        }

                        {
                            let screen_direction = vec2_sub(ui.context.mouse_state.position, ui.window_center());
                            move_player(&mut world.map, screen_direction);                
                        }
                    }

                    if event.args.button == Button::Mouse(MouseButton::Right) {
                        fire_projectile(&mut world.map, "Fireball", pos, &mut world.speaker);
                    }

                    if event.args.button == Button::Keyboard(Key::Character("i".into())) {
                        self.show_inventory = !self.show_inventory;
                    }        
                },
                Some(_comp) => {
                    // the click hit some UI element, so we ignore it.
                }
            }
        
        }

        if self.show_inventory {
            return self.piv.handle_button_event(event, &ui.context.mouse_state, world);
        }

        false
    }


    fn handle_mouse_move_event(&mut self, ui: &mut UI, event: &MouseMoveEvent, world: &mut Self::Appdata) -> bool {
        let _comp = ui.handle_mouse_move_event(event);

        if self.show_inventory {
            self.piv.handle_mouse_move_event(event, &ui.context.mouse_state, &mut world.player_inventory);
        }

        false
    }


    /**
     * @return true if this controller could handle the event, false to pass the event to other controllers
     */
    fn handle_scroll_event(&mut self, ui: &mut UI, event: &ScrollEvent, _world: &mut Self::Appdata) -> bool {

        let _comp = ui.handle_scroll_event(event);

        false
    }


    fn draw(&mut self, target: &mut Frame,
            ui: &mut UI, world: &mut Self::Appdata) {

        ui.draw(target);
 
        if self.show_inventory {
            self.piv.draw(ui, target, 0, 10, &world.player_inventory)
        }
    }


    fn draw_overlay(&mut self, target: &mut Frame,
                    ui: &mut UI, _world: &mut Self::Appdata) {
        ui.context.font_14.draw(&ui.display, target, &ui.program, 10, 20, "Game testing mode", &[1.0, 1.0, 1.0, 1.0]);
    }


    fn update(&mut self, world: &mut Self::Appdata, dt: f32) -> bool {
        let map = &mut world.map;
        let inv = &mut world.player_inventory;
        let rng = &mut world.rng;
        let speaker = &mut world.speaker;
        map.update(dt, inv, rng, speaker);

        let reload = map.check_player_transition(rng);

        reload
    }
}


impl Game {

    pub fn new(inventory_bg: Texture2d, ui: &UI, item_tiles: &TileSet) -> Game {

        let piv = PlayerInventoryView::new(
            (ui.context.window_size[0] as i32) / 2, 0,
            &ui.context.font_14,
            &item_tiles.shallow_copy(),
            inventory_bg,);
    
        Game {
            piv,
            show_inventory: false,
        }
    }
}


pub fn fire_projectile(map: &mut Map, kind: &str, fire_at: Vector2<f32>, speaker: &mut SoundPlayer) -> u64 
{
    let id = map.player_id;
    let player = map.layers[MAP_OBJECT_LAYER].get_mut(&id).unwrap();
    let factory = &mut map.factory;

    let mut projectile = launch_projectile(player.position, fire_at, MobType::PlayerProjectile, factory);
    map.projectile_builder.configure_projectile(kind, &mut projectile.visual, &mut projectile.velocity, speaker);

    let uid = projectile.uid;
    map.layers[MAP_OBJECT_LAYER].insert(uid, projectile);

    uid
}


pub fn launch_projectile(shooter_position: Vector2<f32>, fire_at: Vector2<f32>, 
                       projectile_type: MobType, factory: &mut MapObjectFactory) -> MapObject 
{
    println!("New projectile fired at {:?}", fire_at);

    let np = vec2_sub(fire_at, shooter_position);
    let dir = vec2_normalized(np);
    let start_pos = vec2_add(shooter_position, vec2_scale(dir, 80.0));

    let mut projectile = factory.create_mob(1, PROJECTILE_TILESET, start_pos, 12.0, 0.5);
    projectile.velocity = dir;
    projectile.move_time_left = 2.0;
    projectile.move_end_action = MoveEndAction::RemoveFromMap;
    projectile.mob_type = projectile_type;

    projectile
} 
