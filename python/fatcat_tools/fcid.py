import base64
import uuid


def fcid2uuid(fcid: str) -> str:
    """
    Converts a fatcat identifier (base32 encoded string) to a uuid.UUID object
    """
    b = fcid.split("_")[-1].upper().encode("utf-8")
    assert len(b) == 26
    raw_bytes = base64.b32decode(b + b"======")
    return str(uuid.UUID(bytes=raw_bytes)).lower()


def uuid2fcid(s: str) -> str:
    """
    Converts a uuid.UUID object to a fatcat identifier (base32 encoded string)
    """
    raw = uuid.UUID(s).bytes
    return base64.b32encode(raw)[:26].lower().decode("utf-8")


def test_fcid() -> None:
    test_uuid = "00000000-0000-0000-3333-000000000001"
    assert test_uuid == fcid2uuid(uuid2fcid(test_uuid))
