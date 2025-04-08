use crate::inventory::Inventory;

pub struct Shop
{
    inventory: Inventory,
}


impl Shop
{
    pub fn new() 
    {
        Shop
        {
            inventory: Inventory::new(),
        }
    }
}