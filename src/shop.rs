use std::vec::Vec;
use rand::SeedableRng;

use crate::ItemFactory;
use crate::item::Item;
use crate::Inventory;
use crate::Slot;


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


    pub fn sell_item_to_shop(&mut self, item: Item, player_inventory: &mut Inventory, item_factory: &mut ItemFactory)
    {
        let price = item.calc_price() / 2;   // todo: shops never pay full
    
        let silver = price / 100;
        let copper = price % 100;

        if silver > 0 {
            let mut silver_coins = item_factory.create_base("silver_coin");
            silver_coins.stack_size = silver;
            player_inventory.put_item(silver_coins, Slot::Bag);
        }

        if copper > 0 {
            let mut copper_coins = item_factory.create_base("copper_coin");
            copper_coins.stack_size = copper;
            player_inventory.put_item(copper_coins, Slot::Bag);
        }

        self.items.push(item);
    }
}