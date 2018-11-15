
import base64
import uuid

def fcid2uuid(s):
    """
    Converts a fatcat identifier (base32 encoded string) to a uuid.UUID object
    """
    s = s.split('_')[-1].upper().encode('utf-8')
    assert len(s) == 26
    raw = base64.b32decode(s + b"======")
    return str(uuid.UUID(bytes=raw)).lower()

def uuid2fcid(s):
    """
    Converts a uuid.UUID object to a fatcat identifier (base32 encoded string)
    """
    raw = uuid.UUID(s).bytes
    return base64.b32encode(raw)[:26].lower().decode('utf-8')

def test_fcid():
    test_uuid = '00000000-0000-0000-3333-000000000001'
    assert test_uuid == fcid2uuid(uuid2fcid(test_uuid))
