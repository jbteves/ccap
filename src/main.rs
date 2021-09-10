use clap::{App, Arg};

fn main() {
    let _matches = App::new("offset_srt")
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
}
