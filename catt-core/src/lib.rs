#[macro_use]
extern crate error_chain;

pub mod util;
pub mod bus;
pub mod binding;
pub mod value;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
