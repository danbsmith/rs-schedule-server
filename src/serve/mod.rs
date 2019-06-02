mod html_gen;

use crate::schedule::*;
use crate::BoxFut;
use futures::{Future, Stream};
use html_gen::*;
use hyper::{Body, Request, Response};
use std::collections::HashMap;
use url::form_urlencoded;

pub fn web(
    req: Request<Body>,
    schedules: std::sync::Arc<std::sync::Mutex<std::vec::Vec<Schedule>>>,
    filepath: String,
) -> BoxFut {
    let mut builder = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/index/") => {
            *builder.status_mut() = hyper::StatusCode::OK;
            *builder.body_mut() = Body::from(String::from(gen_main_page(
                schedules.lock().unwrap().as_ref(),
            )));
        }
        (&hyper::Method::GET, uri_path) => {
            for sched in schedules.lock().unwrap().to_vec() {
                if format!("/sched/{}/", sched.name).eq(uri_path) {
                    *builder.status_mut() = hyper::StatusCode::OK;
                    *builder.body_mut() = Body::from(String::from(gen_sched_page(&sched)));
                }
            }
            if uri_path.eq("/newsched/") {
                *builder.status_mut() = hyper::StatusCode::OK;
                *builder.body_mut() = Body::from("<h1>Hello, New Schedule</h1>
                Name: <form action=\"/newsched/\" method=post><div><input type=\"text\" name=\"name\" minlength=\"1\"></div><div>Destination URL: <input type=\"url\" name=\"url\"></div><br><div><input type=\"submit\" value=\"Create Schedule\"></div></form>");
            }
        }
        (&hyper::Method::POST, uri_path) => {
            if uri_path.eq("/newsched/") {
                let result = Box::new(req.into_body()
                    .concat2()
                    .map(move |b| {
                        let query = form_urlencoded::parse(b.as_ref()).into_owned().collect::<HashMap<String, String>>();
                        let (name_field, uri_field) = match (query.get("name"), query.get("url")) {
                            (Some(nf),Some(uf)) => (nf,uf),
                            _ => return Response::builder().status(hyper::StatusCode::BAD_REQUEST).body(Body::from("<h1>Could not create new schedule.<h1><br><a href=\"/index/\">Go back to main page</a>")).unwrap()
                        };
                        let schedules = &mut schedules.lock().unwrap();
                        {
                            let schedules: &mut std::vec::Vec<Schedule> = schedules.as_mut();
                            schedules.push(Schedule::new(uri_field.to_string(), name_field.to_string()));
                            write_schedules(&filepath, schedules);
                        }
                        let new_name = String::from(schedules[0].name.as_str());
                        Response::builder().status(hyper::StatusCode::CREATED).body(Body::from(format!("<h1>Created new schedule</h1><p>Its name is {}</p><br><a href=\"/index/\">Go back to main page</a>", new_name))).unwrap()
                }));
                return result;
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
                                return Box::new(futures::future::ok(
                                    Response::builder().status(hyper::StatusCode::NO_CONTENT).body(Body::from("<h1>Deleted schedule</h1><br><a href=\"/index/\">Go back to main page</a>")).unwrap()
                                ));
                            } else {
                                return Box::new(futures::future::ok(Response::builder().status(hyper::StatusCode::NOT_FOUND).body(Body::from("<h1>No such schedule</h1><p>The schedule you tried to access doesn't exist.  Were you messing with the query?</p><br><a href=\"/index/\">Click here to go back to the main page</a>")).unwrap()));
                            }
                        }
                    }
                }
            }
        }
        _ => {
            *builder.status_mut() = hyper::StatusCode::NOT_FOUND;
        }
    }
    Box::new(futures::future::ok(builder))
}
