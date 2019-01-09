
# bunch of libraries one might want
import uuid
import datetime
import requests
import argparse

import fatcat_client
from fatcat_client import *
from fatcat_client.rest import ApiException
from fatcat_tools import *

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--debug',
        action='store_true',
        help="enable debugging interface")

    args = parser.parse_args()

    #api = 
    print("  __       _            _   _ ")
    print(" / _| __ _| |_ ___ __ _| |_| |")
    print("| |_ / _` | __/ __/ _` | __| |")
    print("|  _| (_| | || (_| (_| | |_|_|")
    print("|_|  \__,_|\__\___\__,_|\__(_)")
    print()

    admin_id = "aaaaaaaaaaaabkvkaaaaaaaaae"

    #fatcat_client.configuration.api_key['Authorization'] = 'YOUR_API_KEY'
    #fatcat_client.configuration.api_key_prefix['Authorization'] = 'Bearer'
    local_conf = fatcat_client.Configuration()
    local_conf.api_key["Authorization"] = "AgEPZGV2LmZhdGNhdC53aWtpAg4yMDE4LTEyLTMxLWRldgACJmVkaXRvcl9pZCA9IGFhYWFhYWFhYWFhYWJrdmthYWFhYWFhYWFlAAIeY3JlYXRlZCA9IDIwMTgtMTItMzFUMjE6MTU6NDdaAAAGIMWFZeZ54pH4OzNl5+U5X3p1H1rMioSuIldihuiM5XAw"
    local_conf.api_key_prefix["Authorization"] = "Bearer"
    local_conf.host = 'http://localhost:9411/v0'
    local_conf.debug = True
    local_api = fatcat_client.DefaultApi(fatcat_client.ApiClient(local_conf))

    #prod_conf = fatcat_client.Configuration()
    #local_conf.api_key["Authorization"] = "AgEPZGV2LmZhdGNhdC53aWtpAg4yMDE4LTEyLTMxLWRldgACJmVkaXRvcl9pZCA9IGFhYWFhYWFhYWFhYWJrdmthYWFhYWFhYWFlAAIeY3JlYXRlZCA9IDIwMTgtMTItMzFUMjE6MTU6NDdaAAAGIMWFZeZ54pH4OzNl5+U5X3p1H1rMioSuIldihuiM5XAw"
    #local_conf.api_key_prefix["Authorization"] = "Bearer"
    #prod_conf.host = 'https://api.fatcat.wiki/v0'
    #prod_api = fatcat_client.DefaultApi(fatcat_client.ApiClient(prod_conf))

    qa_conf = fatcat_client.Configuration()
    local_conf.api_key["Authorization"] = "AgEPZGV2LmZhdGNhdC53aWtpAg4yMDE4LTEyLTMxLWRldgACJmVkaXRvcl9pZCA9IGFhYWFhYWFhYWFhYWJrdmthYWFhYWFhYWFlAAIeY3JlYXRlZCA9IDIwMTgtMTItMzFUMjE6MTU6NDdaAAAGIMWFZeZ54pH4OzNl5+U5X3p1H1rMioSuIldihuiM5XAw"
    local_conf.api_key_prefix["Authorization"] = "Bearer"
    qa_conf.host = 'https://api.qa.fatcat.wiki/v0'
    qa_api = fatcat_client.DefaultApi(fatcat_client.ApiClient(qa_conf))
