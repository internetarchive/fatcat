import re

from fatcat_openapi_client import WebcaptureEntity

STRIP_EXTLINK_XML_RE = re.compile(r"<ext-link.*xlink:type=\"simple\">")


def strip_extlink_xml(unstr: str) -> str:
    unstr = unstr.replace("</ext-link>", "")
    unstr = STRIP_EXTLINK_XML_RE.sub("", unstr)
    return unstr


def test_strip_extlink_xml() -> None:
    assert strip_extlink_xml("asdf") == "asdf"
    assert (
        strip_extlink_xml(
            """LOCKSS (2014) Available: <ext-link xmlns:xlink="http://www.w3.org/1999/xlink" ext-link-type="uri" xlink:href="http://lockss.org/" xlink:type="simple">http://lockss.org/</ext-link>. Accessed: 2014 November 1."""
        )
        == """LOCKSS (2014) Available: http://lockss.org/. Accessed: 2014 November 1."""
    )


def wayback_suffix(entity: WebcaptureEntity) -> str:
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
