# Fiware-admin: A Simple Administration Tool for Fiware

### Current Version: 0.1.0

Welcome to the repository of **fiware-admin**, a command line tool that provides the following functionality (written in Rust btw):

- Fetch all data in a Fiware instance.
- Delete all data in a service of a Fiware instance.
- Upload data into a Fiware instance from a JSON file.
- Generates random data with certain parameters and uploads the data into a Fiware instance.

This tool is compatible with [the Orion NGSIv2 API](https://fiware-orion.readthedocs.io/en/1.13.0/user/walkthrough_apiv2/).

## Usage

You can use then the tool using the following syntax:

```
Usage: fiware_admin [OPTIONS]

Options:
  -c, --config <config_file>     Path to config file [default: config_file.json]
  -f, --fetch                    Fetches all entities of a given type (all entities if no type specified)
  -t, --type <TYPE>              Specifies the type of the entity to be fetched or modified
  -d, --delete                   Delete all the entities of the given type (all entities if no type given)
  -u, --upload <json_data_file>  Path to JSON file with data to upload. The data should be given as a JSON array of entities with IDs and attributes. Can be specified multiple times.
  -s, --service <service_path>   Name of the Fiware-service path
  -g, --generate                 Generates random data
  -m, --min <MIN>                Minimum value for random data [default: 0]
  -M, --max <MAX>                Maximum value for random data [default: 100]
  -b, --batch-size <BATCH_SIZE>  Number of data points to generate for random data [default: 100]
      --metadata <METADATA>      Path to JSON file with metadata to upload
      --debug                    Enable debug output
  -h, --help                     Print help
  -V, --version                  Print version


```
The different parameters are explained in the following:

- **help:** Displays help screen and exits.
- **config:** Specifies the config file (see below section "Configuration").
- **fetch:** Fetches all entities in the given context. The context name (aka Fiware-Service, or service path) is specified with the **service** (-s) switch. See Examples section below.
- **type:** Specifies a type name for automatically generated data (see **generate** switch).
- **delete:** Deletes **all entities** in a given context. This cannot be undone so **use with care**.
- **upload:** Uploads a JSON file with predefined entities. Example files can be found in the ``examples`` directory.
- **service** Specifies the service path (Fiware-Service). It sets the `fiware-service` header in the NGSIv2 request. If the service path does not exist, the Fiware instance will return an error for the **fetch** operation. Otherwise (**upload** or **generate**) it will create a new service path if the user is authorized. If no service path is provided, the default service path `/` will be used.
- **generate:** Specifies that random data should be generated and uploaded using the given service path.
- **min:** Specifies the minimum value for the generated random measurements. Default value is 0.
- **max:** Specifies the maximum value for the generated random measurements. Default value is 100.
- **batch-size:** Specifies the number of generated random measurements. 
- **metadata:** Specifies the path to a metadata file to be attached to every measurement produced. This is normally used to specify additional properties like name of the sensor or location (see 'Examples' section below).

## Configuration

A JSON configuration file is used to specify an NGSIv2 endpoint and an authentication token. You can use the provided template file `examples/config_fiware.json.template` to create your own config file.

```
{
    "platform": "fiware",
    "config":
    {
        "endpoint": <ENDPOINT>,
        "token": <TOKEN>
    }
}
```

## Examples

Here are some examples of how to use the tool for different purposes:

- Fetch all data from service `water-management`:

    ```
    fiware_admin -c config_fiware.json -f -s water-management
    ```
    or alternatively:

    ```
    fiware_admin --config config_fiware.json --fetch --service water-management
    ```
In the following examples we will use the short notation for convenience.

- Upload the contents of the file `examples/air_quality.json` using the service path `air_quality`: 

    ```
    fiware_admin --config config_fiware.json -u examples/air_quality.json -s air_quality
    ```

- Delete all entities in service `air_quality`:

    ```
    fiware_admin --config config_fiware.json -d -s air_quality
    ```

- Generate 100 random instances of type `AirQualityMeasurement` with a minimum value of 10 and a maximum of 15 in the `air_quality` service. Use the metadata file `sensor1-metadata.json` to simulate a particular sensor on a given location.

    ```
    fiware_admin -c config_fiware.json -g -b 100 -m 10 -M 15 --metadata examples/sensor1-metadata.json -t AirQualityMeasurement -s air_quality
    ```
- Now it is easy to simulate addditional sensors just by providing another metadata file:

    ```
    fiware_admin -c config_fiware.json -g -b 100 -m 15 -M 20 --metadata examples/sensor2-metadata.json -t AirQualityMeasurement -s air_quality
    ```

## License

This software is free under the MIT license was developed in the context of the "Smart Communities" research project, funded by the government of Lower Austria. 

## Author

Egor Evlampiev, IMC Krems University of Applied Sciences (22IMC10282@fh-krems.ac.at)

## Acknowledgments
@rrtorrubiano - the original author of `fiware_admin`
