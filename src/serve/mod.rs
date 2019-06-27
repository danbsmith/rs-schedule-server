mod html_gen;
mod actions;

use crate::schedule::*;
use crate::BoxFut;
use futures::{Future, Stream};
use html_gen::*;
use hyper::{Body, Request, Response, StatusCode};
use std::collections::HashMap;
use url::form_urlencoded;

pub fn web(
    req: Request<Body>,
    schedules: std::sync::Arc<std::sync::Mutex<std::vec::Vec<Schedule>>>,
    filepath: String,
) -> BoxFut {
    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/index/") => {
            return gen_main_page(schedules.lock().unwrap().as_ref());
        }
        (&hyper::Method::GET, uri_path) => {
            for sched in schedules.lock().unwrap().to_vec() {
                if format!("/sched/{}/", sched.name).eq(uri_path) {
                    return gen_sched_page(&sched);
                }
            }
            if uri_path.eq("/newsched/") {
                return gen_new_page();
            }
        }
        (&hyper::Method::POST, uri_path) => {
            if uri_path.eq("/newsched/") {
                return actions::create_new_sched(req, schedules, filepath);
            } else {
                let mut selected = None;
                {
                    let schedules = &schedules.lock().unwrap();
                    for v in 0..schedules.to_vec().len() {
                        if format!("/schedit/update/{}/", schedules.to_vec()[v].name).eq(uri_path) {
                            selected = Some(v);
                        }
                    }
                }
                {
                    if let Some(selected) = selected {
                        let result = Box::new(req.into_body().concat2().map(move |b| {
                        let mut schedules = schedules.lock().unwrap();
                        let sched = &mut schedules[selected];
                        let query = form_urlencoded::parse(b.as_ref())
                            .into_owned()
                            .collect::<HashMap<String, String>>();
                        for d in 0..7 {
                            let h = format!("{}_hour", crate::DAY_NAMES[d]);
                            let m = format!("{}_minute", crate::DAY_NAMES[d]);
                            let e = format!("{}_enabled", crate::DAY_NAMES[d]);
                            let input = (
                                query.get(&h).unwrap(),
                                query.get(&m).unwrap(),
                                query.get(&e),
                            );
                            let hour = input.0.parse().unwrap();
                            let minute = input.1.parse().unwrap();
                            let enabled = match input.2 {
                                Some(_) => true,
                                None => false
                            };
                            sched.update_day(d, hour, minute, enabled);
                        }
                        let tmp: &std::vec::Vec<Schedule> = schedules.as_ref();
                        write_schedules(&filepath, tmp);
                        let name = String::from(tmp[selected].name.as_str());
                        Response::builder().status(hyper::StatusCode::OK).body(Body::from(format!("<h1>Updated a schedule.</h1><p> Its name is {}</p><br><a href = \"/index/\">Go back to main page</a>", name))).unwrap()
                    }));
                        return result;
                    } else {
                        let mut selected = None;
                        {
                            let schedules = &schedules.lock().unwrap();
                            for v in 0..schedules.to_vec().len() {
                                if format!("/delete/{}/", schedules.to_vec()[v].name).eq(uri_path) {
                                    selected = Some(v);
                                }
                            }
                        }
                        {
                            if let Some(selected) = selected {
                                let mut schedules = schedules.lock().unwrap();
                                schedules.remove(selected);
                                let tmp: &std::vec::Vec<Schedule> = schedules.as_ref();
                                write_schedules(&filepath, tmp);
                                return html_future_ok(String::from("<h1>Deleted schedule</h1><br><a href=\"/index/\">Go back to main page</a>"), StatusCode::NO_CONTENT);
                            } else {
                                return html_future_ok(String::from("<h1>No such schedule</h1><p>The schedule you tried to access doesn't exist.  Were you messing with the query?</p><br><a href=\"/index/\">Click here to go back to the main page</a>"), StatusCode::NOT_FOUND);
                            }
                        }
                    }
                }
            }
        }
        _ => {
        return html_future_ok(String::from(""), StatusCode::NOT_FOUND);
        }
    }
html_future_ok(String::from(""), StatusCode::INTERNAL_SERVER_ERROR)
}

fn select_sched<'a>(name: &str, schedules: &'a mut Vec<Schedule>) -> Option<&'a mut  Schedule> {
    schedules.iter_mut().filter(|s|{s.name.eq(name)}).next()
}

fn index_sched(name: &str, schedules: &Vec<Schedule>) -> Option<usize> {
    schedules.iter().position(|s| {s.name == name})
}
