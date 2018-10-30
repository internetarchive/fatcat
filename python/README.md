
## Python Web Interface and API Client Library

Use `pipenv` (which you can install with `pip`).

    pipenv run fatcat_webface.py

Run tests:

    pipenv install --dev
    pipenv run pytest

    # for coverage:
    pipenv run pytest --cov --cov-report html

Regeneate swagger-codegen python client library (requires docker):

    ./codegen_python_client.sh
