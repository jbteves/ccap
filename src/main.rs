use clap::{App, Arg};

use offset_caption::{get_offset, offset_file};

fn main() {
    let matches = App::new("offset_srt")
                    .version("0.1")
                    .author("Joshua B. Teves <joshua.teves@nih.gov>")
                    .arg(Arg::with_name("offset")
                         .long("offset")
                         .takes_value(true)
                         .value_name("OFFSET")
                         .help("The offset time in milliseconds to be \
                         applied"))
                    .arg(Arg::with_name("tail_from")
                         .long("tail_from")
                         .takes_value(true)
                         .value_name("FILE")
                         .help("A file from whose tail you would like to \
                         calculate the offset"))
                    .arg(Arg::with_name("block_offset")
                         .long("block_offset")
                         .takes_value(true)
                         .value_name("OFFSET")
                         .help("The number to add to all block numbers."))
                    .arg(Arg::with_name("INPUT")
                         .help("The file to calculate new offsets for")
                         .required(true))
                    .arg(Arg::with_name("OUTPUT")
                         .help("The file to write with new offsets")
                         .required(true))
                    .get_matches();
    let input = matches.value_of("INPUT").unwrap();
    let output = matches.value_of("OUTPUT").unwrap();
    let block_offset = matches.value_of("block_offset").unwrap_or("0")
        .parse::<usize>().unwrap();

    let mut total_offset: usize = 0;
    if let Some(offset) = matches.value_of("offset") {
        total_offset += offset.parse::<usize>().unwrap();
    }
    if let Some(tail_file) = matches.value_of("tail_from") {
        total_offset += get_offset(tail_file);
        }
    if total_offset == 0 { panic!("No offset given!"); }
    offset_file(input, output, total_offset, block_offset);
}
