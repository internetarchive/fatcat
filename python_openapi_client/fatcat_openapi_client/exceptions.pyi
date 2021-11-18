from typing import Any

class OpenApiException(Exception): ...

class ApiTypeError(OpenApiException, TypeError):
    path_to_item: Any
    valid_classes: Any
    key_type: Any
    def __init__(self, msg: str, path_to_item: Any | None = ..., valid_classes: Any | None = ..., key_type: Any | None = ...) -> None: ...

class ApiValueError(OpenApiException, ValueError):
    path_to_item: Any
    def __init__(self, msg: str, path_to_item: Any | None = ...) -> None: ...

class ApiKeyError(OpenApiException, KeyError):
    path_to_item: Any
    def __init__(self, msg: str, path_to_item: Any | None = ...) -> None: ...

class ApiException(OpenApiException):
    status: int
    reason: str
    body: Any
    headers: Any
    def __init__(self, status: Any | None = ..., reason: Any | None = ..., http_resp: Any | None = ...) -> None: ...

def render_path(path_to_item): ...
