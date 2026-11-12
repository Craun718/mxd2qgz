"""API 语义核对；这不是在 QGIS 中目视检查。"""

import json
from pathlib import Path
import sys
from qgis.core import Qgis, QgsApplication, QgsProject

app = QgsApplication([], False)
app.initQgis()
fixtures = Path(__file__).resolve().parent
output = Path(sys.argv[1])


def rectangle(rect):
    return [rect.xMinimum(), rect.yMinimum(), rect.xMaximum(), rect.yMaximum()]


def load(path):
    project = QgsProject()
    assert project.read(str(path)), project.error()
    return project


for suffix in ("qgs", "qgz"):
    baseline = load(fixtures / f"minimal.{suffix}")
    assert baseline.crs().authid() == "EPSG:4326"
    assert rectangle(baseline.viewSettings().defaultViewExtent()) == [0, 0, 10, 10]
    assert not baseline.mapLayers()

reports = []
for suffix in ("qgs", "qgz"):
    project = load(output / f"constructed.{suffix}")
    assert project.title() == "构造 <&> \"'\n工程"
    assert project.crs().authid() == "EPSG:3857"
    assert project.readEntry("mxd2qgz", "frameName")[0] == "主框"
    assert project.readEntry("mxd2qgz", "sourceCrs")[0] == "EPSG:3857"
    assert rectangle(project.viewSettings().defaultViewExtent()) == [0, 0, 10, 10]
    assert project.viewSettings().defaultViewExtent().crs().authid() == "EPSG:3857"
    tree = project.layerTreeRoot()
    assert len(tree.children()) == 2
    group, broken = tree.children()
    assert group.name() == "组 <&> \"'\t\n"
    assert not group.itemVisibilityChecked()
    leaf, subgroup = group.children()
    assert leaf.itemVisibilityChecked() and not leaf.isVisible()
    assert subgroup.itemVisibilityChecked()
    assert not subgroup.children()[0].itemVisibilityChecked()
    assert broken.itemVisibilityChecked()
    layers = project.mapLayers()
    assert set(layers) == {"mxd2qgz_frame_0_0_0", "mxd2qgz_frame_0_0_1_0", "mxd2qgz_frame_0_1"}, list(layers)
    for layer_id in ("mxd2qgz_frame_0_0_0", "mxd2qgz_frame_0_0_1_0"):
        layer = layers[layer_id]
        assert layer.isValid(), layer.source()
        assert layer.featureCount() == 2
        assert layer.crs().authid() == "EPSG:4326"
        assert layer.customProperty("mxd2qgz/sourceCrs") == "EPSG:4326"
        assert layer.customProperty("mxd2qgz/sourceExtent") == "0,0,10,10"
        assert Path(layer.source()) == fixtures / "data/points.shp"
        points = sorted((f.geometry().asPoint().x(), f.geometry().asPoint().y()) for f in layer.getFeatures())
        assert points == [(1, 2), (3, 4)]
    assert not layers["mxd2qgz_frame_0_1"].isValid()
    reports.append({"format": suffix, "layers": len(layers), "valid_layers": 2, "broken_layers": 1, "project_crs": project.crs().authid(), "canvas_extent": rectangle(project.viewSettings().defaultViewExtent()), "loaded_layer_extent": rectangle(layers["mxd2qgz_frame_0_0_0"].extent())})

unknown = load(output / "unknown.qgs")
assert not unknown.crs().isValid()
assert unknown.viewSettings().defaultViewExtent().isNull()
assert unknown.readEntry("mxd2qgz", "sourceCrs")[0] == "unknown"
assert unknown.write(str(output / "unknown-resaved.qgz"))
assert load(output / "unknown-resaved.qgz").readEntry("mxd2qgz", "sourceCrs")[0] == "unknown"
empty = load(output / "empty.qgz")
assert not empty.crs().isValid() and not empty.mapLayers()
print(json.dumps({"qgis_version": Qgis.QGIS_VERSION, "qgis_dev_version": Qgis.QGIS_DEV_VERSION, "checks": reports, "unknown_crs_is_invalid": True, "visual_check": "未验证"}, ensure_ascii=False))
