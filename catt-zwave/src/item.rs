use openzwave::value_classes::value_id::ValueID;

use catt_core::item;
use catt_core::value::Value as CValue;

use errors::*;

use driver::ZWave;
use controller::ControllerItem;
use zwave_item::ZWaveItem;

use futures::{self, Future, BoxFuture};

#[derive(Clone, Default)]
pub struct Item {
    controller: Option<ControllerItem>,
    zwave_item: Option<ZWaveItem>,
}

impl Item {
    pub fn controller(name: &str, value: ZWave, home_id: u32) -> Self {
        Item { controller: ControllerItem::new(name, value, home_id).into(), ..Default::default() }
    }

    pub fn item(name: &str, value: ValueID) -> Self {
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

    fn get_value(&self) -> BoxFuture<CValue, Error> {
        let res = match self {
            &Item { controller: None, zwave_item: Some(ref it) } => it.get_value(),
            &Item { controller: Some(ref it), zwave_item: None } => it.get_value(),
            _ => unreachable!(),
        };

        futures::done(res).boxed()
    }

    fn get_meta(&self) -> Option<item::Meta> {
        let res = match self {
            &Item { controller: None, zwave_item: Some(ref it) } => it.get_meta(),
            &Item { controller: Some(ref it), zwave_item: None } => it.get_meta(),
            _ => unreachable!(),
        };

        res
    }

    fn set_value(&self, value: CValue) -> BoxFuture<(), Error> {
        let res = match self {
            &Item { controller: None, zwave_item: Some(ref it) } => it.set_value(value),
            &Item { controller: Some(ref it), zwave_item: None } => it.set_value(value),
            _ => unreachable!(),
        };

        futures::done(res).boxed()
    }
}
