//! FunctionRasterDB.dll 10.1.0.3035 的共用持久化入口类表。
//! 各类 CLSID 及虚表依据见 docs/mxd-10.1-image-server-rules.md。

pub(super) const fn class_id(a: u32, b: u16, c: u16, d: u64) -> [u8; 16] {
    let [a0, a1, a2, a3] = a.to_le_bytes();
    let [b0, b1] = b.to_le_bytes();
    let [c0, c1] = c.to_le_bytes();
    let [d0, d1, d2, d3, d4, d5, d6, d7] = d.to_be_bytes();
    [
        a0, a1, a2, a3, b0, b1, c0, c1, d0, d1, d2, d3, d4, d5, d6, d7,
    ]
}

pub(super) const GEOMETRIC_FUNCTION: [u8; 16] =
    class_id(0x1013a267, 0x0e0e, 0x428f, 0x88165551f450d711);
pub(super) const CACHED_RASTER_FUNCTION: [u8; 16] =
    class_id(0xaca03769, 0x2d4e, 0x484d, 0x8e141bc4acb17a59);
pub(super) const RGB32_FUNCTION: [u8; 16] =
    class_id(0x95cd2068, 0x42a4, 0x43ee, 0xbf087b2e562a9b47);
pub(super) const MERGE_RASTERS_FUNCTION: [u8; 16] =
    class_id(0x96e60d70, 0x6bf9, 0x4486, 0xb9edc41181259f71);
pub(super) const MERGE_RASTERS_ARGUMENTS: [u8; 16] =
    class_id(0x9a388281, 0xb984, 0x4bd3, 0xb799348d01a05605);
pub(super) const RASTER_INFO_FUNCTION: [u8; 16] =
    class_id(0xd44242bf, 0x18e8, 0x4670, 0x9f0a132f0aa58b17);
pub(super) const RASTER_ITEM_FUNCTION: [u8; 16] =
    class_id(0x31e67a19, 0xed23, 0x4825, 0xbf1e6a7d7b0ef190);
pub(super) const RASTER_FUNCTION_COLLECTION: [u8; 16] =
    class_id(0x1f120f8f, 0x17aa, 0x4b8e, 0x8cd86007e0dedb80);
pub(super) const MOSAIC_FUNCTION: [u8; 16] =
    class_id(0x7406b10e, 0x6009, 0x4366, 0x85ff5a067b54b2ad);
pub(super) const RASTER_FUNCTION_TEMPLATE: [u8; 16] =
    class_id(0x0b67bd89, 0x4783, 0x4a6f, 0xbe52143db3660c11);
pub(super) const RASTER_FUNCTION_VARIABLE: [u8; 16] =
    class_id(0x1799317a, 0xc5ab, 0x4aee, 0x8c9996a5cf1ba0e6);
pub(super) const LOCAL_FUNCTION: [u8; 16] =
    class_id(0x3ecb15f2, 0x6443, 0x443e, 0x9228659328507fee);
pub(super) const LOCAL_FUNCTION_ARGUMENTS: [u8; 16] =
    class_id(0x3b4a5e5b, 0x4ace, 0x4219, 0xbaa9289b5c431e74);

// RasterLocalOp.dll 的公开类将 IPersistStream 转发给相应的局部运算实现。
pub(super) const LOCAL_FUNCTION_ALIASES: [[u8; 16]; 3] = [
    class_id(0x3dedf3d2, 0x61fc, 0x4f9d, 0x9eae5a3f5358308b), // LocalFunction
    class_id(0x2e60049f, 0xfe7f, 0x4bcd, 0x9b34b6ac978b1e10), // MathFunction
    class_id(0xc75db7fb, 0x1bed, 0x46e3, 0x8f9617d24d8a03ce), // ConditionalFunction
];
pub(super) const LOCAL_ARGUMENT_ALIASES: [[u8; 16]; 3] = [
    class_id(0x19da76f9, 0xe143, 0x41b3, 0x955c38033a7189a8), // LocalFunctionArguments
    class_id(0xde07b947, 0xbd0b, 0x4851, 0xaa703826b4983240), // MathFunctionArguments
    class_id(0x35591e68, 0x2dc3, 0x4630, 0x8bba48eaea56f834), // ConditionalFunctionArguments
];

pub(super) const FUNCTIONS: [[u8; 16]; 45] = [
    class_id(0x04e0f604, 0x76f5, 0x4a7a, 0xa1c457f54b9cc4af), // ArithmeticFunction
    class_id(0xd8730557, 0x5a60, 0x4b99, 0x9e6343f56cc1259a), // AspectFunction
    class_id(0xd2455fd9, 0x9a43, 0x4630, 0xb9285cc89e52bf4c), // BandArithmeticFunction
    class_id(0xdff0d3b8, 0xfd42, 0x4250, 0xbe65842828dc8e99), // ClipFunction
    class_id(0x5a5b350d, 0xc1f0, 0x4e50, 0x983c358dbc9c40b5), // ColormapFunction
    class_id(0x6a8d5aae, 0x8c8e, 0x4b67, 0x89b225e1e393557f), // ColormapToRGBFunction
    class_id(0x13b16685, 0xca39, 0x478e, 0x9b4c02f12a73552e), // ColorspaceConversionFunction
    class_id(0x7f9b9795, 0x6d98, 0x4f8b, 0xaeb871fc8fe92e49), // ComplexFunction
    class_id(0xb314f443, 0x3d35, 0x4e34, 0xa389c0822a187395), // CompositeBandFunction
    class_id(0x1423a723, 0x2865, 0x4b58, 0x98ad01928daf4ce8), // ConstantFunction
    class_id(0xe5986c48, 0x1f76, 0x4914, 0xba6e5ea65231d7df), // ContrastBrightnessFunction
    class_id(0x35dbacdc, 0x18e0, 0x4c5b, 0xa087c6c5bb692289), // ConvolutionFunction
    class_id(0xc081e7af, 0x4878, 0x4845, 0x8ae687d42a4435c0), // DodgingFunction
    class_id(0xfc9a46ce, 0xe198, 0x48db, 0xaec208486d19e605), // ExcludedAreaFunction
    class_id(0xf9fe7bb2, 0x8910, 0x4b5a, 0xbcb5fe990bd3e517), // ExtractBandFunction
    class_id(0x4861cc7d, 0x5d08, 0x4af2, 0xb69b49c8e90ca84b), // FootprintFunction
    class_id(0x060cc78f, 0xd651, 0x4df3, 0xaa4795e1d0d1d311), // GrayscaleFunction
    class_id(0x77b291d1, 0x4581, 0x4b4f, 0xa802791de874bc89), // HillshadeFunction
    class_id(0xdc71d132, 0xe02e, 0x4dff, 0x852b7e1f10024d9b), // IdentityFunction
    class_id(0xa19473ea, 0xd62d, 0x4610, 0x833ff2b3d4bcb8e4), // KauthThomasFunction
    class_id(0xa5a0861f, 0x1b50, 0x420c, 0x934b53685732f07c), // MaskFunction
    class_id(0x33c422d8, 0xb2e5, 0x4ba3, 0xbdd32d6000e49723), // MosaicRastersFunction
    class_id(0x1f3ac8e8, 0x3011, 0x4b87, 0x8d4a8150a81fdc58), // NDVIFunction
    class_id(0x47775611, 0x00dd, 0x44dd, 0xb771a50932ebcd40), // PanSharpenGramSchmidtFunction
    class_id(0xb54e8861, 0xfcf0, 0x4642, 0x908a360e1bf5ea79), // PansharpeningFunction
    class_id(0x8d9f5211, 0xf745, 0x45f8, 0x94cf205152875d87), // RadarCalibrationFunction
    class_id(0x5e5c1828, 0xecd1, 0x4620, 0x9ca1970d2aa450de), // ReflectanceFunction
    class_id(0x19156a32, 0xb9a0, 0x4ddf, 0xb79f61d74f367f93), // RemapFunction
    class_id(0x23d7cd5b, 0x224b, 0x42f3, 0x945d13adfbb71b9f), // ReprojectFunction
    class_id(0xee26add9, 0xf748, 0x45cd, 0xb0ae795979090a79), // RGB32ToRGBFunction
    class_id(0x465187c5, 0xc8d4, 0x462b, 0xaf852b375dd7d7e6), // RGBToColormapFunction
    class_id(0x0bdc2211, 0x71c1, 0x48da, 0x9cf66903730e302d), // SAIFunction
    class_id(0x838e348b, 0x33c6, 0x4853, 0x8e512e056884b669), // SelectionFunction
    class_id(0xecb818c5, 0xbc4e, 0x4359, 0xb40ad2ef9e6717c2), // ShadedReliefFunction
    class_id(0xbaf9a2de, 0x8b1a, 0x4dc5, 0xaeae138e2a7bcf3d), // SlopeFunction
    class_id(0x1c11c9e1, 0x25f1, 0x4042, 0xb6e8e9953d85392f), // SpeckleFunction
    class_id(0x35eead6e, 0xd973, 0x41ac, 0x837df16f6023e256), // SpectralConversionFunction
    class_id(0xdd760853, 0x675f, 0x4770, 0xb678ad1600e6db5e), // StatisticsFunction
    class_id(0xa3da7ce4, 0x0af9, 0x4130, 0x954cea3294f549ed), // TableFunction
    class_id(0x12038de4, 0x0d34, 0x4772, 0x9f71a1ea4d1ea83a), // TrendFunction
    // 3DToRasterFunctions.dll、SARasterFunction.dll 和 RasterRenderer.dll 的同形入口。
    class_id(0x06a78a67, 0xba8a, 0x4592, 0x83e55dd4362f6636), // LasDatasetToRasterFunction
    class_id(0x9889d8fb, 0x93f6, 0x49b1, 0xb352d8e2b331c797), // LasToRasterFunction
    class_id(0xd4d3a0bb, 0x3770, 0x40ef, 0xacf1bb3c1c98a38e), // TerrainToRasterFunction
    class_id(0x5803078b, 0xaf97, 0x4388, 0x9b9e7657ae76a310), // MLClassifyFunction
    class_id(0xeed089c5, 0x98f5, 0x496d, 0x903cce3f2a1956dc), // RenderedRasterFunction
];

pub(super) const RASTER_FUNCTION_TEMPLATE_ARGUMENTS: [u8; 16] =
    class_id(0xe7e9fa4f, 0xfcc6, 0x483d, 0x9450ff38b6a2c315);
pub(super) const RASTER_INFO_FUNCTION_ARGUMENTS: [u8; 16] =
    class_id(0x877bd868, 0xee05, 0x4493, 0xb586bb71681f0665);
pub(super) const RASTER_ITEM_FUNCTION_ARGUMENTS: [u8; 16] =
    class_id(0xa2a534d3, 0xd721, 0x4c8c, 0xbfaf98c6d835daf2);

pub(super) const ARGUMENTS: [[u8; 16]; 44] = [
    class_id(0x082dc874, 0x8acc, 0x4fd8, 0xa1d1aeaf73ca899a), // ReflectanceFunctionArguments
    class_id(0x073c8688, 0x949b, 0x4483, 0x9f348289ed1d1072), // ArithmeticFunctionArguments
    class_id(0x6f19d624, 0x24fe, 0x4eac, 0xbf1fa117928de308), // BandArithmeticFunctionArguments
    class_id(0x4d463957, 0x3b81, 0x46ba, 0xaa8c4654de7d94e5), // CachedRasterFunctionArguments
    class_id(0xf72be5e4, 0xfa3f, 0x4626, 0x95cf4a78102f5f6e), // ClipFunctionArguments
    class_id(0xcaaf0682, 0x8040, 0x43d1, 0xae2189f9684df9b4), // ColormapFunctionArguments
    class_id(0x069b72cd, 0xd41c, 0x4c1f, 0x878476609830ab8a), // ColorspaceConversionFunctionArguments
    class_id(0x4036254e, 0x3cc0, 0x460e, 0x98be8e9fb309f0a3), // ConstantFunctionArguments
    class_id(0x634cbd5d, 0xbb9d, 0x4372, 0xa908230fb8ba5355), // ContrastBrightnessFunctionArguments
    class_id(0x78f85973, 0xafd4, 0x4dfa, 0x9a65246e40ea1ef9), // ConvolutionFunctionArguments
    class_id(0xb4c026c5, 0xa517, 0x460f, 0x88fd96afc79bdd4f), // DodgingFunctionArguments
    class_id(0x32248838, 0x455a, 0x42ca, 0x98cdf938e502f495), // ExcludedAreaFunctionArguments
    class_id(0xe35f696d, 0xdc8d, 0x4274, 0xb5a67a15fb0cf512), // ExtractBandFunctionArguments
    class_id(0x80ff48a6, 0xe886, 0x4f70, 0x8022abf1b9275d31), // FootprintFunctionArguments
    class_id(0x562f6ff5, 0x2351, 0x42d1, 0xb464ba03de92e7ef), // GeometricFunctionArguments
    class_id(0xcd0e53bf, 0x6d15, 0x48b6, 0xa7c43971365813aa), // GrayscaleFunctionArguments
    class_id(0x80186241, 0x15df, 0x4c3a, 0x94da50fa3f73fcb2), // HillshadeFunctionArguments
    class_id(0x350d8526, 0x9fad, 0x4842, 0xa77db2a6ec143dbf), // MaskFunctionArguments
    class_id(0x62c333ed, 0xa776, 0x4a2e, 0x80621959c70d2704), // MosaicFunctionArguments
    class_id(0x94f10296, 0x5868, 0x4b2d, 0x9596e17bdd077e25), // MosaicRastersFunctionArguments
    class_id(0xf7871ddd, 0x26cd, 0x40d1, 0xb86ef5308e3828e2), // NDVIFunctionArguments
    class_id(0x767381ff, 0x3ecc, 0x4685, 0x89974f2e0f523cec), // PanSharpenGramSchmidtFunctionArguments
    class_id(0x09bf8db7, 0x1a82, 0x4ad2, 0xa366aee762487f75), // PansharpeningFunctionArguments
    class_id(0xa78dfa71, 0xd2d3, 0x4766, 0x85ddc70c17dbe6b3), // RadarCalibrationFunctionArguments
    class_id(0x86acb7d5, 0xa0e7, 0x4345, 0x9fbf8bf59d5c87b2), // RemapFunctionArguments
    class_id(0x453fbf56, 0xca41, 0x410d, 0x92cffb727cfdc1d1), // ReprojectFunctionArguments
    class_id(0x314e643b, 0x44a2, 0x43ad, 0x93746f4eeade11e9), // SAIFunctionArguments
    class_id(0xa7340aa2, 0x4b37, 0x4c71, 0x9cac2c47d8ac1e09), // SelectionFunctionArguments
    class_id(0xc0c4f4db, 0x7f2a, 0x4050, 0xa1e289cd7868c43a), // ShadedReliefFunctionArguments
    class_id(0xc39388b1, 0x0eb2, 0x4b50, 0x97f647beb2c60d86), // SlopeFunctionArguments
    class_id(0xe0a62507, 0x3ab0, 0x42e9, 0xb3a0a58cd1ff0d6e), // SpeckleFunctionArguments
    class_id(0xf5e78815, 0x251e, 0x4008, 0x9c5bf63abdb3799e), // SpectralConversionFunctionArguments
    class_id(0xc26aac72, 0x5d86, 0x482e, 0x96d4d56986ebf09c), // StatisticsFunctionArguments
    class_id(0xd8261e56, 0x178a, 0x4a52, 0x915b788f9393951c), // StretchFunctionArguments
    class_id(0x4c912271, 0xd13a, 0x43bf, 0xadde722902de115f), // TableFunctionArguments
    class_id(0x954e5bb5, 0x2892, 0x49cb, 0x9bf6033f5a78705b), // TrendFunctionArguments
    RASTER_FUNCTION_TEMPLATE_ARGUMENTS,
    RASTER_INFO_FUNCTION_ARGUMENTS,
    // 外部模块中直接委派 BasicFunctionArguments 的类。
    class_id(0x67876aa0, 0x40eb, 0x470a, 0xabc9036085756c53), // LasDatasetToRasterFunctionArguments
    class_id(0xfb8e343b, 0x8014, 0x4ce9, 0xb457e72389d2b339), // LasToRasterFunctionArguments
    class_id(0x8bf576ca, 0x96c8, 0x4106, 0x9020a2d78d221dc4), // TerrainToRasterFunctionArguments
    class_id(0xc1570587, 0xe9ea, 0x43cc, 0xb938545b1ef289fe), // MLClassifyFunctionArguments
    class_id(0xa6e9ca22, 0x4c2b, 0x4b4c, 0xaea5da37566c3130), // RenderedRasterFunctionArguments
    RASTER_ITEM_FUNCTION_ARGUMENTS,
];
