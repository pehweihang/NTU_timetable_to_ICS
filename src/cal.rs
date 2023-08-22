use std::{error::Error, fmt};

use chrono::{DateTime, Datelike, Days, FixedOffset, NaiveDate, NaiveDateTime, Utc};
use error_stack::{Report, Result};
use ics::{
    properties::{Categories, DtEnd, DtStart, Location, Summary},
    Event,
};
use itertools::Itertools;
use uuid::Uuid;

use crate::course::{Class, Course, Exam};

pub fn generate_events(
    courses: &[Course],
    semester_start_date: NaiveDate,
    offset: FixedOffset,
) -> Vec<Event> {
    let mut events = Vec::new();
    for course in courses.iter() {
        for class in course.classes.iter() {
            events.append(
                &mut generate_class_events(
                    course.code.clone(),
                    course.title.clone(),
                    class,
                    semester_start_date,
                    offset,
                )
                .unwrap(),
            );
        }
        if let Some(exam) = &course.exam {
            events.push(generate_exam_event(
                course.code.clone(),
                course.title.clone(),
                exam,
                offset,
            ))
        }
    }
    events
}

#[derive(Debug)]
pub struct DateTimeError;

impl fmt::Display for DateTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Failed to set datetime")
    }
}

impl Error for DateTimeError {}

pub fn generate_class_events(
    course_code: String,
    course_title: String,
    class: &Class,
    semester_start_date: NaiveDate,
    offset: FixedOffset,
) -> Result<Vec<Event>, DateTimeError> {
    let event_title = format!("{} - {} {}", course_code, course_title, class.class_type);
    let semester_start_date = NaiveDate::from_isoywd_opt(
        semester_start_date.year(),
        semester_start_date.iso_week().week(),
        class.weekday,
    )
    .ok_or(Report::new(DateTimeError).attach_printable(format!(
        "Failed to create date from year: {}, week: {}, day: {}",
        semester_start_date.year(),
        semester_start_date.iso_week().week(),
        class.weekday.number_from_sunday()
    )))?;

    Ok(class
        .weeks
        .iter()
        .map(|w| {
            let mut event = Event::new(
                format!("{}-{}", course_code, Uuid::new_v4()),
                to_rfc5545_datetime_with_utc(Utc::now()),
            );
            let date = semester_start_date
                .checked_add_days(Days::new((w - 1) as u64 * 7))
                .unwrap();
            let start_datetime =
                convert_naive_to_utc_datetime(NaiveDateTime::new(date, class.period.start), offset);
            let end_datetime =
                convert_naive_to_utc_datetime(NaiveDateTime::new(date, class.period.end), offset);
            event.push(Summary::new(event_title.clone()));
            event.push(DtStart::new(to_rfc5545_datetime_with_utc(start_datetime)));
            event.push(DtEnd::new(to_rfc5545_datetime_with_utc(end_datetime)));
            event.push(Categories::new(class.class_type.clone()));
            event.push(Location::new(class.venue.clone()));
            event
        })
        .collect_vec())
}

pub fn generate_exam_event(
    course_code: String,
    course_title: String,
    exam: &Exam,
    offset: FixedOffset,
) -> Event {
    let mut event = Event::new(
        format!("{}-{}", course_code, Uuid::new_v4()),
        to_rfc5545_datetime_with_utc(Utc::now()),
    );
    event.push(Summary::new(format!(
        "{} - {} Exam",
        course_code, course_title
    )));
    let start_datetime =
        convert_naive_to_utc_datetime(NaiveDateTime::new(exam.date, exam.peroid.start), offset);
    let end_datetime =
        convert_naive_to_utc_datetime(NaiveDateTime::new(exam.date, exam.peroid.end), offset);
    event.push(DtStart::new(to_rfc5545_datetime_with_utc(start_datetime)));
    event.push(DtEnd::new(to_rfc5545_datetime_with_utc(end_datetime)));
    event.push(Categories::new("Exam"));
    event
}

pub fn to_rfc5545_datetime_with_utc(datetime: DateTime<Utc>) -> String {
    format!("{}", datetime.format("%Y%m%dT%H%M%SZ"))
}

fn convert_naive_to_utc_datetime(datetime: NaiveDateTime, offset: FixedOffset) -> DateTime<Utc> {
    DateTime::<FixedOffset>::from_local(datetime, offset).with_timezone(&Utc)
}
