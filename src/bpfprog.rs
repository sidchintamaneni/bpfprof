use std::fmt;

#[derive(Clone, Debug)]
pub struct Bpfprog {
    pub id: u32,
    pub bpf_type: String,
    pub name: String,
    pub run_time_ns: u64,
    pub run_cnt: u64,
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
    pub fn new(id: u32, bpf_type: String, name: String, run_time_ns: u64, run_cnt: u64) -> Self {
        Self {
            id,
            bpf_type,
            name,
            run_time_ns,
            run_cnt,
        }
    }
}

