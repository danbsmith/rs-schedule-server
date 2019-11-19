mod actions;
mod html_gen;

use crate::schedule::*;
use crate::BoxFut;
use html_gen::*;
use hyper::{Body, Request, StatusCode};

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
            } else {
                return html_future_ok(String::from("<h1>Not Found</h1><p>The page you requested could not be located.</p><a href=\"/index/\">Return to main page</a>"), StatusCode::NOT_FOUND);
            }
        }
        (&hyper::Method::POST, uri_path) => {
            let path_parts: std::vec::Vec<&str> =
                uri_path.split('/').filter(|s| !s.is_empty()).collect();
            if path_parts.len() == 1 && path_parts[0].eq("newsched") {
                return actions::create_new_sched(req.into_body(), schedules, filepath);
            } else if path_parts.len() == 3
                && path_parts[0].eq("schedit")
                && path_parts[1].eq("update")
            {
                let sched_name = String::from(path_parts[2]);
                return actions::edit_sched(req.into_body(), sched_name, schedules, filepath);
            } else if path_parts.len() == 2 && path_parts[0].eq("delete") {
                return actions::delete_sched(path_parts[1], schedules, filepath);
            } else {
                return html_future_ok(
                    String::from("<p>No posting to that path.</p>"),
                    StatusCode::NOT_FOUND,
                );
            }
        }
        _ => {
            return html_future_ok(
                String::from("<p>I can't find that path.</p>"),
                StatusCode::NOT_FOUND,
            );
        }
    }
}
