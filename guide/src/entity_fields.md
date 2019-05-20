# Common Entity Fields

All entities have:

- `extra`: free-form JSON metadata

The "extra" field is an "escape hatch" to include extra fields not in the
regular schema. It is intended to enable gradual evolution of the schema, as
well as accommodating niche or field-specific content. Reasonable care should
be taken with this extra metadata: don't include large text or binary fields,
hundreds of fields, duplicate metadata, etc.

