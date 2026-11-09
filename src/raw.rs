use std::collections::BTreeMap;

use crate::model::*;

#[derive(Debug)]
enum Wire {
    Varint(u64),
    Fixed64,
    Fixed32,
    Len(Vec<u8>),
}

// 通用原始解码入口
pub fn parse_raw(name: &str, data: &[u8]) -> FileIr {
    let root = build_message("GeneratedMessage", "", data, 0);
    FileIr {
        name: name.to_string(),
        package: String::new(),
        syntax: Syntax::Proto3,
        dependencies: Vec::new(),
        public_dependencies: Vec::new(),
        messages: vec![root],
        enums: Vec::new(),
        services: Vec::new(),
        extensions: Vec::new(),
        options: FileOptions::default(),
    }
}

// 构建消息
fn build_message(name: &str, parent_fqn: &str, data: &[u8], depth: usize) -> Message {
    let fqn = if parent_fqn.is_empty() {
        name.to_string()
    } else {
        format!("{}.{}", parent_fqn, name)
    };

    let fields_raw = decode_fields(data);
    let mut by_num: BTreeMap<u64, Vec<Wire>> = BTreeMap::new();
    for (n, w) in fields_raw {
        by_num.entry(n).or_default().push(w);
    }

    let mut fields = Vec::new();
    let mut nested = Vec::new();
    for (num, vals) in by_num {
        let repeated = vals.len() > 1;
        let first = &vals[0];
        let (kind, sub_msg) = match first {
            Wire::Varint(v) => {
                if *v <= 1 {
                    (TypeRef::Scalar(Scalar::Bool), None)
                } else {
                    (TypeRef::Scalar(Scalar::Int64), None)
                }
            }
            Wire::Fixed64 => (TypeRef::Scalar(Scalar::Fixed64), None),
            Wire::Fixed32 => (TypeRef::Scalar(Scalar::Fixed32), None),
            Wire::Len(b) => {
                if depth < 6 && try_message(b.as_slice()).is_some() {
                    let subname = format!("Nested_{}", num);
                    let sub = build_message(&subname, &fqn, b.as_slice(), depth + 1);
                    (TypeRef::Message(subname), Some(sub))
                } else if is_printable_str(b.as_slice()) {
                    (TypeRef::Scalar(Scalar::String), None)
                } else {
                    (TypeRef::Scalar(Scalar::Bytes), None)
                }
            }
        };
        if let Some(sub) = sub_msg {
            nested.push(sub);
        }
        let label = if repeated {
            Label::Repeated
        } else {
            Label::Optional
        };
        fields.push(Field {
            name: format!("field_{}", num),
            number: num as i32,
            label,
            kind,
            default_value: None,
            oneof_index: None,
            proto3_optional: false,
            packed: None,
            deprecated: false,
            is_map: false,
            map_key: None,
            map_value: None,
        });
    }

    Message {
        name: name.to_string(),
        fields,
        oneofs: Vec::new(),
        nested_messages: nested,
        nested_enums: Vec::new(),
        extensions: Vec::new(),
        deprecated: false,
        reserved_ranges: Vec::new(),
        reserved_names: Vec::new(),
    }
}

// 判断可打印字符串
fn is_printable_str(b: &[u8]) -> bool {
    match std::str::from_utf8(b) {
        Ok(s) => s
            .chars()
            .all(|c| c.is_alphanumeric() || c.is_whitespace() || c.is_ascii_punctuation()),
        Err(_) => false,
    }
}

// 尝试解析为消息
fn try_message(b: &[u8]) -> Option<Vec<(u64, Wire)>> {
    let f = decode_fields(b);
    if f.is_empty() { None } else { Some(f) }
}

// 解码字段
fn decode_fields(data: &[u8]) -> Vec<(u64, Wire)> {
    let mut out = Vec::new();
    let mut pos = 0;
    while pos < data.len() {
        let (tag, p) = read_varint(data, pos);
        if p <= pos {
            break;
        }
        pos = p;
        let num = tag >> 3;
        let wt = tag & 7;
        match wt {
            0 => {
                let (v, p) = read_varint(data, pos);
                pos = p;
                out.push((num, Wire::Varint(v)));
            }
            1 => {
                if pos + 8 > data.len() {
                    break;
                }
                let mut b = [0u8; 8];
                b.copy_from_slice(&data[pos..pos + 8]);
                pos += 8;
                out.push((num, Wire::Fixed64));
            }
            5 => {
                if pos + 4 > data.len() {
                    break;
                }
                let mut b = [0u8; 4];
                b.copy_from_slice(&data[pos..pos + 4]);
                pos += 4;
                out.push((num, Wire::Fixed32));
            }
            2 => {
                let (len, p) = read_varint(data, pos);
                pos = p;
                let len = len as usize;
                if pos + len > data.len() {
                    break;
                }
                let v = data[pos..pos + len].to_vec();
                pos += len;
                out.push((num, Wire::Len(v)));
            }
            _ => break,
        }
    }
    out
}

// 读取变长整数
fn read_varint(data: &[u8], mut pos: usize) -> (u64, usize) {
    let mut result = 0u64;
    let mut shift = 0;
    while pos < data.len() {
        let b = data[pos];
        pos += 1;
        result |= ((b & 0x7f) as u64) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
    }
    (result, pos)
}
