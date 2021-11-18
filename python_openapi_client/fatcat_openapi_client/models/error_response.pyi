from typing import Any

class ErrorResponse:
    openapi_types: Any
    attribute_map: Any
    discriminator: Any
    def __init__(self, success: Any | None = ..., error: Any | None = ..., message: Any | None = ...) -> None: ...
    @property
    def success(self): ...
    @success.setter
    def success(self, success) -> None: ...
    @property
    def error(self): ...
    @error.setter
    def error(self, error) -> None: ...
    @property
    def message(self): ...
    @message.setter
    def message(self, message) -> None: ...
    def to_dict(self): ...
    def to_str(self): ...
    def __eq__(self, other): ...
    def __ne__(self, other): ...
