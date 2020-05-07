extern crate chrono;
extern crate futures;
extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate url;

mod schedule;
mod serve;

use chrono::{Datelike, Timelike};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Error, Response, Server};
use schedule::*;
use std::future::Future;
use std::pin::Pin;
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

type BoxFut = Pin<Box<dyn Future<Output = Response<Body>> + Send>>;
//type BoxFut = Response<Body>;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    let addr = std::net::Ipv4Addr::UNSPECIFIED;
    let addr = (addr.octets(), port).into();
    let service = move |_: &AddrStream| {
        let filename = (&filename).clone();
        let schedules = (&schedules).clone();
        async move {
            Ok::<_, Error>(service_fn(move |req| {
                let filename = (&filename).clone();
                let schedules = (&schedules).clone();
                async move { Ok::<_, Error>(serve::web(req, &schedules, filename).await) }
            }))
        }
    };
    let make_service = make_service_fn(service);
    let server = Server::bind(&addr).serve(make_service);
    tokio::spawn(async move { server.await });
    let background = (&background).clone();
    let client = Arc::from(Client::new());
    let mut waiting = false;
    let mut old_time = chrono::Local::now();
    println!("Starting Request Thread");
    loop {
        let curr_time = chrono::Local::now();
        let mut fired = false;
        if !waiting {
            let scheds = background.lock().unwrap();
            for sched in scheds.to_vec() {
                let day = sched.days[curr_time.weekday().num_days_from_monday() as usize];
                if curr_time.hour() == day.hour && curr_time.minute() == day.minute && day.enable {
                    fired |= true;
                    let client = Arc::clone(&client);
                    println!("{:?}", sched.dest);
                    let res = tokio::spawn(async move {
                        generate_request(&client.clone(), &(sched.dest).clone()).await
                    })
                    .await;
                    println!("{:?}", res.unwrap().await);
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
}

async fn generate_request(
    client: &hyper::Client<hyper::client::HttpConnector>,
    endpoint: &Endpoint,
) -> hyper::client::ResponseFuture {
    let dest = hyper::Uri::from_str(&endpoint.dest).unwrap();
    println!("Generating request for: {:?}", dest);
    match endpoint.method {
        HttpMethod::GET => client.get(dest),
        HttpMethod::PUT => client.request(
            hyper::Request::builder()
                .method(hyper::Method::PUT)
                .uri(dest)
                .body(hyper::Body::from(endpoint.body.clone()))
                .unwrap(),
        ),
        HttpMethod::POST => client.request(
            hyper::Request::builder()
                .method(hyper::Method::POST)
                .uri(dest)
                .body(hyper::Body::from(endpoint.body.clone()))
                .unwrap(),
        ),
    }
}
