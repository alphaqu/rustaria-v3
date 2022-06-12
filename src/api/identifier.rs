use std::fmt::{Display, Formatter, Write};

/// The identifier is a dual-string notifying which mod (namespace) the entry is from. and what it is.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn new(path: &'static str) -> Identifier {
        Identifier {
            namespace: "rustaria".to_string(),
            path: path.to_string()
        }
    }
}


impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.namespace)?;
        f.write_char(':')?;
        f.write_str(&self.path)?;
        Ok(())
    }
}