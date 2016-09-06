extern crate catt;
extern crate env_logger;

use env_logger::LogBuilder;

use catt::zwave_testing::run;

fn main() {
    LogBuilder::new().parse("catt=debug").init().unwrap();

    run()
}
