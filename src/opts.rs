use clap::{crate_description, crate_name, crate_version, App, Arg, ArgMatches};

#[derive(Debug)]
pub struct Opts {
    pub directory: String,
    pub port: u16,
    pub watch: bool,
}

impl Opts {
    pub fn parse() -> Opts {
        let matches = get_matches_from_clap();

        let directory = matches.value_of("DIRECTORY").unwrap().to_string();
        let port: u16 = match matches.value_of("port").unwrap().parse() {
            Ok(v) => v,
            _ => 3000,
        };
        let watch = matches.is_present("watch");

        Opts {
            directory,
            port,
            watch,
        }
    }
}

fn get_matches_from_clap() -> ArgMatches<'static> {
    App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::with_name("DIRECTORY")
                .help("The directory that will act as the root for static files")
                .default_value("."),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .help("The port on which to run the server")
                .takes_value(true)
                .default_value("3000"),
        )
        .arg(
            Arg::with_name("watch")
                .short("w")
                .long("watch")
                .help("Automatically refresh browsers when a change is detected"),
        )
        .get_matches()
}
