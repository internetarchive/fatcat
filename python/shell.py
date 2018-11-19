
# bunch of libraries one might want
import uuid
import datetime
import requests
import argparse

import fatcat_client
from fatcat_client import *
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

    local_conf = fatcat_client.Configuration()
    local_conf.host = 'http://localhost:9411/v0'
    local_api = fatcat_client.DefaultApi(fatcat_client.ApiClient(local_conf))

    prod_conf = fatcat_client.Configuration()
    prod_conf.host = 'https://api.fatcat.wiki/v0'
    prod_api = fatcat_client.DefaultApi(fatcat_client.ApiClient(prod_conf))

    qa_conf = fatcat_client.Configuration()
    qa_conf.host = 'https://api.qa.fatcat.wiki/v0'
    qa_api = fatcat_client.DefaultApi(fatcat_client.ApiClient(qa_conf))
