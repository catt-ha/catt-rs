#![feature(question_mark)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate error_chain;

extern crate openzwave;
extern crate openzwave_stateful;

extern crate catt_core;

extern crate rustc_serialize;

pub mod zwave;
pub mod config;

#[cfg(test)]
extern crate env_logger;

#[cfg(test)]
mod tests {
    use env_logger::LogBuilder;
    use config;
    use zwave;
    use catt_core::binding::Binding;
    use catt_core::binding::Value;

    fn init_logging() {
        let _ = LogBuilder::new().parse("catt_zwave=debug").init();
    }

    #[test]
    fn zwave() {
        init_logging();
        let cfg = config::ZWaveConfig {
            device: vec![
            config::DeviceConfig {
                name: "Switch".into(),
                id: 2,
                zwave_label: Some("Level".into()),
                ..Default::default()
            },
        ],
            ..Default::default()
        };

        let driver = zwave::driver::ZWave::new(cfg).unwrap();

        ::std::thread::sleep(::std::time::Duration::from_secs(5));

        let values: Vec<zwave::value::Value> = driver.get_values().iter().cloned().collect();

        let switch = values[0].clone();

        info!("{:?}", switch);

        switch.set_value("0".as_bytes()).unwrap();

        ::std::thread::sleep(::std::time::Duration::from_secs(2));

        info!("{:?}", switch);

        panic!();
    }
}
