extern crate clap;
extern crate noise_search;
extern crate rocksdb;

use std::io::Write;
use std::process;
use std::str;

use clap::{Arg, App};
use noise_search::index::Index;

fn main() {
    let matches = App::new("dumpnoise")
        .version("0.1")
        .about("Dumps the contents of a Noise instance")
        .arg(Arg::with_name("noise-directory")
             .help("The data directory of the Noise instance")
             .required(true))
        .get_matches();
    let noisedir = matches.value_of("noise-directory").unwrap();

    let mut stderr = std::io::stderr();
    let mut index = Index::new();
    index.open(noisedir, None).unwrap_or_else(|err| {
        writeln!(&mut stderr, "Error: {}", err)
            .expect("Could not write to stderr");
        process::exit(1);
    });

    let db = index.rocks.unwrap();
    println!("Dumping: {}", noisedir);

    for (key, value) in db.iterator(rocksdb::IteratorMode::Start) {
        let key_string = unsafe { str::from_utf8_unchecked((&key)) }.to_string();
        let (type_, value_as_string) = match key_string.chars().next().unwrap() {
            'V' => {
                let value_type = value[0] as char;
                let value_as_string = match value_type {
                    'o' => "{}".to_string(),
                    'a' => "[]".to_string(),
                    's' => format!("\"{}\"", unsafe { str::from_utf8_unchecked((&value[1..])) }),
                    'T' => "true".to_string(),
                    'F' => "false".to_string(),
                    'f' => {
                        unsafe {
                            let array = *(value[1..].as_ptr() as *const [_; 8]);
                            format!("{}", std::mem::transmute::<[u8; 8], f64>(array))
                        }
                    },
                     'N' => "null".to_string(),
                     _ => format!("Unknown value type ({}): {:?}", value_type, value),
                };
                ("Doc".to_string(), value_as_string)
            },
            'W' => {
                ("Word".to_string(), format!("{:?}", value))
            }
            _ => {
                ("Unknown".to_string(), format!("{:?}", value))
            }
        };
        println!("{}\t{}\t{}", type_, key_string, value_as_string);
    }
}
