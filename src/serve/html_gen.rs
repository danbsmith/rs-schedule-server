use crate::BoxFut;
use crate::schedule::{DayInfo, Schedule};
use chrono::{Local, Timelike};

pub fn gen_main_page(schedules: &Vec<Schedule>) -> BoxFut {
    let s = format!("<h1>Hello, Schedule Server</h1><div>Available Schedules:<br>{}</div><div><a href=/newsched/>New Schedule</a></div>", sched_links(schedules));
    html_future_ok(s, hyper::StatusCode::OK)
}

pub fn gen_new_page() -> BoxFut {
    html_future_ok(String::from("<h1>Hello, New Schedule</h1>Name: <form action=\"/newsched/\" method=post><div><input type=\"text\" name=\"name\" minlength=\"1\"></div><div>Destination URL: <input type=\"url\" name=\"url\"></div><br><div><input type=\"submit\" value=\"Create Schedule\"></div></form>"), hyper::StatusCode::OK)
}

pub fn sched_links(schedules: &Vec<Schedule>) -> String {
    let mut lines = String::new();
    for s in schedules {
        lines = lines + &format!("<a href=/sched/{}/>{}</a><br>", s.name, s.name);
    }
    lines
}

pub fn gen_sched_page(schedule: &Schedule) -> BoxFut {
    let monday = sched_form(&schedule.days[0], 0);
    let tuesday = sched_form(&schedule.days[1], 1);
    let wednesday = sched_form(&schedule.days[2], 2);
    let thursday = sched_form(&schedule.days[3], 3);
    let friday = sched_form(&schedule.days[4], 4);
    let saturday = sched_form(&schedule.days[5], 5);
    let sunday = sched_form(&schedule.days[6], 6);
    let curr_time = Local::now();
    let s = format!("<h1>Hello, {0} Editing Page</h1><p>The system time is {8}:{9}</p><div><form action=\"/schedit/update/{0}/\" method=post>{1}{2}{3}{4}{5}{6}{7}<input type=\"submit\" value=\"Update\"></form></div><br><br><div><form action=\"/delete/{0}/\" method=\"post\"><input type=\"submit\" value=\"Delete this schedule\"></form></div>", schedule.name, monday, tuesday, wednesday, thursday, friday, saturday, sunday, curr_time.hour(), curr_time.minute());
    html_future_ok(s, hyper::StatusCode::OK)
}

pub fn sched_form(day: &DayInfo, day_num: u32) -> String {
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
        crate::DAY_NAMES[day_num as usize],
        day.hour,
        day.minute,
        check
    );
    s
}

pub fn html_future_ok(body: String, status: hyper::StatusCode) -> BoxFut {
    Box::new(futures::future::ok(hyper::Response::builder().status(status).body(hyper::Body::from(body)).unwrap()))
}
