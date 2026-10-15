"""Reproduce proposed resource-build positive from an exact qualified control."""
import hashlib
import sys
import zipfile
from pathlib import Path

source = Path(sys.argv[1])
output = Path(sys.argv[2])
assert hashlib.sha256(source.read_bytes()).hexdigest() == 'adb38ee8df9485a864135007753fb935aedf7883fe37f910109f20145f3c7be2'
with zipfile.ZipFile(source) as archive, zipfile.ZipFile(output, 'w') as target:
    for info in archive.infolist():
        content = archive.read(info.filename)
        if info.filename == '3D/Objects/overhang.stl_1.model':
            assert content.count(b'<build/>') == 1
            content = content.replace(b'<build/>', b'')
        target.writestr(info, content)
assert hashlib.sha256(output.read_bytes()).hexdigest() == '0eab5d4dea27aa756422e4a821d8d64d1b2ac544afc83dd121ac4899afa33f7a'
