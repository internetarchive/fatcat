
"""
datacite has a REST API as well as OAI-PMH endpoint.

have about 8 million 

bulk export notes: https://github.com/datacite/datacite/issues/188

fundamentally, very similar to crossref. don't have a scrape... maybe
could/should use this script for that, and dump to JSON?
"""

from fatcat_tools.harvest.ingest_common import DoiApiHarvest

class HarvestDataciteWorker(DoiApiHarvest):

    def __init__(self, kafka_hosts, produce_topic, state_topic, contact_email,
            api_host_url="https://api.datacite.org/works",
            start_date=None, end_date=None):
        super().__init__(kafka_hosts=kafka_hosts,
                         produce_topic=produce_topic,
                         state_topic=state_topic,
                         api_host_url=api_host_url,
                         contact_email=contact_email,
                         start_date=start_date,
                         end_date=end_date)

        self.update_filter_name = "update"

