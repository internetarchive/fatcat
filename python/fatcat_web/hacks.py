
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

