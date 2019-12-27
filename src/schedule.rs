use serde::de::Error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Schedule {
    pub dest: Endpoint,
    pub name: String,
    pub days: [DayInfo; 7],
}

impl PartialEq for Schedule {
    fn eq(&self, rhs: &Self) -> bool {
        self.name == rhs.name
    }
}

impl Schedule {
    pub fn new(dest: String, method: HttpMethod, body: String, name: String) -> Schedule {
        let days = [DayInfo {
            hour: 0,
            minute: 0,
            enable: false,
        }; 7];
        let dest = Endpoint::new(dest, method, body);
        Schedule { dest, name, days }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn update_day(&mut self, ind: usize, hour: u32, minute: u32, enable: bool) -> () {
        self.days[ind] = DayInfo {
            hour,
            minute,
            enable,
        };
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum HttpMethod {
    GET,
    PUT,
    POST,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Endpoint {
    pub method: HttpMethod,
    pub dest: String,
    pub body: String,
}

impl Endpoint {
    pub fn new(dest: String, method: HttpMethod, body: String) -> Endpoint {
        Endpoint { method, dest, body }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct DayInfo {
    pub hour: u32,
    pub minute: u32,
    pub enable: bool,
}

pub fn read_schedules(path: &str) -> Result<Vec<Schedule>, serde_json::Error> {
    let res;
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

pub fn write_schedules(path: &str, schedules: &Vec<Schedule>) {
    let destfile = std::fs::File::create(path).unwrap();
    if let Err(res) = serde_json::to_writer(destfile, schedules) {
        eprint!("ERROR: {:?}, path was {}", res, path);
    }
}

pub fn convert_method(method: &str) -> Option<HttpMethod> {
    match method {
        "GET" => Some(HttpMethod::GET),
        "PUT" => Some(HttpMethod::PUT),
        "POST" => Some(HttpMethod::POST),
        _ => None,
    }
}
