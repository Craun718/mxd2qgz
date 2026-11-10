import unittest
from pathlib import Path

import mxd2qgz


class PythonBindingsTest(unittest.TestCase):
    def test_streams_and_errors(self):
        sample = Path(__file__).resolve().parents[1] / "samples" / "测试底图.mxd"
        streams = dict(mxd2qgz.inspect(sample))
        self.assertIn("/Maps", streams)
        data = mxd2qgz.read_stream(sample, Path("/Maps"))
        self.assertIsInstance(data, bytes)
        self.assertEqual(len(data), streams["/Maps"])
        with self.assertRaises(OSError):
            mxd2qgz.read_stream(sample, "/missing-stream")
        with self.assertRaises(OSError):
            mxd2qgz.inspect(sample.parent / "missing.mxd")


if __name__ == "__main__":
    unittest.main()
