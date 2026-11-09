use std::collections::{BTreeMap, HashMap};

use prost_types::{
    DescriptorProto, EnumDescriptorProto, EnumValueDescriptorProto, FieldDescriptorProto,
    FileDescriptorProto, FileDescriptorSet, FileOptions as PbFileOptions, MessageOptions,
    ServiceDescriptorProto, file_options,
};

use crate::model::*;

// 转换描述符集合
pub fn convert_set(set: &FileDescriptorSet) -> Vec<FileIr> {
    set.file.iter().map(convert_file).collect()
}

// 转换单个文件描述符
pub fn convert_file(fd: &FileDescriptorProto) -> FileIr {
    let package = fd.package.clone().unwrap_or_default();
    let syntax = if fd.syntax.as_deref() == Some("proto3") {
        Syntax::Proto3
    } else {
        Syntax::Proto2
    };

    let mut map_entries: HashMap<String, (Scalar, TypeRef)> = HashMap::new();
    for m in &fd.message_type {
        collect_map_entries(m, &package, &mut map_entries);
    }

    let messages = fd
        .message_type
        .iter()
        .map(|m| convert_message(m, &package, &map_entries))
        .collect();
    let enums = fd.enum_type.iter().map(convert_enum).collect();
    let services = fd.service.iter().map(convert_service).collect();
    let extensions = group_extensions(&fd.extension);

    let mut public_deps = Vec::new();
    for &i in &fd.public_dependency {
        if let Some(d) = fd.dependency.get(i as usize) {
            public_deps.push(d.clone());
        }
    }

    FileIr {
        name: fd.name.clone().unwrap_or_default(),
        package,
        syntax,
        dependencies: fd.dependency.clone(),
        public_dependencies: public_deps,
        messages,
        enums,
        services,
        extensions,
        options: convert_file_options(fd.options.as_ref()),
    }
}

// 拼接全限定名
fn join_fqn(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_string()
    } else {
        format!("{}.{}", parent, name)
    }
}

// 从编号映射标量类型
fn scalar_from_i32(t: i32) -> Option<Scalar> {
    Some(match t {
        1 => Scalar::Double,
        2 => Scalar::Float,
        3 => Scalar::Int64,
        4 => Scalar::Uint64,
        5 => Scalar::Int32,
        6 => Scalar::Fixed64,
        7 => Scalar::Fixed32,
        8 => Scalar::Bool,
        9 => Scalar::String,
        12 => Scalar::Bytes,
        13 => Scalar::Uint32,
        15 => Scalar::Sfixed32,
        16 => Scalar::Sfixed64,
        17 => Scalar::Sint32,
        18 => Scalar::Sint64,
        _ => return None,
    })
}

// 推断字段类型引用
fn type_ref(f: &FieldDescriptorProto) -> TypeRef {
    let t = f.r#type.unwrap_or(0);
    if let Some(s) = scalar_from_i32(t) {
        return TypeRef::Scalar(s);
    }
    let tn = f
        .type_name
        .clone()
        .unwrap_or_default()
        .trim_start_matches('.')
        .to_string();
    match t {
        10 => TypeRef::Group(tn),
        11 => TypeRef::Message(tn),
        14 => TypeRef::Enum(tn),
        _ => TypeRef::Scalar(Scalar::Bytes),
    }
}

// 转换字段
fn convert_field(f: &FieldDescriptorProto) -> Field {
    let label = match f.label.unwrap_or(1) {
        2 => Label::Required,
        3 => Label::Repeated,
        _ => Label::Optional,
    };
    let kind = type_ref(f);
    Field {
        name: f.name.clone().unwrap_or_default(),
        number: f.number.unwrap_or(0),
        label,
        kind,
        default_value: f.default_value.clone(),
        oneof_index: f.oneof_index.map(|x| x as usize),
        proto3_optional: f.proto3_optional.unwrap_or(false),
        packed: f.options.as_ref().and_then(|o| o.packed),
        deprecated: f
            .options
            .as_ref()
            .and_then(|o| o.deprecated)
            .unwrap_or(false),
        is_map: false,
        map_key: None,
        map_value: None,
    }
}

// 收集 map 条目
fn collect_map_entries(
    msg: &DescriptorProto,
    parent_fqn: &str,
    reg: &mut HashMap<String, (Scalar, TypeRef)>,
) {
    let fqn = join_fqn(parent_fqn, msg.name.as_deref().unwrap_or(""));
    if msg
        .options
        .as_ref()
        .and_then(|o| o.map_entry)
        .unwrap_or(false)
    {
        let key = msg.field.iter().find(|f| f.number == Some(1));
        let value = msg.field.iter().find(|f| f.number == Some(2));
        if let (Some(k), Some(v)) = (key, value) {
            let ks = scalar_from_i32(k.r#type.unwrap_or(0)).unwrap_or(Scalar::String);
            let vt = type_ref(v);
            reg.insert(fqn.clone(), (ks, vt));
        }
    }
    for n in &msg.nested_type {
        collect_map_entries(n, &fqn, reg);
    }
}

// 转换消息
fn convert_message(
    msg: &DescriptorProto,
    parent_fqn: &str,
    map_entries: &HashMap<String, (Scalar, TypeRef)>,
) -> Message {
    let fqn = join_fqn(parent_fqn, msg.name.as_deref().unwrap_or(""));

    let mut all: Vec<Field> = msg.field.iter().map(convert_field).collect();

    for f in &mut all {
        if f.label == Label::Repeated
            && let TypeRef::Message(tn) = &f.kind
            && let Some((k, v)) = map_entries.get(tn)
        {
            f.is_map = true;
            f.map_key = Some(*k);
            f.map_value = Some(v.clone());
        }
    }

    let mut oneofs = Vec::new();
    for (idx, decl) in msg.oneof_decl.iter().enumerate() {
        let fields: Vec<Field> = all
            .iter()
            .filter(|f| f.oneof_index == Some(idx) && !f.proto3_optional)
            .cloned()
            .collect();
        if !fields.is_empty() {
            oneofs.push(Oneof {
                name: decl.name.clone().unwrap_or_default(),
                fields,
            });
        }
    }
    let standalone: Vec<Field> = all
        .into_iter()
        .filter(|f| f.oneof_index.is_none() || f.proto3_optional)
        .collect();

    let nested_messages = msg
        .nested_type
        .iter()
        .filter(|m| {
            !m.options
                .as_ref()
                .and_then(|o| o.map_entry)
                .unwrap_or(false)
        })
        .map(|m| convert_message(m, &fqn, map_entries))
        .collect();
    let nested_enums = msg.enum_type.iter().map(convert_enum).collect();
    let extensions = group_extensions(&msg.extension);
    let reserved_ranges = msg
        .reserved_range
        .iter()
        .map(|r| (r.start.unwrap_or(0), r.end.unwrap_or(0)))
        .collect();

    Message {
        name: msg.name.clone().unwrap_or_default(),
        fields: standalone,
        oneofs,
        nested_messages,
        nested_enums,
        extensions,
        deprecated: msg
            .options
            .as_ref()
            .and_then(|o: &MessageOptions| o.deprecated)
            .unwrap_or(false),
        reserved_ranges,
        reserved_names: msg.reserved_name.clone(),
    }
}

// 转换枚举
fn convert_enum(e: &EnumDescriptorProto) -> Enum {
    let values = e
        .value
        .iter()
        .map(|v: &EnumValueDescriptorProto| EnumValue {
            name: v.name.clone().unwrap_or_default(),
            number: v.number.unwrap_or(0),
        })
        .collect();
    Enum {
        name: e.name.clone().unwrap_or_default(),
        values,
        allow_alias: e
            .options
            .as_ref()
            .and_then(|o| o.allow_alias)
            .unwrap_or(false),
        deprecated: e
            .options
            .as_ref()
            .and_then(|o| o.deprecated)
            .unwrap_or(false),
    }
}

// 转换服务
fn convert_service(s: &ServiceDescriptorProto) -> Service {
    let methods = s
        .method
        .iter()
        .map(|m| Method {
            name: m.name.clone().unwrap_or_default(),
            input: m
                .input_type
                .clone()
                .unwrap_or_default()
                .trim_start_matches('.')
                .to_string(),
            output: m
                .output_type
                .clone()
                .unwrap_or_default()
                .trim_start_matches('.')
                .to_string(),
            client_streaming: m.client_streaming.unwrap_or(false),
            server_streaming: m.server_streaming.unwrap_or(false),
        })
        .collect();
    Service {
        name: s.name.clone().unwrap_or_default(),
        methods,
    }
}

// 分组扩展字段
fn group_extensions(fields: &[FieldDescriptorProto]) -> Vec<Extension> {
    let mut map: BTreeMap<String, Vec<Field>> = BTreeMap::new();
    for f in fields {
        let ext = f
            .extendee
            .clone()
            .unwrap_or_default()
            .trim_start_matches('.')
            .to_string();
        map.entry(ext).or_default().push(convert_field(f));
    }
    map.into_iter()
        .map(|(extendee, fields)| Extension { extendee, fields })
        .collect()
}

// 转换文件选项
fn convert_file_options(o: Option<&PbFileOptions>) -> crate::model::FileOptions {
    let mut fo = crate::model::FileOptions::default();
    if let Some(o) = o {
        fo.java_package = o.java_package.clone();
        fo.java_outer_classname = o.java_outer_classname.clone();
        fo.java_multiple_files = o.java_multiple_files;
        fo.optimize_for = o.optimize_for.map(|v| match v {
            x if x == file_options::OptimizeMode::CodeSize as i32 => "CODE_SIZE".to_string(),
            x if x == file_options::OptimizeMode::LiteRuntime as i32 => "LITE_RUNTIME".to_string(),
            _ => "SPEED".to_string(),
        });
        fo.go_package = o.go_package.clone();
        fo.cc_enable_arenas = o.cc_enable_arenas;
        fo.deprecated = o.deprecated;
        fo.objc_class_prefix = o.objc_class_prefix.clone();
        fo.csharp_namespace = o.csharp_namespace.clone();
        fo.php_namespace = o.php_namespace.clone();
        fo.php_metadata_namespace = o.php_metadata_namespace.clone();
        fo.php_class_prefix = o.php_class_prefix.clone();
        fo.ruby_package = o.ruby_package.clone();
        fo.swift_prefix = o.swift_prefix.clone();
    }
    fo
}
