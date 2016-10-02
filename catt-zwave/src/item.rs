use openzwave_stateful as ozw;

use catt_core::item;
use catt_core::value::Value as CValue;

use errors::*;

use driver::ZWave;
use controller::ControllerItem;
use zwave_item::ZWaveItem;

#[derive(Clone, Default)]
pub struct Item {
    controller: Option<ControllerItem>,
    zwave_item: Option<ZWaveItem>,
}

impl Item {
    pub fn controller(name: &str, value: ZWave) -> Self {
        Item { controller: ControllerItem::new(name, value).into(), ..Default::default() }
    }

    pub fn item(name: &str, value: ozw::ValueID) -> Self {
        Item { zwave_item: ZWaveItem::new(name, value).into(), ..Default::default() }
    }
}

impl item::Item for Item {
    type Error = Error;

    fn get_name(&self) -> String {
        if let Some(ref z_item) = self.zwave_item {
            return z_item.get_name();
        }

        if let Some(ref controller) = self.controller {
            return controller.get_name();
        }

        unreachable!()
    }

    fn get_value(&self) -> Result<CValue> {
        if let Some(ref z_item) = self.zwave_item {
            return z_item.get_value();
        }

        if let Some(ref controller) = self.controller {
            return controller.get_value();
        }

        unreachable!()
    }

    fn get_meta(&self) -> Option<item::Meta> {
        if let Some(ref z_item) = self.zwave_item {
            return z_item.get_meta();
        }

        if let Some(ref controller) = self.controller {
            return controller.get_meta();
        }

        unreachable!()
    }

    fn set_value(&self, value: CValue) -> Result<()> {
        if let Some(ref z_item) = self.zwave_item {
            return z_item.set_value(value);
        }

        if let Some(ref controller) = self.controller {
            return controller.set_value(value);
        }

        unreachable!()
    }
}
