use serde::de::Error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Schedule {
    pub dest: String,
    pub name: String,
    pub days: [DayInfo; 7],
}

impl Schedule {
    pub fn new(dest: String, name: String) -> Schedule {
        let days = [DayInfo {
            hour: 0,
            minute: 0,
            enable: false,
        }; 7];
        Schedule { dest, name, days }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct DayInfo {
    pub hour: u32,
    pub minute: u32,
    pub enable: bool,
}

pub fn read_schedules(path: &str) -> Result<Vec<Schedule>, serde_json::Error> {
    let mut res;
    println!("{}", path);
    if let Ok(sourcefile) = std::fs::File::open(path) {
        res = serde_json::from_reader(sourcefile);
    } else {
        res = Err(serde_json::error::Error::custom::<&str>(
            "Couldn't open schedule file",
        ));
    }
    res
}

pub fn write_schedules(path: &str, schedules: Vec<Schedule>) {
    let destfile = std::fs::File::open(path).unwrap();
    let res = serde_json::to_writer(destfile, &schedules);
    match res {
        Err(e) => eprint!("ERROR: {}", e),
        _ => {}
    }
}
