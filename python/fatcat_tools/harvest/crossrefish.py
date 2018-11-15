
"""
Notes on crossref API:

- from-index-date is the updated time
- is-update can be false, to catch only new or only old works

https://api.crossref.org/works?filter=from-index-date:2018-11-14,is-update:false&rows=2

I think the design is going to have to be a cronjob or long-running job
(with long sleeps) which publishes "success through" to a separate state
queue, as simple YYYY-MM-DD strings.

Within a day, will need to use a resumption token. Maybe should use a
crossref library... meh.

will want to have some mechanism in kafka consumer (pushing to fatcat) to group
in batches as well. maybe even pass through as batches? or just use timeouts on
iteration.
"""

from fatcat_tools.harvest.ingest_common import DoiApiHarvest

class HarvestCrossrefWorker(DoiApiHarvest):

    def __init__(self, kafka_hosts, produce_topic, state_topic, contact_email,
            api_host_url="https://api.crossref.org/works",
            is_update_filter=None,
            start_date=None, end_date=None):
        super().__init__(kafka_hosts=kafka_hosts,
                         produce_topic=produce_topic,
                         state_topic=state_topic,
                         api_host_url=api_host_url,
                         contact_email=contact_email,
                         start_date=start_date,
                         end_date=end_date)

        self.is_update_filter = is_update_filter

