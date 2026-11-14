"""Author ordinary bed targets over the synthetic initial-bed fixture."""
from copy import copy
import hashlib
import json
from pathlib import Path
from zipfile import ZipFile

root = Path(__file__).resolve().parent
source = root.parent / "m140-initial-bed/unused-hotter-filament.3mf"
assert hashlib.sha256(source.read_bytes()).hexdigest() == (
    "893689e2ea06b9f6de4704fc21ca6743618ac483e4e02d00eb1e50419431f6b0"
)
manifest = {}
with ZipFile(source) as archive:
    for label, target in [("unchanged-used", "63"), ("cooler-used", "43")]:
        with ZipFile(root / f"{label}.3mf", "w") as output:
            for info in archive.infolist():
                content = archive.read(info.filename)
                if info.filename == "Metadata/project_settings.config":
                    settings = json.loads(content)
                    settings["cool_plate_temp"] = ["88", target]
                    content = json.dumps(settings, indent=4).encode()
                output.writestr(copy(info), content)
        manifest[label] = hashlib.sha256((root / f"{label}.3mf").read_bytes()).hexdigest()
(root / "sha256.json").write_text(json.dumps(manifest, indent=2) + "\n")
