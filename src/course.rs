use chrono::{Month, NaiveDate, NaiveTime, Weekday};
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
    pub weekday: Weekday,
    pub period: Period,
    pub venue: String,
    pub group: String,
    pub weeks: Vec<u32>,
    pub class_type: String,
}

#[derive(Debug)]
pub struct Exam {
    pub date: NaiveDate,
    pub peroid: Period,
}

#[derive(Debug)]
pub struct Period {
    pub start: NaiveTime,
    pub end: NaiveTime,
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
    pub fn parse_from_table(table: String, recess_week: u32) -> Result<Vec<Self>, ParseTableError> {
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

            // Create new course if info exists
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

            // Has exam info
            if !row[15].is_empty() && row[15] != "Not Applicable" {
                if let Some(current_course) = courses.last_mut() {
                    current_course.exam =
                        Some(parse_exam(row[15]).change_context(ParseTableError::Other)?);
                }
            }

            // No classes if there is no class_type
            if row[9].is_empty() {
                continue;
            }
            let weekday = match row[11].parse::<Weekday>() {
                Ok(wd) => wd,
                Err(_) => {
                    return Err(Report::new(ParseTableError::Other)
                        .attach_printable(format!("Failed to parse weekday: {}", row[11])))
                }
            };

            let class = Class {
                weekday,
                period: parse_period(row[12]).change_context(ParseTableError::Other)?,
                venue: row[13].into(),
                group: row[10].into(),
                weeks: parse_weeks(row[14], recess_week).change_context(ParseTableError::Other)?,
                class_type: row[9].into(),
            };

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
pub struct ParseExamError;

impl fmt::Display for ParseExamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Failed to parse exam")
    }
}

impl Error for ParseExamError {}

fn parse_exam(exam_raw: &str) -> Result<Exam, ParseExamError> {
    let re = regex::Regex::new(r"(?<day>\d{2})-(?<month>[A-Z][a-z]{2})-(?<year>[0-9]{4}) (?<start_hour>\d{2})(?<start_minute>\d{2})to(?<end_hour>\d{2})(?<end_minute>\d{2})").unwrap();
    let captures = re
        .captures(exam_raw)
        .ok_or(ParseExamError)
        .into_report()
        .attach_printable_lazy(|| format!("Failed to parse date: {}", exam_raw))?;

    let day = captures.name("day").unwrap().as_str();
    let day = day
        .parse()
        .into_report()
        .change_context(ParseExamError)
        .attach_printable(format!("Failed to parse day: {}", day))?;
    let month = captures.name("month").unwrap().as_str();
    let month = match month.parse::<Month>() {
        Ok(m) => m,
        Err(_) => {
            return Err(Report::new(ParseExamError)
                .attach_printable(format!("Failed to parse month: {}", month)))
        }
    };
    let year = captures.name("year").unwrap().as_str();
    let year = year
        .parse()
        .into_report()
        .change_context(ParseExamError)
        .attach_printable(format!("Failed to parse year: {}", year))?;
    let date = NaiveDate::from_ymd_opt(year, month.number_from_month(), day).ok_or(
        Report::new(ParseExamError).attach_printable(format!(
            "Failed to parse date from year: {}, month: {}, day: {}",
            year,
            month.number_from_month(),
            day
        )),
    )?;
    let start_hour = captures
        .name("start_hour")
        .unwrap()
        .as_str()
        .parse()
        .unwrap();
    let start_minute = captures
        .name("start_minute")
        .unwrap()
        .as_str()
        .parse()
        .unwrap();
    let end_hour = captures.name("end_hour").unwrap().as_str().parse().unwrap();
    let end_minute = captures
        .name("end_minute")
        .unwrap()
        .as_str()
        .parse()
        .unwrap();

    let start = NaiveTime::from_hms_opt(start_hour, start_minute, 0).ok_or(
        Report::new(ParseExamError).attach_printable(format!(
            "Failed to parse time {}{}",
            start_hour, start_minute,
        )),
    )?;
    let end = NaiveTime::from_hms_opt(end_hour, end_minute, 0).ok_or(
        Report::new(ParseExamError)
            .attach_printable(format!("Failed to parse time {}{}", end_hour, end_minute,)),
    )?;
    Ok(Exam {
        date,
        peroid: Period { start, end },
    })
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
    let start_hour = captures.get(1).unwrap().as_str().parse().unwrap();
    let start_minute = captures.get(2).unwrap().as_str().parse().unwrap();
    let end_hour = captures.get(3).unwrap().as_str().parse().unwrap();
    let end_minute = captures.get(4).unwrap().as_str().parse().unwrap();

    let start = NaiveTime::from_hms_opt(start_hour, start_minute, 0).ok_or(
        Report::new(ParsePeriodError).attach_printable(format!(
            "Failed to parse time: {}{}",
            start_hour, start_minute
        )),
    )?;
    let end = NaiveTime::from_hms_opt(end_hour, end_minute, 0).ok_or(
        Report::new(ParsePeriodError)
            .attach_printable(format!("Failed to parse time: {}{}", end_hour, end_minute)),
    )?;
    Ok(Period { start, end })
}

impl fmt::Display for ParseWeeksError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Failed to parse weeks")
    }
}

impl Error for ParseWeeksError {}

fn parse_weeks(weeks_raw: &str, recess_week: u32) -> Result<Vec<u32>, ParseWeeksError> {
    let re = regex::Regex::new("Teaching Wk(.*)").unwrap();
    let weeks_raw = re
        .captures(weeks_raw)
        .ok_or(ParseWeeksError)
        .into_report()
        .attach_printable_lazy(|| "Invalid weeks format".to_string())?
        .get(1)
        .unwrap()
        .as_str();
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

    // account for recess_week
    Ok(weeks
        .into_iter()
        .map(|w| if w < recess_week { w } else { w + 1 })
        .collect_vec())
}
