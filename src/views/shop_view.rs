use glium::Frame;

use crate::Inventory;
use crate::GameWorld;
use crate::ButtonEvent;
use crate::MouseMoveEvent;
use crate::ui::UI;
use crate::ui::MouseState;


pub struct ShopView
{

}


impl ShopView
{
    pub fn new() -> ShopView 
    {
        ShopView 
        {

        }
    }


    pub fn draw(&self, ui: &UI, target: &mut Frame) 
    {
        let size = ui.context.window_size;
        let left = 40;
        let width = (size[0] as i32) - left * 2;
        let top = 20;
        let height = (size[1] as i32) - top * 2;

        ui.draw_box(target, left, top, width, height, &[0.6, 0.6, 0.6, 1.0]);
        ui.fill_box(target, left + 1, top + 1, width - 2, height -2 , &[0.3, 0.3, 0.3, 1.0]);

        let font = &ui.context.font_14;
        let text = "Shop under construction, no sales today.";
        let headline_width = font.calc_string_width(text) as i32;
        font.draw(&ui.display, target, &ui.program, 
                  left + (width - headline_width) / 2, top + height/2 - 20, text, &[1.0, 0.8, 0.2, 1.0]);
    }


    pub fn handle_button_event(&mut self, _event: &ButtonEvent, _mouse: &MouseState, _world: &mut GameWorld) -> bool 
    {
        false
    }


    pub fn handle_mouse_move_event(&mut self, _event: &MouseMoveEvent, _mouse: &MouseState, _player_inventory: &mut Inventory) -> bool 
    {
        false
    }

}