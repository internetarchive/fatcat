from enum import Enum
from typing import List, Optional

from fatcat_openapi_client import ReleaseEntity
from pydantic import BaseModel


class AccessType(str, Enum):
    """describes type of access URL"""

    wayback = "wayback"
    ia_file = "ia_file"
    ia_microfilm = "ia_microfilm"
    repository = "repository"
    openlibrary = "openlibrary"
    wikipedia = "wikipedia"


class AccessOption(BaseModel):

    access_type: AccessType

    # note: for `target_url` refs, would do a CDX lookup and this URL would be
    # a valid/HTTP-200 web.archive.org capture URL
    access_url: str

    # application/pdf, text/html, etc
    # blank for landing pages
    mimetype: Optional[str]

    size_bytes: Optional[int]
    thumbnail_url: Optional[str]


def release_access_options(release: ReleaseEntity) -> List[AccessOption]:
    """
    Extracts access options from a release.

    TODO: proper implementation and filtering, instead of just returning first
    option found
    """
    options: List[AccessOption] = []
    for f in release.files or []:
        thumbnail_url = None
        if f.mimetype == "application/pdf" and f.sha1 and f.urls:
            # NOTE: scholar.archive.org does an actual database check before
            # generating these URLs, but we skip that for speed
            thumbnail_url = f"https://blobs.fatcat.wiki/thumbnail/pdf/{f.sha1[0:2]}/{f.sha1[2:4]}/{f.sha1}.180px.jpg"
        for u in f.urls or []:
            if "://web.archive.org/" in u.url:
                return [
                    AccessOption(
                        access_type="wayback",
                        access_url=u.url,
                        mimetype=f.mimetype,
                        size_bytes=f.size,
                        thumbnail_url=thumbnail_url,
                    )
                ]
            elif "://archive.org/" in u.url:
                return [
                    AccessOption(
                        access_type="ia_file",
                        access_url=u.url,
                        mimetype=f.mimetype,
                        size_bytes=f.size,
                        thumbnail_url=thumbnail_url,
                    )
                ]
    return options
