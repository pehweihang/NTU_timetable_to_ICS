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
    #[arg(short, long, default_value_t = 480)]
    minutes_offset: i32,
    /// output file directory
    #[arg(short, long, default_value = "./cal.ics")]
    out: String,
    /// reccess week number
    #[arg(short, long, default_value_t = 8)]
    recess_week: u32
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    let mut f = File::open(args.file).expect("Failed to open timetable file");
    let mut table = String::new();
    f.read_to_string(&mut table).expect("Failed to read file");
    let courses = Course::parse_from_table(table, args.recess_week);
    match courses {
        Ok(courses) => {
            let offset = FixedOffset::east_opt(args.minutes_offset * 60).expect("Invalid offset");
            let mut calendar = ICalendar::new("1.0", "ntu-ics");

            generate_events(&courses, args.semester_start_date, offset)
                .into_iter()
                .for_each(|e| calendar.add_event(e));
            calendar
                .save_file(args.out.clone())
                .expect("Failed to save calendar");

            println!("Saved calendar to: {}", args.out);
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
