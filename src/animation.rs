use crate::map::MapObject;
use crate::map::UpdateAction;
use crate::ANIMATION_TILESET;
use crate::gl_support::BlendMode;

pub trait Animated {
    fn update(&self, _dt: f32, _mob: &mut MapObject) {

    }
}

pub struct NoAnimation {

}

impl Animated for NoAnimation {

}


pub struct SpinAnimation {
    speed: f32,
}

impl SpinAnimation {
    pub fn new(speed: f32) -> SpinAnimation {
        SpinAnimation {
            speed,
        }
    }
}

impl Animated for SpinAnimation {
    fn update(&self, dt: f32, mob: &mut MapObject) {
        mob.animation_timer += dt;

        let frame = (mob.animation_timer * self.speed) as usize;
        mob.visual.current_image_id = mob.visual.base_image_id + (frame % 8);
    }
}


pub struct RemovalAnimation {
    time_limit: f32,
    timer_start: f32, 
}

impl RemovalAnimation {
    pub fn new(timer_start: f32, time_limit: f32) -> RemovalAnimation {
        RemovalAnimation {
            time_limit,
            timer_start,
        }
    }
}

impl Animated for RemovalAnimation {
    fn update(&self, dt: f32, mob: &mut MapObject) {
        mob.animation_timer += dt;

        let countdown = mob.animation_timer - self.timer_start;
        // println!("Time left: {}", self.time_limit - countdown);

        if countdown < self.time_limit {
            let completion = countdown / self.time_limit;
            let tile_id = 1 + ((completion * 22.0) as usize);

            // println!("tile id = {}", tile_id);

            mob.visual.current_image_id = tile_id;
            mob.visual.color = [1.0, 1.0, 1.0, 1.0];
            mob.visual.blend = BlendMode::Add;
            mob.visual.tileset_id = ANIMATION_TILESET;
            mob.visual.scale = 1.5;
        }
        else {
            mob.update_action = UpdateAction::RemoveFromMap;
        }
    }

}
