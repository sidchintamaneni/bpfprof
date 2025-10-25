use std::fmt;

#[derive(Clone, Debug)]
pub struct Bpfprog {
    pub id: u32,
    pub bpf_type: String,
    pub name: String,
}

impl fmt::Display for Bpfprog {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ID: {} | Type: {} | Name: {}", self.id, self.bpf_type, self.name)
    }
}

impl PartialEq for Bpfprog {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Bpfprog {
    pub fn new(id: u32, bpf_type: String, name: String) -> Self {
        Self {
            id,
            bpf_type,
            name,
        }
    }
}

