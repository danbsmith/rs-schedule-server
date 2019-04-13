mod html_gen;

use crate::schedule::*;
use crate::BoxFut;
use futures::{Future, Stream};
use html_gen::*;
use hyper::{Body, Error, Request, Response};
use std::collections::HashMap;
use url::form_urlencoded;

pub struct ScheduleService {
    schedules: Box<Vec<Schedule>>,
    filepath: String,
}

impl ScheduleService {
    pub fn new(scheds: &Vec<Schedule>, path: String) -> ScheduleService {
        ScheduleService {
            schedules: Box::new(scheds.to_vec()),
            filepath: path.clone(),
        }
    }
}

impl hyper::service::Service for ScheduleService {
    type Future = BoxFut;
    type Error = Error;
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
                    if format!("/sched/{}/", sched.name).eq(s) {
                        *builder.status_mut() = hyper::StatusCode::OK;
                        *builder.body_mut() = Body::from(String::from(gen_sched_page(sched)));
                    }
                }
                if s.eq("/newsched/") {
                    *builder.status_mut() = hyper::StatusCode::OK;
                    *builder.body_mut() = Body::from("<h1>Hello, New Schedule</h1>
                    Name: <form action=\"/newsched/\" method=post><div><input type=\"text\" name=\"name\" minlength=\"1\"></div><div>Destination URL: <input type=\"url\" name=\"url\"></div><div><input type=\"submit\" value=\"Create Schedule\"></div></form>");
                }
            }
            (&hyper::Method::POST, s) => {
                for sched in self.schedules.as_ref() {
                    if format!("/schedit/update/{}/", sched.name).eq(s) {}
                }
                if s.eq("/newsched/") {
                    println!("Posted to /newsched/");
                    return Box::new(req.into_body()
                        .concat2()
                        .map(|b| { let query = form_urlencoded::parse(b.as_ref()).into_owned().collect::<HashMap<String, String>>();
                            let (name_field, uri_field) = match (query.get("name"), query.get("url")) {
                                (Some(nf),Some(uf)) => (nf,uf),
                                _ => return Response::builder().status(hyper::StatusCode::BAD_REQUEST).body(Body::from("<h1>Could not create new schedule.<h1><br><a href=\"/main/\">Go back to main page</a>")).unwrap()
                            };
                            self.schedules.push(Schedule::new(uri_field.to_string(), name_field.to_string()));
                            write_schedules(&self.filepath, self.schedules.to_vec());
                            Response::builder().status(hyper::StatusCode::CREATED).body(Body::from(format!("<h1>Created new schedule</h1><p>Its name is {}</p><br><a href=\"main\">Go back to main page</a>", self.schedules[0].name))).unwrap()
                    }));
                }
            }
            _ => {
                *builder.status_mut() = hyper::StatusCode::NOT_FOUND;
            }
        }
        Box::new(futures::future::ok(builder))
    }
}
