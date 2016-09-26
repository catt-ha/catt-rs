use std::error::Error as StdError;

error_chain!{
    errors {
        Impl(err: Box<StdError + Send>) {
            description("value implementation internal error")
            display("value error: {}", err)
        }
    }
}

pub trait Value {
    fn set_value(&mut self, val: &[u8]) -> Result<()>;
}
