use std::fmt;
use std::fmt::Display;
use openzwave_stateful::ValueID;
use openzwave_stateful::Node;

use std::collections::BTreeSet;

#[derive(Eq, PartialEq, Debug)]
pub struct Device {
    pub node: Node,
    pub values: BTreeSet<ValueID>,
}

impl PartialOrd for Device {
    fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
        self.node.get_id().partial_cmp(&other.node.get_id())
    }
}

impl Ord for Device {
    fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
        self.node.get_id().cmp(&other.node.get_id())
    }
}

impl Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        PrettyNode(&self.node).fmt(f)?;
        PrettyValues(&self.values).fmt(f)?;
        Ok(())
    }
}

#[derive(Debug)]
struct PrettyNode<'a>(&'a Node);
#[derive(Debug)]
struct PrettyValues<'a>(&'a BTreeSet<ValueID>);


impl<'a> Display for PrettyNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Node {}:", self.0.get_id())?;
        writeln!(f, "\tProduct: {}", self.0.get_product_name())?;
        writeln!(f, "\tManufacturer: {}", self.0.get_manufacturer_name())?;
        writeln!(f, "\tType: {}", self.0.get_type())?;
        Ok(())
    }
}

impl<'a> Display for PrettyValues<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut cc = None;
        for v in self.0.iter() {
            let new_cc = if let Some(c) = cc {
                if v.get_command_class_id() != c as u8 {
                    true
                } else {
                    false
                }
            } else {
                true
            };

            if new_cc {
                cc = Some(v.get_command_class().unwrap());
                writeln!(f, "\tCommandClass {}", v.get_command_class().unwrap())?;
            }

            writeln!(f, "\t\tValue {}:", v.get_index())?;
            writeln!(f, "\t\t\tLabel: {}", v.get_label())?;
            writeln!(f, "\t\t\tHelp {}", v.get_help())?;
            writeln!(f, "\t\t\tGenre: {:?}", v.get_genre())?;
            writeln!(f, "\t\t\tType: {:?}", v.get_type())?;
            writeln!(f, "\t\t\tState: {}", v.as_string().unwrap_or("???".into()))?;
        }
        Ok(())
    }
}
