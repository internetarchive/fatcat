from typing import Tuple, Union

from flask import Response as FlaskResponse
from werkzeug.wrappers import Response as WerkzeugResponse

# type to represent any of the used/plausible return types
AnyResponse = Union[
    FlaskResponse,
    str,
    Tuple[str, int],
    WerkzeugResponse,
    Tuple[FlaskResponse, int],
    Tuple[WerkzeugResponse, int],
]
