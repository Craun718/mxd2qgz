use mxd2qgz::objects::element_body::TEXT_ELEMENT_CLASS_ID;
use mxd2qgz::objects::element_collection::{
    ELEMENT_COLLECTION_CLASS_ID, ELEMENT_REQUEST_INTERFACE_ID, ElementCollectionBody10_1,
    ElementCollectionStop10_1, ElementEntry10_1, read_element_collection_10_1,
};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::storage::object_stream::{ObjectHeader, read_header_10_1};
use mxd2qgz::stream::reader::Reader;
use std::io::ErrorKind;

/// 样本 FDO 槽位 ElementCollection 的对象键（`/Maps:0x1608` 头，体
/// `0x161c..0x1622` = 版本 4 + 计数 0）。测试仅镜像样本字节形态。
const SAMPLE_FDO_SLOT_KEY: u32 = 86;
/// 样本键 1612（`/Maps:0x18537`）非空实例首元素的类：TextElement
/// `{204034D3-F6EA-11D0-83AD-080009B996CC}`（esri.clsid 归名），用于停止
/// 点分支的样本形态镜像。
const SAMPLE_TEXT_ELEMENT_CLASS_ID: [u8; 16] = [
    0xd3, 0x34, 0x40, 0x20, 0xea, 0xf6, 0xd0, 0x11, 0x83, 0xad, 0x08, 0x00, 0x09, 0xb9, 0x96, 0xcc,
];
/// 合成元素条目的类标识（区别于真实 TextElement，用于普通停止分支）。
const SYNTHETIC_ELEMENT_CLASS_ID: [u8; 16] = [0xa5; 16];

/// AfCore String::Write 布局：u32 字节长度前缀（含终止零）+ UTF-16LE。
fn encoded(text: &str) -> Vec<u8> {
    let payload: Vec<_> = text
        .encode_utf16()
        .chain([0])
        .flat_map(u16::to_le_bytes)
        .collect();
    let mut bytes = (payload.len() as u32).to_le_bytes().to_vec();
    bytes.extend(payload);
    bytes
}

/// 追加对象条目头（16 字节类标识 + 4 字节键；回引与新对象同构）。
fn push_entry_header(bytes: &mut Vec<u8>, class_id: &[u8; 16], key: u32) {
    bytes.extend_from_slice(class_id);
    bytes.extend(key.to_le_bytes());
}

/// 追加 int32。
fn push_i32(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend(value.to_le_bytes());
}

/// 版本 short + 计数 int32 的体前缀。
fn prefix(version: u16, count: i32) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    bytes.extend(count.to_le_bytes());
    bytes
}

#[test]
fn class_id_constants_match_documented_guids() {
    // 证据：{CE8F3972-E9BE-11D1-A232-080009B6F22B}，Layer.dll 10.1.0.3035
    // GetClassID 0x78659d80 写出（.rdata 常量 0x7866db18）；内存序 = Data1-3
    // 小端 + Data4 原序，与样本 /Maps:0x1608 等 12 处对象头逐字节相同（辅
    // 助观察）。
    assert_eq!(
        ELEMENT_COLLECTION_CLASS_ID,
        [
            0x72, 0x39, 0x8f, 0xce, 0xbe, 0xe9, 0xd1, 0x11, 0xa2, 0x32, 0x08, 0x00, 0x09, 0xb6,
            0xf2, 0x2b,
        ]
    );
    // 元素循环请求 IID {9B001319-9ED7-11D0-859E-0800091A2A72}（IElement，
    // esriCarto 类型库）：与类 CLSID 不同物（请求 IID ≠ CLSID）。
    assert_eq!(
        ELEMENT_REQUEST_INTERFACE_ID,
        [
            0x19, 0x13, 0x00, 0x9b, 0xd7, 0x9e, 0xd0, 0x11, 0x85, 0x9e, 0x08, 0x00, 0x09, 0x1a,
            0x2a, 0x72,
        ]
    );
    assert_ne!(ELEMENT_COLLECTION_CLASS_ID, ELEMENT_REQUEST_INTERFACE_ID);
}

#[test]
fn reads_sample_shaped_empty_collection_in_full_chain() {
    // 样本键 86 字节镜像（辅助观察）：对象头（CLSID + 键 86）→ 体
    // `04 00` + `00 00 00 00`（版本 4、计数 0）完整消费，恰 6 字节，无尾随
    // 字段；后续 4 字节留给调用者（样本中是委派 FeatureLayer 体首 short）。
    let mut bytes = ELEMENT_COLLECTION_CLASS_ID.to_vec();
    bytes.extend(SAMPLE_FDO_SLOT_KEY.to_le_bytes());
    bytes.extend(prefix(4, 0));
    bytes.extend([0x22, 0x00, 0x24, 0x00]); // 样本 0x1622 起的字节形态。
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let header = read_header_10_1(&mut reader).unwrap();
    let ObjectHeader::Object { class_id, key } = header else {
        panic!("ElementCollection 对象头不应为 Null：{header:?}");
    };
    assert_eq!(class_id, ELEMENT_COLLECTION_CLASS_ID);
    assert_eq!(key, SAMPLE_FDO_SLOT_KEY);
    let parsed = read_element_collection_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    assert_eq!(parsed.version, 4);
    assert_eq!(parsed.count, 0);
    assert_eq!(parsed.stop, None);
    assert_eq!(
        parsed.body,
        ElementCollectionBody10_1::Interleaved { slots: Vec::new() }
    );
    assert_eq!(parsed.remaining_bytes, 4);
    assert_eq!(reader.position(), 26);
    // 空集合不登记任何键。
    assert!(table.is_empty());
}

#[test]
fn reads_version_four_interleaved_null_and_back_reference_elements() {
    // 版本 4 交错布局：每元素 [条目 + int32×3]。空标记只消耗 16 字节；回
    // 引（键已登记）消耗 20 字节无体。三个 int32 取值互异以钉住流内顺序。
    let mut table = RefTable10_1::new();
    table.register(210); // 元素 1 的回引目标。
    let mut bytes = prefix(4, 2);
    bytes.extend([0_u8; 16]); // 元素 0：空标记。
    push_i32(&mut bytes, 11);
    push_i32(&mut bytes, -12);
    push_i32(&mut bytes, 13);
    push_entry_header(&mut bytes, &SAMPLE_TEXT_ELEMENT_CLASS_ID, 210); // 元素 1：回引。
    push_i32(&mut bytes, 21);
    push_i32(&mut bytes, 22);
    push_i32(&mut bytes, -23);
    bytes.extend(b"tail");
    let body_end = bytes.len() - 4;
    let mut reader = Reader::new(&bytes);
    let parsed = read_element_collection_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    let ElementCollectionBody10_1::Interleaved { slots } = parsed.body else {
        panic!("版本 4 应为交错布局：{:?}", parsed.body);
    };
    assert_eq!(slots.len(), 2);
    assert_eq!(slots[0].entry, ElementEntry10_1::Null);
    assert_eq!(slots[0].values, [11, -12, 13]);
    assert_eq!(
        slots[1].entry,
        ElementEntry10_1::BackReference {
            class_id: SAMPLE_TEXT_ELEMENT_CLASS_ID,
            key: 210,
        }
    );
    assert_eq!(slots[1].values, [21, 22, -23]);
    assert_eq!(parsed.stop, None);
    assert_eq!(parsed.remaining_bytes, 4);
    assert_eq!(reader.position(), body_end);
    assert_eq!(reader.read_bytes(4).unwrap(), b"tail");
    // 空标记不登记键；回引不重复登记。
    assert_eq!(table.len(), 1);
    assert!(table.contains(210));
}

/// 最小完整 TextElement 体（版本 8）：EP 前缀（版本 6，VT_EMPTY）+ 空文本
/// + 布尔 int32 + 符号槽/几何槽空标记 + 版本 >2/>3 的双 int32 -1。
fn minimal_text_element_body() -> Vec<u8> {
    let mut bytes = 8_u16.to_le_bytes().to_vec();
    bytes.extend(6_u16.to_le_bytes()); // EP 版本
    bytes.extend(encoded(""));
    bytes.extend(encoded(""));
    bytes.extend([0, 0]); // VT_EMPTY
    bytes.extend(0xffff_u16.to_le_bytes());
    bytes.extend(5393_f64.to_le_bytes());
    bytes.extend(6_i32.to_le_bytes());
    bytes.extend(encoded("")); // 文本
    bytes.extend(0_i32.to_le_bytes()); // 布尔
    bytes.extend([0_u8; 16]); // ITextSymbol 槽：空标记
    bytes.extend([0_u8; 16]); // IGeometry 槽：空标记
    bytes.extend((-1_i32).to_le_bytes());
    bytes.extend((-1_i32).to_le_bytes());
    bytes
}

#[test]
fn stops_at_unknown_element_and_parses_closed_text_element() {
    // 未知元素类的新对象（接线前样本形态：首元素即 TextElement 新对象，
    // @/Maps:0x18537 键 1612——接线轮起该真实类已闭合并整体解析，见下半用
    // 例）：元素体布局未闭合且无外部长度，读取器在元素对象头起点停止，该
    // 键不进入登记表；此前元素的消费与登记保持提交。闭合的 TextElement 新
    // 对象解析其体并继续循环（后续回引元素照常消费）。
    for (class_id, key) in [(&SYNTHETIC_ELEMENT_CLASS_ID, 3001)] {
        let mut table = RefTable10_1::new();
        table.register(210);
        let mut bytes = prefix(4, 3);
        push_entry_header(&mut bytes, &SAMPLE_TEXT_ELEMENT_CLASS_ID, 210); // 元素 0：回引。
        push_i32(&mut bytes, 1);
        push_i32(&mut bytes, 2);
        push_i32(&mut bytes, 3);
        let stop_offset = bytes.len();
        push_entry_header(&mut bytes, class_id, key); // 元素 1：新对象 → 停止。
        push_i32(&mut bytes, 4);
        push_i32(&mut bytes, 5);
        push_i32(&mut bytes, 6);
        let mut reader = Reader::new(&bytes);
        let parsed = read_element_collection_10_1(&mut reader, usize::MAX, &mut table).unwrap();
        assert_eq!(parsed.count, 3, "{class_id:?}");
        let ElementCollectionBody10_1::Interleaved { slots } = &parsed.body else {
            panic!("版本 4 应为交错布局");
        };
        assert_eq!(slots.len(), 1, "停止元素不进入 slots");
        assert_eq!(slots[0].values, [1, 2, 3]);
        assert_eq!(
            parsed.stop,
            Some(ElementCollectionStop10_1::ElementBodyNotClosed {
                index: 1,
                class_id: *class_id,
                key,
            })
        );
        let message = parsed.stop.as_ref().unwrap().describe();
        assert!(message.contains("元素体布局未闭合"), "{message}");
        assert!(message.contains(&format!("键 {key}")), "{message}");
        // 游标回退到该元素对象头起点（不是元素头之后，也不是体末尾）。
        assert_eq!(reader.position(), stop_offset, "{class_id:?}");
        assert_eq!(parsed.remaining_bytes, bytes.len() - stop_offset);
        // 停止元素的键不登记；既有键保持。
        assert!(!table.contains(key), "{class_id:?}");
        assert!(table.contains(210));
        assert_eq!(table.len(), 1);
        // 停止后可重读：同一游标位置重新进入得到相同停止点（对象头仍在）。
        let again = read_element_collection_10_1(
            &mut Reader::new(&bytes[..stop_offset + 20]),
            usize::MAX,
            &mut table,
        )
        .unwrap();
        assert_eq!(again.stop, parsed.stop);
        assert_eq!(again.body, parsed.body);
    }

    // 闭合元素类（样本真实形态 TextElement）：体整体解析后循环继续。
    let mut table = RefTable10_1::new();
    table.register(210);
    let mut bytes = prefix(4, 2);
    push_entry_header(&mut bytes, &TEXT_ELEMENT_CLASS_ID, 1612); // 元素 0：TextElement 新对象
    bytes.extend(minimal_text_element_body());
    push_i32(&mut bytes, 1);
    push_i32(&mut bytes, 2);
    push_i32(&mut bytes, 3);
    push_entry_header(&mut bytes, &SAMPLE_TEXT_ELEMENT_CLASS_ID, 210); // 元素 1：回引
    push_i32(&mut bytes, 4);
    push_i32(&mut bytes, 5);
    push_i32(&mut bytes, -6);
    let mut reader = Reader::new(&bytes);
    let parsed = read_element_collection_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    assert_eq!(parsed.count, 2);
    assert_eq!(parsed.stop, None, "闭合元素体解析后循环完成");
    let ElementCollectionBody10_1::Interleaved { slots } = &parsed.body else {
        panic!("版本 4 应为交错布局");
    };
    assert_eq!(slots.len(), 2);
    let mxd2qgz::objects::element_collection::ElementEntry10_1::TextElement(te) = &slots[0].entry
    else {
        panic!("元素 0 应为 TextElement 体：{:?}", slots[0].entry);
    };
    assert!(te.stop.is_none(), "TextElement 体完整消费");
    assert_eq!(slots[0].values, [1, 2, 3]);
    assert!(matches!(
        slots[1].entry,
        mxd2qgz::objects::element_collection::ElementEntry10_1::BackReference { key: 210, .. }
    ));
    assert_eq!(slots[1].values, [4, 5, -6]);
    assert_eq!(reader.remaining(), 0);
    // 元素对象键与其体内嵌套对象的键（EP 的 CustomProperty 为 VT_EMPTY，无
    // 嵌套对象键）均登记。
    assert!(table.contains(1612));
    assert!(table.contains(210));
    assert_eq!(table.len(), 2);
}

#[test]
fn text_element_body_internal_stop_stops_collection() {
    // 嵌套停止：闭合元素类的体在其内部槽位停止（此处 TextSymbol 的符号槽
    // #2 为未知类）→ 集合以 ElementBodyIncomplete 停止，游标停在内层停止
    // 点，元素键已登记，其后的交错 int32 不可达。
    let mut bytes = prefix(4, 1);
    push_entry_header(&mut bytes, &TEXT_ELEMENT_CLASS_ID, 1612);
    bytes.extend(8_u16.to_le_bytes()); // TE 版本
    bytes.extend(6_u16.to_le_bytes()); // EP 版本
    bytes.extend(encoded(""));
    bytes.extend(encoded(""));
    bytes.extend([0, 0]); // VT_EMPTY
    bytes.extend(0xffff_u16.to_le_bytes());
    bytes.extend(5393_f64.to_le_bytes());
    bytes.extend(6_i32.to_le_bytes());
    bytes.extend(encoded("")); // 文本
    bytes.extend(0_i32.to_le_bytes()); // 布尔
    // ITextSymbol 槽：TextSymbol 新对象（键 1614），体按字段序推进到符号槽
    // #2：版本 4 + IColor #1 空标记 + int32 ×3 + byte ×2 + double ×3 +
    // int32 ×2 + IColor #2 空标记 + 字段 8 标量组 + 原样 GUID + ITextPath/
    // ISymbol #1 空标记 + 空串 + double/int32/double。
    bytes.extend(mxd2qgz::objects::element_body::TEXT_SYMBOL_CLASS_ID);
    bytes.extend(1614_u32.to_le_bytes());
    bytes.extend(4_u16.to_le_bytes());
    bytes.extend([0_u8; 16]); // IColor #1
    bytes.extend((-1_i32).to_le_bytes());
    bytes.extend(3_i32.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.push(0);
    bytes.push(0);
    for value in [0.0_f64; 3] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(13_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend([0_u8; 16]); // IColor #2
    for value in [0.0_f64; 2] {
        bytes.extend(value.to_le_bytes());
    }
    for value in [0_i32; 2] {
        bytes.extend(value.to_le_bytes());
    }
    for value in [0.0_f64; 3] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.push(1);
    bytes.extend(0_f64.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(90_f64.to_le_bytes());
    bytes.push(1);
    bytes.extend(mxd2qgz::objects::element_body::TEXT_SYMBOL_CLASS_ID); // 原样 GUID
    bytes.extend([0_u8; 16]); // ITextPath
    bytes.extend([0_u8; 16]); // ISymbol #1
    bytes.extend(encoded("")); // String
    bytes.extend(8_f64.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(1.5_f64.to_le_bytes());
    let stop_offset = bytes.len();
    bytes.extend([0x99_u8; 16]); // ISymbol #2：未知类（TextSymbol 在此停止）
    bytes.extend(b"unreachable");
    let mut reader = Reader::new(&bytes);
    let parsed = read_element_collection_10_1(&mut reader, usize::MAX, &mut RefTable10_1::new())
        .unwrap_or_else(|error| panic!("元素体内部停止应为成功返回：{error}"));
    assert_eq!(parsed.count, 1);
    let ElementCollectionStop10_1::ElementBodyIncomplete {
        index, key, reason, ..
    } = parsed.stop.clone().expect("应停止于元素体内部")
    else {
        panic!("应为 ElementBodyIncomplete：{:?}", parsed.stop);
    };
    assert_eq!(index, 0);
    assert_eq!(key, 1612);
    assert!(reason.describe().contains("ISymbol"), "{reason:?}");
    // 游标停在内层符号槽 #2 头起点（TE 头 20 之后到该处的全部字节已消费）。
    assert_eq!(reader.position(), stop_offset);
    assert_eq!(parsed.remaining_bytes, bytes.len() - stop_offset);
    // 元素对象键已随解析登记（登记先于持久化），TextSymbol 键同样登记；
    // 未知符号键不保留。
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let again = read_element_collection_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    assert_eq!(again.stop, parsed.stop, "同输入重读得到相同停止点");
    assert!(table.contains(1612), "元素键登记");
    assert!(table.contains(1614), "TextSymbol 键登记");
    assert_eq!(table.len(), 2, "未知符号键不保留");
}

#[test]
fn reads_legacy_sequential_layouts_for_versions_zero_one_two_and_three() {
    // 旧布局（FUN_7865a650，版本 0..3 同一代码路径）：先全部元素条目，再全
    // 部 int32，版本 ≥3 再全部 int32。版本 0/1/2 无第三组（原生内存写 -1，
    // 不读流）。
    let element_class = [0x11; 16];
    let mut table = RefTable10_1::new();
    table.register(90);
    let mut base = Vec::new();
    base.push(2_i32.to_le_bytes()); // 计数 2。
    base.push(0x5a_i32.to_le_bytes()); // 无关前导字节，用于错位检验（不属本布局）。
    // 正确构造：两条目 + 两组 int32（+ 版本 3 第三组）。
    let mut v2 = 2_u16.to_le_bytes().to_vec();
    v2.extend(2_i32.to_le_bytes());
    v2.extend([0_u8; 16]); // 元素 0：空标记。
    push_entry_header(&mut v2, &element_class, 90); // 元素 1：回引。
    push_i32(&mut v2, 31);
    push_i32(&mut v2, 32);
    v2.extend(b"tail");
    let mut reader = Reader::new(&v2);
    let parsed = read_element_collection_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    assert_eq!(parsed.version, 2);
    assert_eq!(parsed.count, 2);
    assert_eq!(parsed.stop, None);
    assert_eq!(
        parsed.body,
        ElementCollectionBody10_1::Sequential {
            elements: vec![
                ElementEntry10_1::Null,
                ElementEntry10_1::BackReference {
                    class_id: element_class,
                    key: 90,
                },
            ],
            group1: vec![31, 32],
            group2: None,
        }
    );
    assert_eq!(reader.position(), v2.len() - 4);
    assert_eq!(reader.read_bytes(4).unwrap(), b"tail");

    // 版本 3：第三组 int32 与第二组分离（两组取值互异钉住顺序）。
    let mut v3 = 3_u16.to_le_bytes().to_vec();
    v3.extend(2_i32.to_le_bytes());
    v3.extend([0_u8; 16]);
    push_entry_header(&mut v3, &element_class, 90);
    push_i32(&mut v3, 41);
    push_i32(&mut v3, 42);
    push_i32(&mut v3, -51);
    push_i32(&mut v3, 52);
    let mut reader = Reader::new(&v3);
    let parsed = read_element_collection_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    let ElementCollectionBody10_1::Sequential { group1, group2, .. } = &parsed.body else {
        panic!("旧布局应为顺序分离布局");
    };
    assert_eq!(group1, &vec![41, 42]);
    assert_eq!(group2, &Some(vec![-51, 52]));
    assert_eq!(reader.remaining(), 0);

    // 版本 0：与 1/2 同码路径（`版本 < 3` 分支），布局证据覆盖，接受。
    let mut v0 = 0_u16.to_le_bytes().to_vec();
    v0.extend(0_i32.to_le_bytes());
    let mut reader = Reader::new(&v0);
    let parsed = read_element_collection_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    assert_eq!(parsed.version, 0);
    assert_eq!(
        parsed.body,
        ElementCollectionBody10_1::Sequential {
            elements: Vec::new(),
            group1: Vec::new(),
            group2: None,
        }
    );
}

#[test]
fn legacy_layout_stops_at_new_element_without_consuming_int32_groups() {
    // 旧布局的元素循环中遇到新对象：游标回退到该元素对象头起点；两个
    // int32 组在停止点之后，结构性未消费。
    let mut bytes = 3_u16.to_le_bytes().to_vec();
    bytes.extend(2_i32.to_le_bytes());
    let slot_offset = bytes.len();
    push_entry_header(&mut bytes, &SYNTHETIC_ELEMENT_CLASS_ID, 1612); // 新对象（合成未闭合类）。
    // 后续字节（不会被消费）。
    push_i32(&mut bytes, 1);
    let mut reader = Reader::new(&bytes);
    let parsed =
        read_element_collection_10_1(&mut reader, usize::MAX, &mut RefTable10_1::new()).unwrap();
    assert_eq!(
        parsed.stop,
        Some(ElementCollectionStop10_1::ElementBodyNotClosed {
            index: 0,
            class_id: SYNTHETIC_ELEMENT_CLASS_ID,
            key: 1612,
        })
    );
    assert_eq!(
        parsed.body,
        ElementCollectionBody10_1::Sequential {
            elements: Vec::new(),
            group1: Vec::new(),
            group2: None,
        }
    );
    // 游标回退到元素对象头起点（版本 short + 计数之后），两个 int32 组
    // 结构性未消费。
    assert_eq!(reader.position(), slot_offset);
    assert_eq!(parsed.remaining_bytes, bytes.len() - slot_offset);
    assert!(!matches!(parsed.count, 0));
}

#[test]
fn rejects_versions_above_four_and_negative_bit_patterns() {
    // 原生有符号比较 >4 拒绝（0x80004005）；负值位模式原生落入旧加载器，
    // 本读取器一并拒绝（较原生更严，见模块文档）。仅 2 字节版本即触发。
    for version in [5_u16, 6, 100, 0x1234, 0x8000, 0xffff] {
        let bytes = version.to_le_bytes();
        let mut reader = Reader::new(&bytes);
        let first = read_element_collection_10_1(&mut reader, usize::MAX, &mut RefTable10_1::new())
            .unwrap_err();
        assert_eq!(first.kind(), ErrorKind::InvalidData, "version={version:#x}");
        let message = first.to_string();
        assert!(message.contains("版本"), "{message}");
        assert!(message.contains("offset"), "{message}");
        assert_eq!(reader.position(), 0, "version={version:#x}");
        // 失败可重复：重读得到相同错误类别且游标仍在入口。
        let second =
            read_element_collection_10_1(&mut reader, usize::MAX, &mut RefTable10_1::new())
                .unwrap_err();
        assert_eq!(first.kind(), second.kind(), "version={version:#x}");
        assert_eq!(reader.position(), 0, "version={version:#x}");
    }
}

#[test]
fn rejects_negative_count_and_enforces_min_bytes_limit_before_any_entry() {
    // 负计数：原生按 ≤0 不进循环直接成功，语义未知，本读取器拒绝（较原生
    // 更严，与 Map 后缀循环计数处理一致）。
    let bytes = prefix(4, -1);
    let mut reader = Reader::new(&bytes);
    let error = read_element_collection_10_1(&mut reader, usize::MAX, &mut RefTable10_1::new())
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    let message = error.to_string();
    assert!(message.contains("负数"), "{message}");
    assert_eq!(reader.position(), 0);

    // 限额预检在任何条目读取之前拒绝：版本 4 每元素最小 28 字节，计数 2
    // 需 56；计数 0 不受限额约束（无条目可读）。
    let mut bytes = prefix(4, 2);
    bytes.extend([0_u8; 56]);
    let mut reader = Reader::new(&bytes);
    assert!(
        read_element_collection_10_1(&mut reader, 56, &mut RefTable10_1::new()).is_ok(),
        "恰好等于最小总量的计数应通过预检"
    );
    let mut reader = Reader::new(&bytes);
    let error =
        read_element_collection_10_1(&mut reader, 55, &mut RefTable10_1::new()).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("限额"), "{}", error);
    assert_eq!(reader.position(), 0);

    // 旧布局（版本 3）每元素最小 20 + 4 = 24 字节。
    let mut legacy = prefix(3, 1);
    legacy.extend([0_u8; 24]);
    let mut reader = Reader::new(&legacy);
    assert!(
        read_element_collection_10_1(&mut reader, 24, &mut RefTable10_1::new()).is_ok(),
        "版本 3 最小总量 24 应通过预检"
    );
    let mut reader = Reader::new(&legacy);
    assert!(
        read_element_collection_10_1(&mut reader, 23, &mut RefTable10_1::new()).is_err(),
        "版本 3 最小总量 23 应被预检拒绝"
    );
}

#[test]
fn rejects_every_truncated_prefix_without_advancing_or_touching_table() {
    // 逐字节截断扫描覆盖每个字段内部：版本 short、计数 int32、元素对象头
    // 的 GUID 与键、交错的 int32、旧布局第二/第三组。任何截断都必须失败、
    // 不推进游标、不修改登记表，且重读得到相同错误类别。
    let cases: Vec<(&str, Vec<u8>)> = vec![
        (
            "版本 4：空标记元素 + 回引元素 + 各自 int32×3",
            {
                let mut bytes = prefix(4, 2);
                bytes.extend([0_u8; 16]);
                push_i32(&mut bytes, 1);
                push_i32(&mut bytes, 2);
                push_i32(&mut bytes, 3);
                push_entry_header(&mut bytes, &SAMPLE_TEXT_ELEMENT_CLASS_ID, 210);
                push_i32(&mut bytes, 4);
                push_i32(&mut bytes, 5);
                push_i32(&mut bytes, 6);
                bytes
            },
        ),
        ("版本 3 旧布局：两元素条目 + 两组 int32", {
            let mut bytes = prefix(3, 2);
            bytes.extend([0_u8; 16]);
            push_entry_header(&mut bytes, &SAMPLE_TEXT_ELEMENT_CLASS_ID, 210);
            push_i32(&mut bytes, 7);
            push_i32(&mut bytes, 8);
            push_i32(&mut bytes, 9);
            push_i32(&mut bytes, 10);
            bytes
        }),
    ];
    for (label, full) in cases {
        for end in 0..full.len() {
            let mut table = RefTable10_1::new();
            table.register(210);
            let mut reader = Reader::new(&full[..end]);
            let first = read_element_collection_10_1(&mut reader, usize::MAX, &mut table)
                .err()
                .unwrap_or_else(|| panic!("{label}: end={end} 截断前缀未返回错误"));
            assert_eq!(reader.position(), 0, "{label}: end={end}");
            let second = read_element_collection_10_1(&mut reader, usize::MAX, &mut table)
                .err()
                .unwrap_or_else(|| panic!("{label}: end={end} 重读未返回错误"));
            assert_eq!(first.kind(), second.kind(), "{label}: end={end}");
            assert_eq!(reader.position(), 0, "{label}: end={end}");
            // 失败路径不得修改登记表：原有键保留，试读新键不泄漏。
            assert_eq!(table.len(), 1, "{label}: end={end}");
            assert!(table.contains(210), "{label}: end={end}");
            assert!(!table.contains(1612), "{label}: end={end}");
        }
        // 完整输入本身必须成功（两种布局的元素 1 都是回引，无停止点）。
        let mut table = RefTable10_1::new();
        table.register(210);
        let mut reader = Reader::new(&full);
        let parsed = read_element_collection_10_1(&mut reader, usize::MAX, &mut table)
            .unwrap_or_else(|error| panic!("{label}: 完整输入失败：{error}"));
        assert_eq!(parsed.stop, None, "{label}");
        assert_eq!(reader.remaining(), 0, "{label}");
    }
}

#[test]
fn preserves_int32_bit_patterns_and_records_zero_counts_in_both_layouts() {
    // int32 仅原样保留位模式：i32::MIN/MAX、-1、0 都按位保留；两个布局的
    // 计数 0 体（仅 6 字节）都完整消费且不登记键。
    let values = [i32::MIN, -1, i32::MAX];
    let mut bytes = prefix(4, 1);
    bytes.extend([0_u8; 16]);
    for value in values {
        push_i32(&mut bytes, value);
    }
    let mut reader = Reader::new(&bytes);
    let parsed =
        read_element_collection_10_1(&mut reader, usize::MAX, &mut RefTable10_1::new()).unwrap();
    let ElementCollectionBody10_1::Interleaved { slots } = &parsed.body else {
        panic!("版本 4 应为交错布局");
    };
    assert_eq!(slots[0].values, values);
    assert_eq!(slots[0].entry, ElementEntry10_1::Null);
    assert_eq!(reader.remaining(), 0);

    for version in [0_u16, 1, 2, 3, 4] {
        let bytes = prefix(version, 0);
        let mut reader = Reader::new(&bytes);
        let parsed =
            read_element_collection_10_1(&mut reader, usize::MAX, &mut RefTable10_1::new())
                .unwrap_or_else(|error| panic!("version={version}: {error}"));
        assert_eq!(parsed.stop, None, "version={version}");
        assert_eq!(parsed.remaining_bytes, 0, "version={version}");
    }
}

#[test]
fn slot_entry_classification_depends_on_table_state() {
    // 回引与新对象在流内同构、不可局部区分（object_refs 保留的原生语义）：
    // 同一字节序列在已登记键的表下按回引用（20 字节无体），在空表下按新对
    // 象停止（游标回退到元素对象头起点、键不登记）。
    let mut bytes = prefix(4, 1);
    push_entry_header(&mut bytes, &SYNTHETIC_ELEMENT_CLASS_ID, 210);
    push_i32(&mut bytes, 5);
    push_i32(&mut bytes, 6);
    push_i32(&mut bytes, 7);
    let slot_start = 6;

    let mut table = RefTable10_1::new();
    table.register(210);
    let mut reader = Reader::new(&bytes);
    let parsed = read_element_collection_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    assert_eq!(parsed.stop, None);
    assert_eq!(reader.position(), bytes.len());
    assert_eq!(table.len(), 1);

    let mut empty = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_element_collection_10_1(&mut reader, usize::MAX, &mut empty).unwrap();
    assert!(matches!(
        parsed.stop,
        Some(ElementCollectionStop10_1::ElementBodyNotClosed {
            index: 0,
            key: 210,
            ..
        })
    ));
    assert_eq!(reader.position(), slot_start, "游标回退到元素对象头起点");
    assert!(empty.is_empty(), "停止元素的键不得登记");
}
