"""用本机 QGIS API 生成合成基准；不接触 MXD 或真实数据。"""

import json
from pathlib import Path
import tempfile
import xml.etree.ElementTree as ET
import zipfile

from qgis.core import (
    Qgis, QgsApplication, QgsCoordinateReferenceSystem, QgsFeature, QgsGeometry,
    QgsPointXY, QgsProject, QgsRectangle, QgsReferencedRectangle,
    QgsVectorFileWriter, QgsVectorLayer,
)
from qgis.PyQt.QtCore import QT_VERSION_STR, PYQT_VERSION_STR


app = QgsApplication([], False)
app.initQgis()
root = Path(__file__).resolve().parent


def sanitized(data):
    # 仅脱敏系统自动填入的账户名；其余 XML 保持原始结构。
    node = ET.fromstring(data)
    node.set("saveUser", "baseline")
    node.set("saveUserFull", "Synthetic baseline")
    author = node.find("projectMetadata/author")
    if author is not None:
        author.text = "Synthetic baseline"
    return ET.tostring(node, encoding="utf-8", xml_declaration=True)


def clean_zip(source, destination):
    with zipfile.ZipFile(source) as original, zipfile.ZipFile(destination, "w", zipfile.ZIP_DEFLATED) as target:
        for name in original.namelist():
            data = original.read(name)
            if name.endswith(".qgs"):
                data = sanitized(data)
            target.writestr(name, data)


with tempfile.TemporaryDirectory() as directory:
    directory = Path(directory)
    project = QgsProject()
    project.setTitle("Minimal baseline")
    project.setCrs(QgsCoordinateReferenceSystem("EPSG:4326"))
    project.viewSettings().setDefaultViewExtent(
        QgsReferencedRectangle(QgsRectangle(0, 0, 10, 10), project.crs())
    )
    for suffix in ("qgs", "qgz"):
        path = directory / ("minimal." + suffix)
        assert project.write(str(path)), project.error()
        if suffix == "qgs":
            (root / path.name).write_bytes(sanitized(path.read_bytes()))
        else:
            clean_zip(path, root / path.name)
    clean_zip(directory / "minimal_attachments.zip", root / "minimal_attachments.zip")

    # 从 API 直接提取受支持 CRS 的完整 XML，不猜测内部 srsid。
    for code in (4326, 3857):
        project.setCrs(QgsCoordinateReferenceSystem(f"EPSG:{code}"))
        path = directory / "crs.qgs"
        assert project.write(str(path))
        srs = ET.parse(path).getroot().find("projectCrs/spatialrefsys")
        (root / f"epsg{code}.xml").write_bytes(ET.tostring(srs, encoding="utf-8"))

    data = root / "data"
    data.mkdir(exist_ok=True)
    layer = QgsVectorLayer("Point?crs=EPSG:4326&field=value:integer", "Synthetic points", "memory")
    for value, point in enumerate(((1, 2), (3, 4))):
        feature = QgsFeature(layer.fields())
        feature.setGeometry(QgsGeometry.fromPointXY(QgsPointXY(*point)))
        feature.setAttributes([value])
        assert layer.dataProvider().addFeatures([feature])[0]
    options = QgsVectorFileWriter.SaveVectorOptions()
    options.driverName = "ESRI Shapefile"
    result = QgsVectorFileWriter.writeAsVectorFormatV3(
        layer, str(data / "points.shp"), project.transformContext(), options
    )
    assert result[0] == QgsVectorFileWriter.NoError, result

manifest = {
    "qgis_version": Qgis.QGIS_VERSION,
    "qgis_version_int": Qgis.QGIS_VERSION_INT,
    "qgis_dev_version": Qgis.QGIS_DEV_VERSION,
    "qt": QT_VERSION_STR,
    "pyqt": PYQT_VERSION_STR,
    "qgz_members": zipfile.ZipFile(root / "minimal.qgz").namelist(),
    "synthetic_features": [[1, 2], [3, 4]],
    "sanitization": "Only saveUser, saveUserFull and projectMetadata/author replaced",
}
(root / "baseline.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
print(json.dumps(manifest, ensure_ascii=False))
