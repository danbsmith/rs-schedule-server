extern crate futures;
extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate url;

mod schedule;
mod serve;

use hyper::rt::Future;
use hyper::{Body, Response, Server};
use schedule::*;
use serve::ScheduleService;

static DAY_NAMES: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn main() {
    let mut port = 5104;
    let snap_common_path = match std::env::var("SNAP_COMMON") {
        Ok(v) => std::path::PathBuf::from(v),
        Err(_) => std::path::PathBuf::from(""),
    };
    let filename = std::path::PathBuf::from("schedules.json");
    let filename = snap_common_path.join(filename);
    let mut filename = filename.to_str();
    if !snap_common_path.is_dir() {
        filename = None;
    }
    let mut tmp = String::default();
    for a in std::env::args() {
        if let Some(ind) = a.rfind("--port=") {
            if let Some(ptext) = a.get(ind + 7..) {
                port = match ptext.parse() {
                    Ok(pnum) => pnum,
                    Err(_) => {
                        eprintln!(
                            "WARNING: Couldn't parse port number ({}), using default",
                            ptext
                        );
                        5104
                    }
                }
            } else {
                port = 5104
            }
        } else if let Some(ind) = a.rfind("--file=") {
            if let Some(filepath) = a.get(ind + 7..) {
                if filename.is_none() {
                    tmp = String::from(filepath);
                }
            }
        }
    }
    let filename = String::from(match filename {
        Some(s) => s,
        None => &tmp,
    });
    let schedules = read_schedules(&filename);
    let schedules = match schedules {
        Ok(val) => val,
        Err(e) => {
            eprintln!("File Parsing Error: {}", e);
            panic!()
        }
    };
    let new_service = move || {
        futures::future::ok::<ScheduleService, hyper::Error>(ScheduleService::new(
            &schedules,
            String::from(filename.as_str()),
        ))
    };
    let addr = std::net::Ipv4Addr::UNSPECIFIED;
    let addr = (addr.octets(), port).into();
    let server = Server::bind(&addr)
        .serve(new_service)
        .map_err(|e| eprintln!("Server Error: {}", e));
    hyper::rt::run(server);
}
