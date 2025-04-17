use rand::Rng;
use rand::SeedableRng;

use crate::inventory::Inventory;
use crate::Slot;
use crate::ItemFactory;

pub struct Shop
{
    pub inventory: Inventory,
    pub name: String,
}


impl Shop
{
    pub fn new() -> Shop
    {
        Shop
        {
            inventory: Inventory::new(),
            name: "Test Shop".to_string(),
        }
    }


    pub fn restock(&mut self, item_factory: &mut ItemFactory)
    {
        self.inventory.clear();

        let mut rng = rand::rngs::StdRng::seed_from_u64(12345678901);

        for _i in 0 .. 20 {
            let item = item_factory.create_random(&mut rng, 1);
            self.inventory.put_item(item, Slot::Bag);
        }
    }
}