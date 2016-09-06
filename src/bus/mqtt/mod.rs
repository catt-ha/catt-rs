mod errors;
use std::net::TcpStream;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::net::SocketAddr;
use std::str::FromStr;
use mqtt3;
use mqtt3::{MqttRead, MqttWrite};
pub use mqtt3::Connect;
pub use mqtt3::Protocol;

pub use self::errors::*;

pub struct Mqtt {
}

impl Mqtt {
    pub fn connect(address: &str, tls: bool, options: Connect) -> Result<Mqtt> {
        let mut stream = TcpStream::connect(&SocketAddr::from_str(address)?)?;
        let mut rd = BufReader::new(stream.try_clone()?);
        let mut wr = BufWriter::new(stream.try_clone()?);

        let pkt = mqtt3::Packet::Connect(Box::new(options));
        wr.write_packet(&pkt)?;
        wr.flush();

        println!("{:?}", rd.read_packet()?);

        Ok(Mqtt {})
    }
}

#[cfg(test)]
mod test {
    use mioco;
    use super::Mqtt;
    use super::Connect;
    use super::Protocol;
    #[test]
    fn test_connect() {
        Mqtt::connect("10.8.0.1:1883",
                      false,
                      Connect {
                          protocol: Protocol::MQTT(4),
                          keep_alive: 30,
                          client_id: "rust-mq-example-pub".to_owned(),
                          clean_session: true,
                          last_will: None,
                          username: None,
                          password: None,
                      })
            .unwrap();
        panic!();
    }
}
