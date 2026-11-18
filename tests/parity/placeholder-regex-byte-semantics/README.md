# Multiline placeholder regex fixture

`multiline-notes.3mf` is a synthetic edit of the committed [`positive.3mf`](positive.3mf)
(introduced in `0a511548cda2c524136262c7b7201190989b678b`), not a slicer export.
Only `Metadata/project_settings.config` changes: `printer_notes` becomes
`front\nCORE\nback` (literal line feeds), and `machine_start_gcode` replaces the
UTF-8 regex probe with a `=~` test on the note and a template-local multiline
copy. The local copy exercises the regex engine independently of config-block
string escaping. All other settings and archive member contents are inherited,
including the application/version metadata. The ZIP timestamp comes from
repacking with Python's `zipfile`, not from a new slicer export.

From this directory, reproduce and compare the committed bytes with Python
3.12.13 and zlib 1.3.2. The command writes a temporary file, not the fixture.

```sh
tmp=$(mktemp -d)
python3 - "$tmp/multiline-notes.3mf" <<'PY'
import json
import sys
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

with ZipFile("positive.3mf") as source, ZipFile(sys.argv[1], "w") as output:
    for name in source.namelist():
        data = source.read(name)
        if name == "Metadata/project_settings.config":
            settings = json.loads(data)
            settings["printer_notes"] = "front\nCORE\nback"
            settings["machine_start_gcode"] = (
                ';MACHINE_START\n'
                '{local fixture_notes = "front\nCORE\nback"}\n'
                ';REGEX_DOT_NEWLINE {if (fixture_notes =~ /front.*back/) and '
                '(printer_notes =~ /front.*back/)}true{else}false{endif}\n'
                'G28\n'
            )
            data = json.dumps(settings).encode()
        entry = ZipInfo(name, (2026, 11, 19, 1, 45, 40))
        output.writestr(entry, data, compress_type=ZIP_DEFLATED)
PY
cmp multiline-notes.3mf "$tmp/multiline-notes.3mf"
sha256sum "$tmp/multiline-notes.3mf"
```

Expected SHA-256: `89508b0e02823a9502aad4f5df937096c2e1c2b68bc2b39f2200288d72691a06`.
