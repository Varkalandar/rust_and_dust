use crate::inventory::Inventory;

pub struct Shop
{
    inventory: Inventory,
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
}