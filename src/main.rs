use std::{fs::File, io::Read};

use chrono::FixedOffset;
use clap::Parser;
use ics::ICalendar;
use ntu_timetable_ics::{
    cal::generate_events,
    course::{Course, ParseTableError},
};

#[derive(Parser, Debug)]
struct Args {
    /// input timetable file
    file: String,
    /// date containing starting week of semester
    #[arg(value_parser = is_valid_date)]
    semester_start_date: chrono::NaiveDate,
    /// timezone offset hours
    #[arg(default_value_t = 8)]
    offset_hours: i32,
    /// output file directory
    #[arg(short, long, default_value = "./cal.ics")]
    out: String,
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    let mut f = File::open(args.file).expect("Failed to open timetable file");
    let mut table = String::new();
    f.read_to_string(&mut table).expect("Failed to read file");
    let courses = Course::parse_from_table(table);
    match courses {
        Ok(courses) => {
            let offset = FixedOffset::east_opt(args.offset_hours * 3600).expect("Invalid offset");
            let mut calendar = ICalendar::new("1.0", "ntu-ics");

            generate_events(&courses, args.semester_start_date, offset)
                .into_iter()
                .for_each(|e| calendar.add_event(e));
            calendar
                .save_file(args.out)
                .expect("Failed to save calendar");
        }
        Err(err) => {
            match err.current_context() {
                ParseTableError::Other => println!("Something went wrong"),
                ParseTableError::UnknownCourse(e) => println!("{}", e),
                ParseTableError::MissingValues(e) => println!("{}", e),
            };
            log::error!("\n{:?}", err);
        }
    }
}

fn is_valid_date(s: &str) -> Result<chrono::NaiveDate, String> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|_| format!("{} is not a valid date with format: Y-m-d", s))
}
