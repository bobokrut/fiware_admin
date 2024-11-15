use std::{collections::HashSet, fs, io::Write, process::exit};

use clap::Parser;
use random_helper::generate_simple_time_series;
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;
use ureq::Error;

mod client;
mod random_helper;

fn get_data(path: &str) -> Value {
    match fs::exists(path) {
        Ok(true) => {
            let data = fs::read_to_string(path).unwrap();
            serde_json::from_str(&data).unwrap()
        }
        _ => {
            error!("Could not load data file");
            exit(1)
        }
    }
}

#[derive(Parser, Debug, Clone)]
#[command(version, about, author, long_about = None, help_template="\
{name}: {version}
{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
{author-section}
")]
struct Args {
    #[arg(
        short,
        long,
        help = "Path to config file",
        value_name = "config_file",
        default_value = "config_file.json"
    )]
    config: String,

    #[arg(
        short,
        long,
        help = "Fetches all entities of a given type (all entities if no type specified)"
    )]
    fetch: bool,

    #[arg(
        short,
        long,
        help = "Specifies the type of the entity to be fetched or modified",
        default_value = None
    )]
    r#type: Option<String>,

    #[arg(
        short,
        long,
        help = "Delete all the entities of the given type (all entities if no type given)"
    )]
    delete: bool,

    #[arg(
        short,
        long,
        help = "Path to JSON file with data to upload. The data should be given as a JSON array of entities with IDs and attributes. Can be specified multiple times.",
        value_name = "json_data_file"
    )]
    upload: Option<Vec<String>>,

    #[arg(
        short,
        long,
        help = "Name of the Fiware-service path",
        value_name = "service_path",
        default_value = None
    )]
    service: Option<String>,

    #[arg(short, long, help = "Generates random data")]
    generate: bool,

    #[arg(
        short = 'm',
        long,
        help = "Minimum value for random data",
        default_value_t = 0,
        requires = "generate"
    )]
    min: u16,

    #[arg(
        short = 'M',
        long,
        help = "Maximum value for random data",
        default_value_t = 100,
        requires = "generate"
    )]
    max: u16,

    #[arg(
        short,
        long,
        help = "Number of data points to generate for random data",
        default_value_t = 100,
        requires = "generate"
    )]
    batch_size: u16,

    #[arg(
        long,
        help = "Path to JSON file with metadata to upload",
        requires = "generate"
    )]
    metadata: Option<String>,

    #[arg(long, help = "Enable debug output")]
    debug: bool,
}

#[derive(Deserialize)]
struct FiwareClientConfig {
    endpoint: String,
    token: String,
}

#[derive(Deserialize)]
struct FiwareConfig {
    platform: String,
    config: FiwareClientConfig,
}

fn main() {
    let opts = Args::parse();
    let level: Level = if opts.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    match fs::exists(&opts.config) {
        Ok(true) => (),
        _ => {
            error!("Could not load configuration file");
            exit(1)
        }
    }
    let config: FiwareConfig = serde_json::from_str(&fs::read_to_string(&opts.config).unwrap())
        .expect("Failed to parse config file");

    let client = client::Client::new(config, opts.service);

    if opts.fetch {
        info!("Fetching all entities...");
        let result = client.get_all_entities(&opts.r#type, &None);
        debug!("Response: {:?}", result);
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    }

    if opts.delete {
        info!("Deleting all entities...");
        let result = client.delete_all_entities(&opts.r#type);

        match result {
            Ok(r) => {
                debug!("Response {}: {}", r.status(), r.into_string().unwrap());
                info!("Entities deleted successfully");
            }
            Err(Error::Status(code, response)) => {
                error!("Response {}: {}", code, response.into_string().unwrap())
            }
            Err(e) => error!("Failed to delete entities: {}", e),
        }
    }

    if opts.upload.is_some() {
        for to_upload in &opts.upload.unwrap() {
            let json_data = get_data(to_upload);
            let data_types = json_data
                .as_array()
                .unwrap()
                .iter()
                .map(|e| {
                    e.as_object()
                        .unwrap()
                        .get("type")
                        .unwrap()
                        .as_str()
                        .unwrap()
                })
                .collect::<HashSet<_>>();
            info!("Uploading entities of types: {:?}", data_types);
            let result = client.upload_entities(&json_data, false);

            match result {
                Ok(r) => {
                    debug!("Response {}: {}", r.status(), r.into_string().unwrap());
                    info!("Entities uploaded successfully");
                }
                Err(Error::Status(code, response)) => {
                    error!("Response {}: {}", code, response.into_string().unwrap())
                }
                Err(e) => error!("Failed to upload entities: {}", e),
            }
        }
    }

    if opts.generate {
        let metadata = if opts.metadata.is_some() {
            Some(get_data(&opts.metadata.unwrap()))
        } else {
            None
        };
        let result = generate_simple_time_series(
            opts.min,
            opts.max,
            opts.batch_size,
            None,
            opts.r#type,
            metadata,
        );
        let result_str = serde_json::to_string_pretty(&result).unwrap();
        println!("{}", result_str);

        println!("Do you want to upload the generated data? (y/N)");
        std::io::stdout().flush().unwrap();
        let mut answer = String::new();
        std::io::stdin()
            .read_line(&mut answer)
            .expect("Failed to read your input");

        if answer.trim().to_lowercase() == "y" {
            let data_types = result
                .as_array()
                .unwrap()
                .iter()
                .map(|e| {
                    e.as_object()
                        .unwrap()
                        .get("type")
                        .unwrap()
                        .as_str()
                        .unwrap()
                })
                .collect::<HashSet<_>>();

            info!("Uploading entities of types: {:?}", data_types);
            let result = client.upload_entities(&result, false);

            match result {
                Ok(r) => {
                    debug!("Response {}: {}", r.status(), r.into_string().unwrap());
                    info!("Entities uploaded successfully");
                }
                Err(Error::Status(code, response)) => {
                    error!("Response {}: {}", code, response.into_string().unwrap())
                }
                Err(e) => error!("Failed to upload entities: {}", e),
            }
        }
    }
}
