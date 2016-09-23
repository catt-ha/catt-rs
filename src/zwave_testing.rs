#![allow(dead_code)]
#![allow(unused_imports)]

use util::CVar;

use openzwave_stateful::{ConfigPath, InitOptions, ValueGenre, ValueID, ValueType,
                         ZWaveNotification, init};
use shellexpand::full;
use std::env;
use std::sync::Arc;

use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;
use std::collections::BTreeSet;
use std::thread;

use std::time;

use zwave::driver::Driver;

const MQTT_BROKER: &'static str = "10.8.0.1:1883";
const MQTT_BASE: &'static str = "/openhab";
const MQTT_COMMAND_PUBLISH: &'static str = "/out/{{item}}/command";
const MQTT_STATE_PUBLISH: &'static str = "/out/{{item}}/state";
const MQTT_COMMAND_SUBSCRIBE: &'static str = "/in/{{item}}/command";
const MQTT_STATE_SUBSCRIBE: &'static str = "/in/{{item}}/state";

const ZWAVE_DEVICE: &'static str = "/dev/ttyUSB0";

const ZWAVE_CONFIG_PATH: &'static str = "/etc/openzwave-git";
const ZWAVE_USER_PATH: &'static str = "~/.ozw";

lazy_static! {
    static ref ZWAVE_USER_PATH_EXPANDED: String = full(ZWAVE_USER_PATH).unwrap().into_owned();
}

use openzwave_stateful::CommandClass;
use std::time::Duration;

pub fn run() {
    info!("User path: {}", *ZWAVE_USER_PATH_EXPANDED);

    let driver = Driver::new(Some(ZWAVE_DEVICE),
                             Some(ZWAVE_CONFIG_PATH),
                             &*ZWAVE_USER_PATH_EXPANDED)
        .unwrap();

    info!("Waiting for controller to become ready...");

    // wait for the thread to start up
    driver.wait_ready();

    let light_set = driver.state()
        .get_values()
        .iter()
        .cloned()
        .filter(|v| v.get_command_class_id() == CommandClass::SwitchBinary as u8)
        .collect::<BTreeSet<ValueID>>();
    light_set.iter()
        .map(|v| {
            v.set_bool(false).unwrap();
        })
        .collect::<Vec<()>>();
    thread::sleep(Duration::from_secs(5));
    light_set.iter()
        .map(|v| {
            v.set_bool(true).unwrap();
        })
        .collect::<Vec<()>>();
    thread::sleep(Duration::from_secs(5))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
