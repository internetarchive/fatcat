
Top missing OA journals by `container_id`:

    POST _xpack/sql?format=txt
    {
    "query": "SELECT container_id, count(*) from fatcat_release WHERE preservation = 'none' AND is_oa = true GROUP BY container_id ORDER BY count(*) DESC LIMIT 20"
    }

