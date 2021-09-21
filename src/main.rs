use std::{error::Error, path::PathBuf};
use clap::{App, Arg, SubCommand};
use offset_caption::{VttParser, VttWriter, SrtWriter, Caption};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("cptcaption")
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
                        .after_help("Creates a file with the extension changed. For example,\ncaption.vtt -> caption.srt"))
                    .get_matches();
   
    // Get the subcommand to run and run it
    if let Some(info_matches) = matches.subcommand_matches("info") {
        let input = info_matches.value_of("INPUT").unwrap();
        let caption = VttParser::from_file(&input)?;
        println!("{:?}", caption);
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
        let mut cap = VttParser::from_file(&input)?;
        cap.offset_milliseconds(offset)?;
        VttWriter::to_file(&output, &cap)?;
    }
    if let Some(concatenate_matches) = matches.subcommand_matches("concatenate") {
        let output = concatenate_matches.value_of("OUTPUT").unwrap();
        let files: Vec<&str> = concatenate_matches.values_of("INPUT")
            .unwrap()
            .collect();
        let mut captions: Vec<Caption> = Vec::with_capacity(files.len());
        for f in files.iter() {
            captions.push(VttParser::from_file(&f)?);
        }
        let mega_caption = Caption::concatenate(captions);
        VttWriter::to_file(&output, &mega_caption)?;
    }
    if let Some(convert_matches) = matches.subcommand_matches("convert") {
        let input = convert_matches.value_of("INPUT").unwrap();
        let caption = VttParser::from_file(&input)?;
        if convert_matches.is_present("srt") {
            let mut path = PathBuf::from(&input);
            path.set_extension("srt");
            SrtWriter::to_file(&path.to_string_lossy(), &caption)?;
        }
    }

    Ok(())
}
