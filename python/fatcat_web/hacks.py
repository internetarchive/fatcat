
import re

STRIP_EXTLINK_XML_RE = re.compile(r"<ext-link.*xlink:type=\"simple\">")

def strip_extlink_xml(unstr):
    unstr = unstr.replace("</ext-link>", "")
    unstr = STRIP_EXTLINK_XML_RE.sub("", unstr)
    return unstr

def test_strip_extlink_xml():
    assert strip_extlink_xml("asdf") == "asdf"
    assert strip_extlink_xml("""LOCKSS (2014) Available: <ext-link xmlns:xlink="http://www.w3.org/1999/xlink" ext-link-type="uri" xlink:href="http://lockss.org/" xlink:type="simple">http://lockss.org/</ext-link>. Accessed: 2014 November 1.""") == \
    """LOCKSS (2014) Available: http://lockss.org/. Accessed: 2014 November 1."""

def wayback_suffix(entity):
    """
    Takes a webcapture entity and returns a suffix to be appended to wayback URLs
    """
    ret = ""
    if entity.original_url:
        if entity.timestamp:
            ret = entity.timestamp.strftime("%Y%m%d%H%M%S/")
        else:
            ret = "*/"
        ret += entity.original_url
    return ret

def camp_sha1_path(sha1):
    assert len(sha1) == 40
    if sha1[0:2] > 'a9' or (sha1[0:2] == 'a9' and sha1[2:4] >= '48'):
        shard = 'pdf3'
    elif sha1[0:2] < '53' or (sha1[0:2] == '53' and sha1[2:4] < 'db'):
        shard = 'pdf1'
    else:
        shard = 'pdf2'
    return '/{}/pdf/{}/{}/{}.pdf'.format(
        shard, sha1[0:2], sha1[2:4], sha1)

def test_camp_sha1_path():
    assert camp_sha1_path('00000088bbc15a03ab89d8da6c356bf25aea9519') == '/pdf1/pdf/00/00/00000088bbc15a03ab89d8da6c356bf25aea9519.pdf'
    assert camp_sha1_path('53da34b7df640a5065e347ea99e40302154225d5') == '/pdf1/pdf/53/da/53da34b7df640a5065e347ea99e40302154225d5.pdf'
    assert camp_sha1_path('53db34b7df640a5065e347ea99e40302154225d5') == '/pdf2/pdf/53/db/53db34b7df640a5065e347ea99e40302154225d5.pdf'
    assert camp_sha1_path('a947d6e2e55dbe649cfde780991f3989e235843d') == '/pdf2/pdf/a9/47/a947d6e2e55dbe649cfde780991f3989e235843d.pdf'
    assert camp_sha1_path('a948d6e2e55dbe649cfde780991f3989e235843d') == '/pdf3/pdf/a9/48/a948d6e2e55dbe649cfde780991f3989e235843d.pdf'
    assert camp_sha1_path('ff48d6e2e55dbe649cfde780991f3989e235843d') == '/pdf3/pdf/ff/48/ff48d6e2e55dbe649cfde780991f3989e235843d.pdf'

def get_camp_pdf_path(release):
    """
    Assumes the release has been expanded (includes file entities).

    Returns a full local URL to a PDF of the file if one should be available;
    otherwise returns None.
    """
    for f in release.files:
        for u in f.urls:
            if '://web.archive.org/' in u.url:
                return camp_sha1_path(f.sha1)
    return None
