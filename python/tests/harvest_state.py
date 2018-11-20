
import json
import pytest
import datetime
from fatcat_tools.harvest import *


def test_harvest_state():

    today = datetime.datetime.utcnow().date()

    hs = HarvestState(catchup_days=5)
    assert max(hs.to_process) < today
    assert len(hs.to_process) is 5

    for d in list(hs.to_process):
        hs.complete(d)

    assert hs.next() is None

    hs = HarvestState(
        start_date=datetime.date(2000,1,1),
        end_date=datetime.date(2000,1,3),
    )
    assert len(hs.to_process) is 3
    hs = HarvestState(
        start_date=datetime.date(2000,1,29),
        end_date=datetime.date(2000,2,2),
    )
    assert len(hs.to_process) is 5

    hs = HarvestState(catchup_days=0)
    assert hs.next() is None
    hs.enqueue_period(
        start_date=datetime.date(2000,1,1),
        end_date=datetime.date(2000,1,3),
    )
    assert len(hs.to_process) is 3
    hs.update('{"completed-date": "2000-01-02"}')
    assert len(hs.to_process) is 2
