use std::{fs::File, io::Read};

use clap::Parser;
use ntu_timetable_ics::course::{Course, ParseTableError};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    /// Path to copied time table
    file: String,
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    let mut f = File::open(args.file).expect("Failed to open file");
    let mut table = String::new();
    f.read_to_string(&mut table).expect("Failed to read file");
    let courses = Course::parse_from_table(table);
    match courses {
        Ok(courses) => {
            println!("{:#?}", courses)
        },
        Err(err) => {
            match err.current_context() {
                ParseTableError::Other => println!("Something went wrong"),
                ParseTableError::UnknownCourse(e) => println!("{}", e),
                ParseTableError::MissingValues(e) => println!("{}", e)
            };
            log::error!("\n{:?}", err);
        }

    }
}
