"""
This snippet from: http://flask.pocoo.org/snippets/56/
"Posted by Armin Ronacher on 2011-07-14"
"""

from datetime import timedelta
from functools import update_wrapper
from typing import Any

from flask import current_app, make_response, request


def crossdomain(
    origin: Any = None,
    methods: Any = None,
    headers: Any = None,
    max_age: Any = 21600,
    attach_to_all: bool = True,
    automatic_options: bool = True,
) -> Any:
    if methods is not None:
        methods = ", ".join(sorted(x.upper() for x in methods))
    if headers is not None and not isinstance(headers, str):
        headers = ", ".join(x.upper() for x in headers)
    if not isinstance(origin, str):
        origin = ", ".join(origin)  # type: ignore
    if isinstance(max_age, timedelta):
        max_age = max_age.total_seconds()

    def get_methods() -> Any:
        if methods is not None:
            return methods

        options_resp = current_app.make_default_options_response()
        return options_resp.headers["allow"]

    def decorator(f: Any) -> Any:
        def wrapped_function(*args, **kwargs) -> Any:
            if automatic_options and request.method == "OPTIONS":
                resp = current_app.make_default_options_response()
            else:
                resp = make_response(f(*args, **kwargs))
            if not attach_to_all and request.method != "OPTIONS":
                return resp

            h = resp.headers

            h["Access-Control-Allow-Origin"] = origin  # type: ignore
            h["Access-Control-Allow-Methods"] = get_methods()
            h["Access-Control-Max-Age"] = str(max_age)
            if headers is not None:
                h["Access-Control-Allow-Headers"] = headers
            return resp

        f.provide_automatic_options = False
        return update_wrapper(wrapped_function, f)

    return decorator
