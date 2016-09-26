pub const MQTT_BASE_DEFAULT: &'static str = "catt/items";
pub const MQTT_QOS_DEFAULT: u8 = 0;

#[derive(RustcDecodable,Debug,Clone,Default)]
pub struct Config {
    pub broker: String,
    pub item_base: Option<String>,
    pub client_id: Option<String>,
    pub qos: Option<u8>,
    pub tls: Option<bool>,
}
