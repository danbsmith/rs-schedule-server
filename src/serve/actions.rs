use crate::BoxFut;
use url::form_urlencoded;
use crate::futures::{Future, Stream};
use crate::schedule::*;

pub fn create_new_sched(req: hyper::Request<hyper::Body>,
                        schedules: std::sync::Arc<std::sync::Mutex<std::vec::Vec<Schedule>>>,
                        filepath: String) -> BoxFut {
    Box::new(req.into_body()
        .concat2()
        .map(move |b| {
            let query = form_urlencoded::parse(b.as_ref())
            .into_owned()
            .collect::<std::collections::HashMap<String, String>>();
            let (name_field, uri_field) = match (query.get("name"), query.get("url")) {
                (Some(nf),Some(uf)) => (nf,uf),
                _ => return hyper::Response::builder()
                .status(hyper::StatusCode::BAD_REQUEST)
                .body(hyper::Body::from("<h1>Could not create new schedule.<h1><br><a href=\"/index/\">Go back to main page</a>"))
                .unwrap()
            };
            let schedules = &mut schedules.lock().unwrap();
            {
                let schedules: &mut std::vec::Vec<Schedule> = schedules.as_mut();
                schedules.push(Schedule::new(uri_field.to_string(), name_field.to_string()));
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
