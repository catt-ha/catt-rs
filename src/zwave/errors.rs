use config;
use openzwave_stateful as ozw;

error_chain! {
    links {
        config::Error, config::ErrorKind, Config;
    }

    foreign_links {
        ozw::Error, ZWave;
    }

    errors {

    }
}
