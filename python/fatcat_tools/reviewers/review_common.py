import datetime
import subprocess
import time
from collections import Counter
from typing import Any, List, Optional

import fatcat_openapi_client

"""
checks should return:
- status: pass, fail, warning, skipped
- description

annotation extra should include:
- useragent
- git_rev
- submit timestamp

can apply to individual entity revs, or to entire editgroup

reviewbot takes an editgroup (object) and returns an annotation (object)
"""


class CheckResult:
    status = None
    description = None
    ident = None
    rev = None
    check_type = None

    def __init__(self, status, check_type=None, description=None, **kwargs):
        self.status = status
        self.check_type = check_type
        self.description = description
        self.ident = kwargs.get("ident")
        self.rev = kwargs.get("rev")

    def __repr__(self):
        return str(self.__dict__)


class EditCheck:

    scope: List[Any] = []
    name: Optional[str] = None

    def check_editgroup(self, editgroup):
        raise NotImplementedError

    def check_container(self, edit, entity):
        raise NotImplementedError

    def check_creator(self, edit, entity):
        raise NotImplementedError

    def check_file(self, edit, entity):
        raise NotImplementedError

    def check_fileset(self, edit, entity):
        raise NotImplementedError

    def check_webcapture(self, edit, entity):
        raise NotImplementedError

    def check_release(self, edit, entity):
        raise NotImplementedError

    def check_work(self, edit, work):
        raise NotImplementedError


class ReviewBot:
    def __init__(self, api, verbose=False, **kwargs):

        self.api = api
        self.checks = []
        self.verbose = verbose
        self.extra = kwargs.get("extra", dict())
        self.extra["git_rev"] = self.extra.get(
            "git_rev", subprocess.check_output(["git", "describe", "--always"]).strip()
        ).decode("utf-8")
        self.extra["agent"] = self.extra.get("agent", "fatcat_tools.ReviewBot")
        self.poll_interval = kwargs.get("poll_interval", 10.0)

    def run_single(self, editgroup_id, annotate=True):
        eg = self.api.get_editgroup(editgroup_id)
        annotation = self.review_editgroup(eg)
        if annotate:
            self.api.create_editgroup_annotation(eg.editgroup_id, annotation)
        return annotation

    def run(self, since=None):
        if since is None:
            since = datetime.datetime.utcnow()
        while True:
            # XXX: better isoformat conversion?
            eg_list = self.api.get_editgroups_reviewable(
                since=since.isoformat()[:19] + "Z", limit=100
            )
            if not eg_list:
                print("Sleeping {} seconds...".format(self.poll_interval))
                time.sleep(self.poll_interval)
                continue
            for eg in eg_list:
                # TODO: fetch annotations to ensure we haven't already annotated
                annotation = self.review_editgroup(eg)
                print(
                    "Reviewed {} disposition:{}".format(
                        eg.editgroup_id, annotation.extra["disposition"]
                    )
                )
                self.api.create_editgroup_annotation(eg.editgroup_id, annotation)
                since = eg.submitted
            # to prevent busy loops (TODO: needs review/rethink; multiple
            # editgroups in the same second)
            since = since + datetime.timedelta(seconds=1)

    def review_editgroup(self, editgroup):
        results = self.run_checks(editgroup)
        result_counts = self.result_counts(results)
        disposition = self.disposition(results)
        if disposition == "accept":
            comment = "This editgroup looks great! All checks passed."
        elif disposition == "revise":
            comment = "Some issues were found, and changes or close review are recommended before accepting."
        elif disposition == "reject":
            comment = "Serious issues were found; this editgroup should **not** be accepted."
        else:
            raise ValueError

        for (status, title) in (("fail", "Failed check"), ("warning", "Warnings")):
            if result_counts[status] > 0:
                comment += "\n\n### {} ({}):\n".format(status, result_counts[status])
            for result in results:
                if result.status == status and result.check_type == "editgroup":
                    comment += "\n- {description}".format(description=result.description)
                if result.status == status and result.check_type != "editgroup":
                    comment += "\n- {check_type} [{rev}](/{entity_type}/rev/{rev}): {description}".format(
                        check_type=result.check_type,
                        rev=result.rev,
                        entity_type=result.check_type,
                        description=result.description,
                    )

        extra = self.extra.copy()
        extra.update(
            {
                "disposition": disposition,
                "submit_timestamp": editgroup.submitted.isoformat(),
                "checks": [check.name for check in self.checks],
                "result_counts": dict(result_counts),
            }
        )
        annotation = fatcat_openapi_client.EditgroupAnnotation(
            comment_markdown=comment,
            editgroup_id=editgroup.editgroup_id,
            extra=extra,
        )
        return annotation

    def result_counts(self, results):
        counts = Counter()
        for result in results:
            counts["total"] += 1
            counts[result.status] += 1
        return counts

    def disposition(self, results):
        """
        Returns one of: accept, revise, reject
        """
        raise NotImplementedError

    def run_checks(self, editgroup):

        results = []

        # any full-editgroup checks
        for check in self.checks:
            if "editgroup" in check.scope:
                result = check.check_editgroup(editgroup)
                if self.verbose:
                    print(result)
                results.append(result)

        if not editgroup.edits:
            entity_edits = {}
        else:
            entity_edits = {
                "container": editgroup.edits.containers,
                "creator": editgroup.edits.creators,
                "file": editgroup.edits.files,
                "fileset": editgroup.edits.filesets,
                "webcapture": editgroup.edits.webcaptures,
                "release": editgroup.edits.releases,
                "work": editgroup.edits.works,
            }

        # entity-specific checks
        for entity_type, edits in entity_edits.items():
            for edit in edits:
                entity = None
                for check in self.checks:
                    if entity_type in check.scope:
                        # hack-y python munging
                        get_method = getattr(self.api, "get_{}_rev".format(entity_type))
                        check_method = getattr(check, "check_{}".format(entity_type))
                        entity = get_method(self.api, edit.rev)
                        result = check_method(check, edit, entity)
                        result.rev = edit.rev
                        result.ident = edit.ident
                        if self.verbose:
                            print(result)
                        results.append(result)

        return results


class DummyCheck(EditCheck):

    scope = ["editgroup", "work"]
    name = "DummyCheck"

    def check_editgroup(self, editgroup):
        return CheckResult(
            "pass",
            "editgroup",
            "every edit is precious, thanks [editor {editor_id}](/editor/{editor_id})!".format(
                editor_id=editgroup.editor_id
            ),
        )

    def check_work(self, entity, edit):
        return CheckResult("pass", "work", "this work edit is beautiful")


class DummyReviewBot(ReviewBot):
    """
    This bot reviews everything and always passes.
    """

    def __init__(self, api, **kwargs):
        super().__init__(api, **kwargs)
        self.checks = [DummyCheck()]

    def disposition(self, results):
        return "accept"
