extern crate chrono;
extern crate futures;
extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate url;

mod schedule;
mod serve;

use chrono::{Datelike, Timelike};
use hyper::rt::Future;
use hyper::{Body, Client, Response, Server};
use schedule::*;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

static DAY_NAMES: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

type BoxFut = Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn main() {
    let mut port = 5104;
    let snap_common_path = match std::env::var("SNAP_USER_COMMON") {
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
    let schedules = Arc::new(Mutex::new(schedules));
    let background = Arc::clone(&schedules);
    let request_thread = std::thread::Builder::new()
        .name("Requestor Thread".into())
        .spawn(move || {
            let client = Arc::from(Client::new());
            let mut waiting = false;
            let mut old_time = chrono::Local::now();
            loop {
                let scheds = background.lock().unwrap();
                let curr_time = chrono::Local::now();
                let mut fired = false;
                if !waiting {
                    for sched in scheds.to_vec() {
                        let day = sched.days[curr_time.weekday().num_days_from_monday() as usize];
                        if curr_time.hour() == day.hour
                            && curr_time.minute() == day.minute
                            && day.enable
                        {
                            fired |= true;
                            let client = Arc::clone(&client);
                            hyper::rt::run(futures::future::lazy(move || {
                                client
                                    .get(hyper::Uri::from_str(&sched.dest).unwrap())
                                    .map(|_res| {})
                                    .map_err(|err| eprintln!("Requestor Error: {}", err))
                            }))
                        }
                    }
                }
                if fired {
                    old_time = chrono::Local::now();
                    waiting = true;
                } else if old_time.minute() != chrono::Local::now().minute() {
                    waiting = false;
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        })
        .unwrap();
    hyper::rt::run(futures::future::lazy(move || {
        let new_service = move || {
            let schedules = schedules.clone();
            let filename = filename.clone();
            hyper::service::service_fn(move |req| {
                serve::web(req, schedules.clone(), filename.clone())
            })
        };
        let addr = std::net::Ipv4Addr::UNSPECIFIED;
        let addr = (addr.octets(), port).into();
        let server = Server::bind(&addr)
            .serve(new_service)
            .map_err(|e| eprintln!("Server Error: {}", e));
        server
    }));
    let _joined = request_thread.join();
}
