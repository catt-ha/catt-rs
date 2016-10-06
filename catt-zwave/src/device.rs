use openzwave::value_classes::value_id::ValueID;

use std::collections::BTreeMap;

use item::Item;

#[derive(Default)]
pub struct DB {
    values: BTreeMap<ValueID, String>,
    catt_values: BTreeMap<String, Item>,
}

impl DB {
    pub fn get_name(&self, val: &ValueID) -> Option<&String> {
        self.values.get(val)
    }

    pub fn get_item(&self, name: &String) -> Option<&Item> {
        self.catt_values.get(name)
    }

    pub fn add_value(&mut self, name: String, val: ValueID) -> Item {
        let item = Item::item(&name, val);
        self.values.insert(val, name.clone());
        self.catt_values.insert(name.clone(), item.clone());
        item
    }

    pub fn add_item(&mut self, name: String, item: Item) {
        self.catt_values.insert(name, item);
    }

    pub fn remove_item(&mut self, name: &String) -> Option<Item> {
        self.catt_values.remove(name)
    }

    pub fn remove_value(&mut self, val: ValueID) -> Option<Item> {
        self.values.remove(&val).and_then(|name| self.catt_values.remove(&name))
    }
}
