use std::{error::Error, path::PathBuf};
use clap::{App, Arg, SubCommand};
use ccap::{
    SimpleTime,
    write_caption, parse_file,
    VttParser, VttWriter, SrtWriter,
    Caption
};

fn parse_time(time: Option<&str>, as_millis: bool) -> Result<Option<SimpleTime>, Box<dyn Error>> {
    let t = match time {
        Some(t) => {
            if as_millis {
                Some(SimpleTime::from_milliseconds(t.parse::<usize>()?))
            }
            else {
                Some(VttParser::block_timestamp(&t)?)
            }
        },
        None => None,
    };
    Ok(t)
}


fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Captain Caption")
                    .version("0.1")
                    .author("Joshua B. Teves <joshua.teves@nih.gov>")
                    .subcommand(
                        SubCommand::with_name("info")
                        .about("Get information about the caption file")
                        .arg(Arg::with_name("INPUT")
                             .required(true)
                             .takes_value(true)
                             .help("The file to get information for")))
                    .subcommand(
                        SubCommand::with_name("offset")
                        .about("offset a caption file by some time")
                        .arg(Arg::with_name("INPUT")
                             .required(true)
                             .takes_value(true)
                             .help("File to be offset"))
                        .arg(Arg::with_name("OUTPUT")
                             .required(true)
                             .takes_value(true)
                             .help("Name of the resulting file"))
                        .arg(Arg::with_name("OFFSET")
                             .required(true)
                             .takes_value(true)
                             .help("Offset to apply as HH:MM:SS.mmm"))
                        .arg(Arg::with_name("millis")
                             .long("millis")
                             .help("Supply offset in milliseconds instead"))
                        .arg(Arg::with_name("subtract")
                             .long("subtract")
                             .help("Subtract instead of add offset")))
                    .subcommand(
                        SubCommand::with_name("concatenate")
                        .about("Concatenate multiple caption files")
                        .arg(Arg::with_name("OUTPUT")
                             .required(true)
                             .takes_value(true)
                             .help("The output filename"))
                        .arg(Arg::with_name("INPUT")
                             .required(true)
                             .takes_value(true)
                             .min_values(2)
                             .help("The files to concatenate")))
                    .subcommand(
                        SubCommand::with_name("convert")
                        .about("Convert caption formats")
                        .arg(Arg::with_name("INPUT")
                             .takes_value(true)
                             .required(true)
                             .help("The file to be converted"))
                        .arg(Arg::with_name("srt")
                             .long("srt")
                             .help("Convert to SRT"))
                        .arg(Arg::with_name("vtt")
                             .long("vtt")
                             .help("Convert to VTT"))
                        .after_help("Creates a file with the extension changed. For example,\ncaption.vtt -> caption.srt"))
                    .subcommand(
                        SubCommand::with_name("crop")
                        .about("Crop a caption")
                        .arg(Arg::with_name("INPUT")
                             .takes_value(true)
                             .help("The input filename"))
                        .arg(Arg::with_name("OUTPUT")
                             .takes_value(true)
                             .help("The output filename"))
                        .arg(Arg::with_name("millis")
                             .long("millis")
                             .help("Supply offset in milliseconds instead"))
                        .arg(Arg::with_name("from")
                             .long("from")
                             .takes_value(true)
                             .required_unless("to")
                             .help("Time to crop from (inclusive)"))
                        .arg(Arg::with_name("to")
                             .long("to")
                             .takes_value(true)
                             .required_unless("from")
                             .help("Time to crop to (inclusive)"))
                        .after_help("Creates a new file that is cropped"))
                    .get_matches();
   
    // Get the subcommand to run and run it
    if let Some(info_matches) = matches.subcommand_matches("info") {
        let input = info_matches.value_of("INPUT").unwrap();
        let caption = parse_file(&input)?;
        println!("File: {}", input);
        caption.print_report();
    }
    if let Some(offset_matches) = matches.subcommand_matches("offset") {
        let input = offset_matches.value_of("INPUT").unwrap();
        let output = offset_matches.value_of("OUTPUT").unwrap();
        let offset_str = offset_matches.value_of("OFFSET").unwrap();
        let offset_millis = {
            if offset_matches.is_present("millis") {
                offset_str.parse::<isize>()?
            }
            else {
                let st = VttParser::block_timestamp(&offset_str)?;
                st.to_milliseconds() as isize
            }
        };
        let offset = {
            if offset_matches.is_present("subtract") {
                0 - offset_millis
            }
            else {
                offset_millis
            }
        };
        let mut cap = parse_file(&input)?;
        cap.offset_milliseconds(offset)?;
        write_caption(&output, &cap)?;
    }
    if let Some(concatenate_matches) = matches.subcommand_matches("concatenate") {
        let output = concatenate_matches.value_of("OUTPUT").unwrap();
        let files: Vec<&str> = concatenate_matches.values_of("INPUT")
            .unwrap()
            .collect();
        let mut captions: Vec<Caption> = Vec::with_capacity(files.len());
        for f in files.iter() {
            captions.push(parse_file(&f)?);
        }
        let mega_caption = Caption::concatenate(captions);
        write_caption(&output, &mega_caption)?;
    }
    if let Some(convert_matches) = matches.subcommand_matches("convert") {
        let input = convert_matches.value_of("INPUT").unwrap();
        let caption = parse_file(&input)?;
        if convert_matches.is_present("srt") {
            let mut path = PathBuf::from(&input);
            path.set_extension("srt");
            SrtWriter::to_file(&path.to_string_lossy(), &caption)?;
        }
        if convert_matches.is_present("vtt") {
        let mut path = PathBuf::from(&input);
            path.set_extension("vtt");
            VttWriter::to_file(&path.to_string_lossy(), &caption)?;
        }
    }
    if let Some(crop_matches) = matches.subcommand_matches("crop") {
        let input = crop_matches.value_of("INPUT").unwrap();
        let output = crop_matches.value_of("OUTPUT").unwrap();
        let mut caption = parse_file(&input)?;
        let use_millis = crop_matches.is_present("millis");
        let from = parse_time(crop_matches.value_of("from"), use_millis)?;
        let to = parse_time(crop_matches.value_of("to"), use_millis)?;
        caption.crop(from, to);
        write_caption(&output, &caption)?;
    }

    Ok(())
}
