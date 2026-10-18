"""Minimal validator for the JSON Schema subset the adapter emits ($ref to
$defs, anyOf, type, enum, properties, required, additionalProperties,
items, maxLength, minimum, maximum, maxItems). Test helper only: it lets the
tests check that sample model outputs conform to the declared schema
without a third-party dependency."""


def validate(instance, schema, defs, path="$"):
    if "$ref" in schema:
        name = schema["$ref"].split("/")[-1]
        return validate(instance, defs[name], defs, path)
    if "anyOf" in schema:
        errors = []
        for option in schema["anyOf"]:
            try:
                validate(instance, option, defs, path)
                return
            except AssertionError as error:
                errors.append(str(error))
        raise AssertionError(f"{path}: no anyOf branch matches ({'; '.join(errors)[:300]})")
    t = schema.get("type")
    checks = {"object": dict, "array": list, "string": str, "null": type(None)}
    if t == "integer":
        assert isinstance(instance, int) and not isinstance(instance, bool), f"{path}: integer"
    elif t in checks:
        assert isinstance(instance, checks[t]), f"{path}: {t} expected, got {type(instance).__name__}"
    if "enum" in schema:
        assert instance in schema["enum"], f"{path}: {instance!r} not in {schema['enum']}"
    if isinstance(instance, str) and "maxLength" in schema:
        assert len(instance) <= schema["maxLength"], f"{path}: longer than {schema['maxLength']}"
    if isinstance(instance, int):
        if "minimum" in schema:
            assert instance >= schema["minimum"], f"{path}: below minimum"
        if "maximum" in schema:
            assert instance <= schema["maximum"], f"{path}: above maximum"
    if t == "object":
        props = schema.get("properties", {})
        missing = set(schema.get("required", [])) - set(instance)
        assert not missing, f"{path}: missing {sorted(missing)}"
        if schema.get("additionalProperties") is False:
            extra = set(instance) - set(props)
            assert not extra, f"{path}: unexpected {sorted(extra)}"
        for k, v in instance.items():
            if k in props:
                validate(v, props[k], defs, f"{path}.{k}")
    if t == "array":
        if "maxItems" in schema:
            assert len(instance) <= schema["maxItems"], f"{path}: too many items"
        for i, v in enumerate(instance):
            validate(v, schema.get("items", {}), defs, f"{path}[{i}]")
