use core::fmt;
use std::error::Error;

use error_stack::{IntoReport, Report, Result, ResultExt};
use itertools::{enumerate, Itertools};

const NUM_COLUMNS: usize = 16;

#[derive(Debug)]
pub struct Course {
    pub code: String,
    pub title: String,
    pub au: String,
    pub course_type: String,
    pub index: String,
    pub status: String,
    pub classes: Vec<Class>,
    pub exam: Option<Exam>,
}

#[derive(Debug)]
pub struct Class {
    pub weekday: u32,
    pub period: Period,
    pub venue: String,
    pub group: String,
    pub weeks: Vec<u32>,
    pub class_type: String,
}

#[derive(Debug)]
pub struct Exam {
    pub day: u32,
    pub month: u32,
    pub year: u32,
    pub peroid: Period,
}

#[derive(Debug)]
pub struct Period {
    start: Time,
    end: Time,
}

#[derive(Debug)]
pub struct Time {
    hour: u32,
    minute: u32,
}

#[derive(Debug)]
pub enum ParseTableError {
    MissingValues(String),
    UnknownCourse(String),
    Other,
}

impl fmt::Display for ParseTableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Failed to parse table")
    }
}

impl Error for ParseTableError {}

impl Course {
    pub fn parse_from_table(table: String) -> Result<Vec<Self>, ParseTableError> {
        let mut courses = Vec::new();
        for (i, row) in enumerate(&table.replace('\n', "").split('\t').chunks(NUM_COLUMNS)) {
            let row = row.map(|item| item.trim()).collect_vec();
            if row.len() != NUM_COLUMNS {
                let msg = format!(
                    "Missing columns from table on line {}, expected {}, found {}",
                    i,
                    row.len(),
                    NUM_COLUMNS
                );
                return Err(
                    Report::new(ParseTableError::MissingValues(msg.clone())).attach_printable(msg)
                );
            }
            if row[9].is_empty() {
                continue;
            }

            let class = Class {
                weekday: parse_weekday(row[11]).change_context(ParseTableError::Other)?,
                period: parse_period(row[12]).change_context(ParseTableError::Other)?,
                venue: row[13].into(),
                group: row[10].into(),
                weeks: parse_weeks(row[14]).change_context(ParseTableError::Other)?,
                class_type: row[9].into(),
            };

            if let Ok(new_course) = Course::new(
                row[0].into(),
                row[1].into(),
                row[2].into(),
                row[3].into(),
                row[6].into(),
                row[7].into(),
            ) {
                courses.push(new_course);
            }
            if let Some(current_course) = courses.last_mut() {
                current_course.classes.push(class);
            } else {
                let msg = format!("Unknown course for class {:#?}", class);
                return Err(
                    Report::new(ParseTableError::UnknownCourse(msg.clone())).attach_printable(msg)
                );
            }
        }
        Ok(courses)
    }
}

#[derive(Debug)]
pub struct ParseCourseError;

impl fmt::Display for ParseCourseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Failed to parse course")
    }
}

impl Error for ParseCourseError {}

impl Course {
    fn new(
        code: String,
        title: String,
        au: String,
        course_type: String,
        index: String,
        status: String,
    ) -> Result<Self, ParseCourseError> {
        if code.is_empty()
            || title.is_empty()
            || au.is_empty()
            || course_type.is_empty()
            || index.is_empty()
            || status.is_empty()
        {
            Err(Report::new(ParseCourseError))
        } else {
            Ok(Self {
                code,
                title,
                au,
                course_type,
                index,
                status,
                classes: Vec::new(),
                exam: None,
            })
        }
    }
}

#[derive(Debug)]
pub struct ParsePeriodError;

impl fmt::Display for ParsePeriodError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Failed to parse period.")
    }
}

impl Error for ParsePeriodError {}

#[derive(Debug)]
pub struct ParseWeeksError;

fn parse_period(period: &str) -> Result<Period, ParsePeriodError> {
    let re = regex::Regex::new(r"^(\d\d)(\d\d)to(\d\d)(\d\d)$").unwrap();
    let captures = re
        .captures(period)
        .ok_or(ParsePeriodError)
        .into_report()
        .attach_printable_lazy(|| format!("Unable to parse period from {}", period))?;
    Ok(Period {
        start: Time {
            hour: captures.get(1).unwrap().as_str().parse().unwrap(),
            minute: captures.get(2).unwrap().as_str().parse().unwrap(),
        },
        end: Time {
            hour: captures.get(3).unwrap().as_str().parse().unwrap(),
            minute: captures.get(4).unwrap().as_str().parse().unwrap(),
        },
    })
}

impl fmt::Display for ParseWeeksError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Failed to parse weeks")
    }
}

impl Error for ParseWeeksError {}

fn parse_weeks(weeks_raw: &str) -> Result<Vec<u32>, ParseWeeksError> {
    let re = regex::Regex::new("Teaching Wk(.*)").unwrap();
    let weeks_raw = re
        .captures(weeks_raw)
        .ok_or(ParseWeeksError)
        .into_report()
        .attach_printable_lazy(|| "Invalid weeks format".to_string())?
        .get(1)
        .unwrap().as_str();
    let mut weeks = Vec::new();
    let re = regex::Regex::new("^(?<start>[0-9]+)-(?<end>[0-9]+)$").unwrap();
    for x in weeks_raw.split(',') {
        // match week ranges
        if let Some(range) = re.captures(x) {
            let start: u32 = range.name("start").unwrap().as_str().parse().unwrap();
            let end: u32 = range.name("end").unwrap().as_str().parse().unwrap();
            weeks.extend((start..end + 1).collect::<Vec<u32>>());
        } else {
            weeks.push(x.parse::<u32>().unwrap());
        }
    }
    Ok(weeks)
}

#[derive(Debug)]
pub struct ParseWeekdayError;

impl std::fmt::Display for ParseWeekdayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Unable to parse weekday")
    }
}

impl Error for ParseWeekdayError {}

fn parse_weekday(weekday: &str) -> Result<u32, ParseWeekdayError> {
    match weekday {
        "Sun" => Ok(0),
        "Mon" => Ok(1),
        "Tue" => Ok(2),
        "Wed" => Ok(3),
        "Thu" => Ok(4),
        "Fri" => Ok(5),
        "Sat" => Ok(6),
        _ => Err(Report::new(ParseWeekdayError).attach_printable(format!(
            "Unable to parse weekday {}, expected one of: Sun, Mon, Tue, Wed, Thu, Fri, Sat",
            weekday
        ))),
    }
}
