# Implementation of the necessary parts of the Fiware NGSI v2 API
# Specs: https://fiware-ges.github.io/orion/api/v2/stable/

from dataclasses import dataclass
import requests
import json
from datetime import datetime
from dateutil import parser

@dataclass
class MeasurementRequest():
    urn: str
    name: str

@dataclass
class MeasurementResult():
    urn: str
    name: str
    value: str
    timestamp: datetime

class FiwareClient():
    """
    Class that implements the Fiware NGSI v2 API.
    """
    def __init__(self, endpoint, token, service=None) -> None:
        """
        Constructor accepts two parameters:
        @param endpoint: URL of API endpoint.
        @param token: access token.
        @param service: service path.
        """
        self.endpoint = endpoint
        self.token = token
        self.service = service
        
    def send_get(self, request):
        """
        Helper method that sends a GET request with the authorization token
        """
        return requests.get(request, headers = {"X-Auth-Token": self.token, "fiware-service": self.service})
    
    def send_post(self, request, body):
        """
        Helper method that sends a POST request with the authorization token
        """
        return requests.post(request, headers = {"X-Auth-Token": self.token, "fiware-service": self.service}, json = body)

    def get_all_entities(self, type=None):
        """
        Gets all entities of a given type (if type provided)
        """
        call_endpoint = f"{self.endpoint}/entities?"
        if type is not None:
            call_endpoint += "type=" + type + "&limit=1000&offset="
        else:
            call_endpoint += "limit=1000&offset="
        offset = 0
        call_endpoint_base = call_endpoint
        final_response = []
        while True:
            call_endpoint = call_endpoint_base + str(offset)
            response = self.send_get(call_endpoint)
            response_json = response.json()
            if len(response_json) == 0:
                break
            for value in response_json:
                final_response.append(value)
            offset += 1000
        return final_response
    
    def delete_all_entities(self, type=None):
        """
        Gets all entities of a given type (if type provided)
        """
        call_endpoint = f"{self.endpoint}/op/update"
        # First query all entities to get their IDs
        result = self.get_all_entities(type)
        ids = []
        for entity in result:
            ids.append(
                {
                    "id": entity["id"]
                })
        payload = {
            "actionType": "delete",
            "entities": ids
        }
        print(payload)
        response = self.send_post(call_endpoint, body = payload)
        return response
    
    def upload_entities(self, entities, key_values = False):
        """
        Uploads entities to the Fiware instance.
        """
        call_endpoint = f"{self.endpoint}/op/update"
        if key_values:
            call_endpoint += "?options=keyValues"
        payload = {
            "actionType": "append_strict",
            "entities": entities
        }
        response = self.send_post(call_endpoint, body = payload)
        print(payload)
        return response
    
    def query_entity(self, measurement_request: MeasurementRequest):
        """
        Queries latest data for an entity.
        @param urn: the unique identifier for the entity.
        """
        call_endpoint = f"{self.endpoint}/entities/{measurement_request.urn}"
        response = self.send_get(call_endpoint)

        response_json = response.json()
        result = None
        #print(json.dumps(response_json, indent=4, sort_keys=True))
        if measurement_request.name in response_json:
            ts = None
            if "TimeInstant" in response_json:
                ts_str = response_json["TimeInstant"]["value"]
                ts = parser.parse(ts_str)
            result = MeasurementResult(measurement_request.urn, measurement_request.name, response_json[measurement_request.name]["value"], ts)

        return result

