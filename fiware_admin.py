import argparse
import json
from client import FiwareClient

## Helper functions
def get_type(args):
    """
    Returns the entity type if given.
    """
    type = None
    if args.type is not None:
        type = args.type
    return type

def check_if_file_exists(path):
    """
    Checks if a given path exists
    """
    try:
        f = open(path)
    except FileNotFoundError:
        return False
    f.close()
    return True


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='General Fiware admin util for Dataskop/Smart Communities projects.',
                                     epilog='Author: Ruben Ruiz-Torrubiano (ruben.ruiz@fh-krems.ac.at)')
    

    # BEGIN PARSING ARGUMENTS
    # Config file
    parser.add_argument('-c', '--config', metavar='<config_file>', nargs=1, 
                        help='Path to config file')
    
    # Get all entities
    parser.add_argument('-G', '--get_all',
                        action='store_true', 
                        help='Gets all entities of a given type (all entities if no type specified).')
    
    # Specify entity type
    parser.add_argument('-t', '--type',
                        help='Specifies the type of the entity to be fetched or modified.')
    
    # Delete all entities
    parser.add_argument('-D', '--delete',
                        action='store_true',
                        help='Delete all the entities of the given type (all entities if no type given)')
    
    # Upload data from JSON file
    parser.add_argument('-U', '--upload', metavar='<json_data_file>',
                        help='Path to JSON file with data to upload. The data should be given as a JSON array of entities with IDs and attributes.')
    
    # Specify the Fiware-Service path
    parser.add_argument('-S', '--service',
                        help='Name of the Fiware-service path.')

    args = parser.parse_args()
    # END PARSING ARGUMENTS
    
    try:
        with open(args.config[0]) as config_file:
            config_json = json.load(config_file)
            config = config_json["config"]
            service = ""
            if args.service:
                service = args.service
            client = FiwareClient(config["endpoint"], config["token"], service)

            if args.get_all:
                # Get entities
                print('Fetching all entities...')
                type = get_type(args)
                result = client.get_all_entities(type=type)
                print(result)
            if args.delete:
                # Delete entities
                print('Deleting entities...')
                type = get_type(args)
                result = client.delete_all_entities(type=type)
                print(result)
                print('Ok')
            if args.upload:
                # Upload data
                if check_if_file_exists(args.upload) is False:
                    print(f'Error: data file {args.upload} does not exist')
                    exit(1)
                with open(args.upload) as data_file:
                    data_json = json.load(data_file)
                    result = client.upload_entities(data_json)
                print('Ok')

    except FileNotFoundError:
        print('Error: Config file could not be loaded')
        exit(1)