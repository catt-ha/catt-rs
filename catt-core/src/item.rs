use value::Value;

pub trait Item {
    type Error: ::std::error::Error;

    fn get_name(&self) -> String;

    fn get_value(&self) -> Result<Value, Self::Error>;
    fn set_value(&self, Value) -> Result<(), Self::Error>;
}
