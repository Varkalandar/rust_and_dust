use std::vec::Vec;
use rand::SeedableRng;

use crate::ItemFactory;
use crate::item::Item;

pub struct Shop
{
    pub items: Vec<Item>,
    pub name: String,
}


impl Shop
{
    pub fn new() -> Shop
    {
        Shop
        {
            items: Vec::new(),
            name: "Test Shop".to_string(),
        }
    }


    pub fn restock(&mut self, item_factory: &mut ItemFactory)
    {
        self.items.clear();

        let mut rng = rand::rngs::StdRng::seed_from_u64(12345678901);

        for _i in 0 .. 20 {
            let item = item_factory.create_random(&mut rng, 1);
            self.items.push(item);
        }
    }
}