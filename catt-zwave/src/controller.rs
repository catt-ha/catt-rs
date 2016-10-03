use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use catt_core::item::Item;
use catt_core::item::Meta;
use catt_core::value::Value;

use errors::*;

use driver::ZWave;

#[derive(Clone)]
pub struct ControllerItem {
    name: String,
    driver: ZWave,
    home_id: u32,
    state: Arc<Mutex<String>>,
}

impl ControllerItem {
    pub fn new<S: Into<String>>(name: S, driver: ZWave, home_id: u32) -> Self {
        ControllerItem {
            name: name.into(),
            driver: driver,
            home_id: home_id,
            state: Arc::new(Mutex::new("idle".into())),
        }
    }

    pub fn get_state(&self) -> MutexGuard<String> {
        ::catt_core::util::always_lock(self.state.lock())
    }
}

impl Item for ControllerItem {
    type Error = Error;

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_meta(&self) -> Option<Meta> {
        Some(Meta {
            backend: "zwave".to_string().into(),
            value_type: "string".to_string().into(),
            ..Default::default()
        })
    }

    fn get_value(&self) -> Result<Value> {
        let s = self.get_state();
        Ok(Value::String(s.clone()))
    }

    fn set_value(&self, val: Value) -> Result<()> {
        let manager = self.driver.get_manager();
        let home_id = {
            let state = manager.get_state();
            let mut home_ids: Vec<u32> =
                state.get_controllers().keys().map(|c| c.get_home_id()).collect();
            home_ids.sort();
            home_ids[0]
        };

        let cmd = val.as_string()?.trim().to_lowercase();
        match cmd.as_str() {
            "include" => {
                manager.add_node(home_id, false)?;
            }
            "exclude" => {
                manager.remove_node(home_id)?;
            }
            _ => {}
        }

        let mut s = self.get_state();
        *s = cmd;
        Ok(())
    }
}
