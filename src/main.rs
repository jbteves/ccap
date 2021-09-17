use std::error::Error;
use clap::{App, Arg, SubCommand};
use offset_caption::{ParseFrom, VttParser};

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
                    .get_matches();
   
    // Get the subcommand to run and run it
    if let Some(info_matches) = matches.subcommand_matches("info") {
        let input = info_matches.value_of("INPUT").unwrap();
        let caption = VttParser::parse(ParseFrom::File(&input))?;
        println!("{:?}", caption);
    }

    Ok(())
}
