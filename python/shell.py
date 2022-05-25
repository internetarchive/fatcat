# flake8: noqa

# bunch of libraries one might want
import argparse
import datetime
import uuid

import fatcat_openapi_client
import requests
from fatcat_openapi_client import *
from fatcat_openapi_client.rest import ApiException

from fatcat_tools import *

if __name__ == "__main__":

    # api =
    print("  __       _            _   _ ")
    print(" / _| __ _| |_ ___ __ _| |_| |")
    print("| |_ / _` | __/ __/ _` | __| |")
    print("|  _| (_| | || (_| (_| | |_|_|")
    print(r"|_|  \__,_|\__\___\__,_|\__(_)")
    print()

    admin_id = "aaaaaaaaaaaabkvkaaaaaaaaae"

    # fatcat_openapi_client.configuration.api_key['Authorization'] = 'YOUR_API_KEY'
    # fatcat_openapi_client.configuration.api_key_prefix['Authorization'] = 'Bearer'
    local_conf = fatcat_openapi_client.Configuration()
    local_conf.api_key[
        "Authorization"
    ] = "AgEPZGV2LmZhdGNhdC53aWtpAhYyMDE5MDEwMS1kZXYtZHVtbXkta2V5AAImZWRpdG9yX2lkID0gYWFhYWFhYWFhYWFhYmt2a2FhYWFhYWFhYWkAAht0aW1lID4gMjAxOS0wMS0wOVQwMDo1Nzo1MloAAAYgnroNha1hSftChtxHGTnLEmM/pY8MeQS/jBSV0UNvXug="
    local_conf.api_key_prefix["Authorization"] = "Bearer"
    local_conf.host = "http://localhost:9411/v0"
    local_conf.debug = True
    local_api = fatcat_openapi_client.DefaultApi(fatcat_openapi_client.ApiClient(local_conf))

    # prod_conf = fatcat_openapi_client.Configuration()
    # prod_conf.api_key["Authorization"] = "AgEPZGV2LmZhdGNhdC53aWtpAg4yMDE4LTEyLTMxLWRldgACJmVkaXRvcl9pZCA9IGFhYWFhYWFhYWFhYWJrdmthYWFhYWFhYWFlAAIeY3JlYXRlZCA9IDIwMTgtMTItMzFUMjE6MTU6NDdaAAAGIMWFZeZ54pH4OzNl5+U5X3p1H1rMioSuIldihuiM5XAw"
    # prod_conf.api_key_prefix["Authorization"] = "Bearer"
    # prod_conf.host = 'https://api.fatcat.wiki/v0'
    # prod_api = fatcat_openapi_client.DefaultApi(fatcat_openapi_client.ApiClient(prod_conf))

    qa_conf = fatcat_openapi_client.Configuration()
    qa_conf.api_key[
        "Authorization"
    ] = "AgEPZGV2LmZhdGNhdC53aWtpAg4yMDE4LTEyLTMxLWRldgACJmVkaXRvcl9pZCA9IGFhYWFhYWFhYWFhYWJrdmthYWFhYWFhYWFlAAIeY3JlYXRlZCA9IDIwMTgtMTItMzFUMjE6MTU6NDdaAAAGIMWFZeZ54pH4OzNl5+U5X3p1H1rMioSuIldihuiM5XAw"
    qa_conf.api_key_prefix["Authorization"] = "Bearer"
    qa_conf.host = "https://api.qa.fatcat.wiki/v0"
    qa_api = fatcat_openapi_client.DefaultApi(fatcat_openapi_client.ApiClient(qa_conf))
