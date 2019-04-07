extern crate futures;
extern crate hyper;
extern crate serde;
extern crate serde_json;

use hyper::rt::Future;
use hyper::service::{make_service_fn, service_fn_ok};
use hyper::{Body, Request, Response, Server};
use serde::{Deserialize, Serialize};
use std::ops::Add;

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

#[derive(Serialize, Deserialize, Clone)]
struct Schedule {
    dest: String,
    name: String,
    days: [DayInfo; 7],
}

#[derive(Serialize, Deserialize, Clone)]
struct DayInfo {
    hour: u32,
    minute: u32,
    enable: bool,
}

struct ScheduleService {
    schedules: Box<Vec<Schedule>>,
}

impl ScheduleService {
    fn new(scheds: &Vec<Schedule>) -> ScheduleService {
        ScheduleService {
            schedules: Box::new(scheds.to_vec()),
        }
    }
}

impl hyper::service::Service for ScheduleService {
    type Future = BoxFut;
    type Error = hyper::Error;
    type ResBody = Body;
    type ReqBody = Body;
    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        let mut builder = Response::new(Body::empty());
        match (req.method(), req.uri().path()) {
            (&hyper::Method::GET, "/main") => {
                *builder.status_mut() = hyper::StatusCode::OK;
                *builder.body_mut() =
                    Body::from(String::from(gen_main_page(self.schedules.as_ref())));
            }
            (&hyper::Method::GET, s) => {
                for sched in self.schedules.as_ref() {
                    if (format!("/sched/{}/", sched.name)).eq(s) {
                        *builder.status_mut() = hyper::StatusCode::OK;
                        *builder.body_mut() = Body::from(String::from(gen_sched_page(sched)));
                    }
                }
            }
            _ => {
                *builder.status_mut() = hyper::StatusCode::NOT_FOUND;
            }
        }
        Box::new(futures::future::ok(builder))
    }
}

fn gen_main_page(schedules: &Vec<Schedule>) -> String {
    let s = format!("<h1>Hello, Schedule Server</h1><div>Available Schedules:<br>{}</div><div><a href=/newsched/>New Schedule</a></div>", sched_links(schedules));
    s
}

fn sched_links(schedules: &Vec<Schedule>) -> String {
    let mut realLines = String::new();
    for s in schedules {
        realLines = realLines + &format!("<a href=/sched/{}/>{}</a><br>", s.name, s.name);
    }
    realLines
}

fn gen_sched_page(schedule: &Schedule) -> String {
    let monday = sched_form(&schedule.days[0], 0);
    let tuesday = sched_form(&schedule.days[1], 1);
    let wednesday = sched_form(&schedule.days[2], 2);
    let thursday = sched_form(&schedule.days[3], 3);
    let friday = sched_form(&schedule.days[4], 4);
    let saturday = sched_form(&schedule.days[5], 5);
    let sunday = sched_form(&schedule.days[6], 6);
    let s = format!("<h1>Hello, {0} Editing Page</h1><p>The system time is %d:%d</p><div><form action=\"/schedit/update/{0}/\" method=post>{1}{2}{3}{4}{5}{6}{7}<input type=\"submit\" value=\"Update\"></form></div>", schedule.name,monday,tuesday, wednesday, thursday, friday, saturday, sunday);
    s
}

fn sched_form(day: &DayInfo, day_num: u32) -> String {
    let mut check = "";
    if day.enable {
        check = " checked";
    }
    let s = format!(
        "<div>
        {0}
        <input type=\"number\" name=\"{0}_hour\" value=\"{1}\" min=\"0\" max=\"23\">
        :
        <input type=\"number\" name=\"{0}_minute\" value=\"{2}\" min=\"0\" max=\"59\">
        <input type=\"checkbox\" name=\"{0}_enabled\"{3}>
      </div>",
        DAY_NAMES[day_num as usize], day.hour, day.minute, check
    );
    s
}

fn read_schedules(path: &str) -> Vec<Schedule> {
    let sourcefile = std::fs::File::open(path).unwrap();
    let schedules: Vec<Schedule> = serde_json::from_reader(sourcefile).unwrap();
    schedules
}

fn write_schedules(path: &str, schedules: Vec<Schedule>) {
    let destfile = std::fs::File::open(path).unwrap();
    let res = serde_json::to_writer(destfile, &schedules);
    match res {
        Err(e) => eprint!("ERROR: {}", e),
        _ => {}
    }
}

fn main() {
    let mut port = 5104;
    let snap_common_path = std::env::var("SNAP_COMMON").expect("ERROR: No $SNAP_COMMON path set.");
    let mut schedules = read_schedules(&(snap_common_path + "schedules.json"));
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
        }
    }
    let new_service = move || {
        futures::future::ok::<ScheduleService, hyper::Error>(ScheduleService::new(&schedules))
    };
    let addr = std::net::Ipv4Addr::UNSPECIFIED;
    let addr = (addr.octets(), port).into();
    let server = Server::bind(&addr)
        .serve(new_service)
        .map_err(|e| eprintln!("Server Error: {}", e));
    hyper::rt::run(server);
}
