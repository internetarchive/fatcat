
import pytest
import fatcat.api_client
from fixtures import *


def test_rich_app_fixture(rich_app):
    app = rich_app

    assert ChangelogEntry.query.count() == 1

    for cls in (WorkIdent, WorkRev, WorkEdit,
                ContainerIdent, ContainerRev, ContainerEdit,
                CreatorIdent, CreatorRev, CreatorEdit,
                FileIdent, FileRev, FileEdit):
        assert cls.query.count() == 1
    for cls in (ReleaseIdent, ReleaseRev, ReleaseEdit):
        assert cls.query.count() == 2

    for cls in (WorkIdent,
                ContainerIdent,
                CreatorIdent,
                FileIdent):
        assert cls.query.filter(cls.is_live==True).count() == 1
    assert ReleaseIdent.query.filter(ReleaseIdent.is_live==True).count() == 2

    # test that editor's active edit group is now invalid
    editor = Editor.query.first()
    assert editor.active_edit_group == None
