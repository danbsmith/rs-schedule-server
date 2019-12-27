use crate::futures::{Future, Stream};
use crate::schedule::*;
use crate::serve::html_gen::html_future_ok;
use crate::BoxFut;
use hyper::StatusCode;
use std::collections::HashMap;
use url::form_urlencoded;

pub fn create_new_sched(
    req: hyper::Body,
    schedules: std::sync::Arc<std::sync::Mutex<std::vec::Vec<Schedule>>>,
    filepath: String,
) -> BoxFut {
    Box::new(req.concat2()
        .map(move |b| {
            let blank = String::from("");
            let query = form_urlencoded::parse(b.as_ref())
                .into_owned()
                .collect::<std::collections::HashMap<String, String>>();
            let (name_field, uri_field, method_field, body_field) = match (query.get("name"), query.get("url"), query.get("method"), query.get("body")) {
                (Some(nf),Some(uf),Some(mf),Some(bf)) => (nf,uf,mf,bf),
                _ => (&blank, &blank, &blank, &blank)
            };
            if !is_safe_string(&name_field) || !is_safe_string(&uri_field) || !is_safe_string(method_field) {
                return hyper::Response::builder()
                    .status(hyper::StatusCode::BAD_REQUEST)
                    .body(hyper::Body::from("<h1>Could not create new schedule.</h1><br><a href=\"/index/\">Go back to main page</a>"))
                    .unwrap();
            }
            let method = convert_method(method_field).unwrap();
            let dest = Endpoint::new(uri_field.clone(), method, body_field.clone());
            let schedules = &mut schedules.lock().unwrap();
            {
                let schedules: &mut std::vec::Vec<Schedule> = schedules.as_mut();
                schedules.push(Schedule::new(dest.dest, dest.method, dest.body, name_field.to_string()));
                write_schedules(&filepath, schedules);
            }
            let new_name = String::from(schedules[0].name.as_str());
            hyper::Response::builder()
                .status(hyper::StatusCode::CREATED)
                .body(hyper::Body::from(format!("<h1>Created new schedule</h1><p>Its name is {}</p><br><a href=\"/index/\">Go back to main page</a>",
                    new_name)))
                .unwrap()
    }))
}

pub fn edit_sched(
    req: hyper::Body,
    sched_name: String,
    schedules: std::sync::Arc<std::sync::Mutex<std::vec::Vec<Schedule>>>,
    filepath: String,
) -> BoxFut {
    if !is_safe_string(&sched_name) {
        return Box::new(futures::future::ok(hyper::Response::builder()
            .status(hyper::StatusCode::BAD_REQUEST)
            .body(hyper::Body::from("<h1>Could not edit schedule.</h1><br><a href=\"/index/\">Go back to main page</a>"))
            .unwrap()));
    }
    let response = req.concat2().map(move |b| {
            let schedules = &mut schedules.lock().unwrap();
            if let Some(selected) = select_sched(&sched_name, schedules) {
                let name = selected.get_name();
                let dest = selected.dest.clone();
                let mut sched = Schedule::new(dest.dest, dest.method, dest.body, name.clone());
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
                        None => false,
                    };
                    sched.update_day(d, hour, minute, enabled);
                }
                *selected = sched;
                write_schedules(&filepath, schedules);
                return hyper::Response::builder()
                    .status(StatusCode::OK)
                    .body(hyper::Body::from(String::from(format!("<h1>Updated a schedule.</h1><p> Its name is {}</p><br><a href = \"/index/\">Go back to main page</a>", name))))
                    .unwrap()
                }
            hyper::Response::builder().status(StatusCode::NOT_FOUND).body(hyper::Body::from(String::from(format!("<h1>No such schedule</h1><p>There is no schedule named {}</p><br><a href = \"/index/\">Go back to main page</a>", sched_name)))).unwrap()
        });
    return Box::new(response);
}

pub fn delete_sched(
    sched_name: &str,
    schedules: std::sync::Arc<std::sync::Mutex<std::vec::Vec<Schedule>>>,
    filepath: String,
) -> BoxFut {
    if !is_safe_string(&String::from(sched_name)) {
        return Box::new(futures::future::ok(hyper::Response::builder()
            .status(hyper::StatusCode::BAD_REQUEST)
            .body(hyper::Body::from("<h1>Could not delete schedule.</h1><br><a href=\"/index/\">Go back to main page</a>"))
            .unwrap()));
    }
    let mut schedules = schedules.lock().unwrap();
    if let Some(index) = index_sched(sched_name, &schedules) {
        schedules.remove(index);
        let tmp: &std::vec::Vec<Schedule> = schedules.as_ref();
        write_schedules(&filepath, tmp);
        return html_future_ok(
            String::from(
                "<h1>Deleted schedule</h1><br><a href=\"/index/\">Go back to main page</a>",
            ),
            StatusCode::NO_CONTENT,
        );
    } else {
        return html_future_ok(String::from("<h1>No such schedule</h1><p>The schedule you tried to delete doesn't exist.  Were you messing with the query?</p><br><a href=\"/index/\">Click here to go back to the main page</a>"), StatusCode::NOT_FOUND);
    }
}

fn select_sched<'a>(name: &'a str, schedules: &'a mut Vec<Schedule>) -> Option<&'a mut Schedule> {
    schedules.iter_mut().filter(|s| s.name.eq(name)).next()
}

fn index_sched(name: &str, schedules: &Vec<Schedule>) -> Option<usize> {
    schedules.iter().position(|s| s.name == name)
}

fn is_safe_string(input: &String) -> bool {
    input.is_ascii() && !input.is_empty()
}
