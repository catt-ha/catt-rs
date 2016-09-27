use zwave::errors as zwave;
use bus::mqtt::errors as mqtt;
error_chain!{
    links {
        zwave::Error, zwave::ErrorKind, ZWave;
        mqtt::Error, mqtt::ErrorKind, Mqtt;
    }
}
