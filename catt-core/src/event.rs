use std::collections::HashMap;
use bus::Message;
use futures::stream::Stream;
use value::Value;

pub enum Event {
    State {
        item_name: String,
        prev_state: Option<Value>,
        new_state: Value,
    },
    Command { item_name: String, command: Value },
    Time { timestamp: i64 },
}

pub trait EventHandler {
    type Error;
    fn handle_event(event: Event) -> Result<(), Self::Error>;
}

pub fn to_event_source<S>(stream: S) -> impl Stream<Item = Event, Error = S::Error>
    where S: Stream<Item = Message>
{
    let mut states: HashMap<String, Value> = Default::default();
    stream.filter_map(move |msg| {
        let (name, value) = match msg {
            Message::Update(name, value) => (name, value),
            Message::Command(name, value) => {
                return Some(Event::Command {
                    item_name: name,
                    command: value,
                })
            }
            _ => return None,
        };

        let mut prev_state = states.insert(name.clone(), value.clone());

        prev_state = match prev_state {
            Some(s) => if s == value { None } else { Some(s) },
            None => None,
        };

        Some(Event::State {
            item_name: name,
            prev_state: prev_state,
            new_state: value,
        })
    })
}
