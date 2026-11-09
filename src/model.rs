#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Syntax {
    Proto2,
    Proto3,
}

impl Syntax {
    // 返回语法字符串
    pub fn as_str(self) -> &'static str {
        match self {
            Syntax::Proto2 => "proto2",
            Syntax::Proto3 => "proto3",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scalar {
    Double,
    Float,
    Int64,
    Uint64,
    Int32,
    Fixed64,
    Fixed32,
    Bool,
    String,
    Bytes,
    Uint32,
    Sfixed32,
    Sfixed64,
    Sint32,
    Sint64,
}

impl Scalar {
    // 返回标量类型字符串
    pub fn as_str(self) -> &'static str {
        match self {
            Scalar::Double => "double",
            Scalar::Float => "float",
            Scalar::Int64 => "int64",
            Scalar::Uint64 => "uint64",
            Scalar::Int32 => "int32",
            Scalar::Fixed64 => "fixed64",
            Scalar::Fixed32 => "fixed32",
            Scalar::Bool => "bool",
            Scalar::String => "string",
            Scalar::Bytes => "bytes",
            Scalar::Uint32 => "uint32",
            Scalar::Sfixed32 => "sfixed32",
            Scalar::Sfixed64 => "sfixed64",
            Scalar::Sint32 => "sint32",
            Scalar::Sint64 => "sint64",
        }
    }
}

#[derive(Clone, Debug)]
pub enum TypeRef {
    Scalar(Scalar),
    Message(String),
    Enum(String),
    Group(String),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Label {
    Optional,
    Required,
    Repeated,
}

#[derive(Clone, Debug)]
pub struct Field {
    // 字段名
    pub name: String,
    // 字段编号
    pub number: i32,
    // 标签类型
    pub label: Label,
    // 类型引用
    pub kind: TypeRef,
    // 默认值
    pub default_value: Option<String>,
    // oneof 索引
    pub oneof_index: Option<usize>,
    // proto3 可选标记
    pub proto3_optional: bool,
    // 是否 packed
    pub packed: Option<bool>,
    // 是否废弃
    pub deprecated: bool,
    // 是否为 map
    pub is_map: bool,
    // map 键类型
    pub map_key: Option<Scalar>,
    // map 值类型
    pub map_value: Option<TypeRef>,
}

#[derive(Clone, Debug)]
pub struct Oneof {
    // oneof 名称
    pub name: String,
    // 字段列表
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug)]
pub struct EnumValue {
    // 枚举值名
    pub name: String,
    // 枚举值编号
    pub number: i32,
}

#[derive(Clone, Debug)]
pub struct Enum {
    // 枚举名
    pub name: String,
    // 枚举值列表
    pub values: Vec<EnumValue>,
    // 允许别名
    pub allow_alias: bool,
    // 是否废弃
    pub deprecated: bool,
}

#[derive(Clone, Debug)]
pub struct Message {
    // 消息名
    pub name: String,
    // 字段列表
    pub fields: Vec<Field>,
    // oneof 列表
    pub oneofs: Vec<Oneof>,
    // 嵌套消息
    pub nested_messages: Vec<Message>,
    // 嵌套枚举
    pub nested_enums: Vec<Enum>,
    // 扩展列表
    pub extensions: Vec<Extension>,
    // 是否废弃
    pub deprecated: bool,
    // 保留编号范围
    pub reserved_ranges: Vec<(i32, i32)>,
    // 保留名称
    pub reserved_names: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Extension {
    // 扩展目标类型
    pub extendee: String,
    // 扩展字段
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug)]
pub struct Method {
    // 方法名
    pub name: String,
    // 请求类型
    pub input: String,
    // 响应类型
    pub output: String,
    // 客户端流
    pub client_streaming: bool,
    // 服务端流
    pub server_streaming: bool,
}

#[derive(Clone, Debug)]
pub struct Service {
    // 服务名
    pub name: String,
    // 方法列表
    pub methods: Vec<Method>,
}

#[derive(Clone, Debug, Default)]
pub struct FileOptions {
    // java 包名
    pub java_package: Option<String>,
    // java 外部类名
    pub java_outer_classname: Option<String>,
    // java 多文件
    pub java_multiple_files: Option<bool>,
    // 优化模式
    pub optimize_for: Option<String>,
    // go 包名
    pub go_package: Option<String>,
    // 启用 arena
    pub cc_enable_arenas: Option<bool>,
    // 是否废弃
    pub deprecated: Option<bool>,
    // objc 类前缀
    pub objc_class_prefix: Option<String>,
    // csharp 命名空间
    pub csharp_namespace: Option<String>,
    // php 命名空间
    pub php_namespace: Option<String>,
    // php 元数据命名空间
    pub php_metadata_namespace: Option<String>,
    // php 类前缀
    pub php_class_prefix: Option<String>,
    // ruby 包名
    pub ruby_package: Option<String>,
    // swift 前缀
    pub swift_prefix: Option<String>,
}

#[derive(Clone, Debug)]
pub struct FileIr {
    // 文件名
    pub name: String,
    // 包名
    pub package: String,
    // 语法版本
    pub syntax: Syntax,
    // 依赖列表
    pub dependencies: Vec<String>,
    // 公共依赖
    pub public_dependencies: Vec<String>,
    // 消息列表
    pub messages: Vec<Message>,
    // 枚举列表
    pub enums: Vec<Enum>,
    // 服务列表
    pub services: Vec<Service>,
    // 扩展列表
    pub extensions: Vec<Extension>,
    // 文件选项
    pub options: FileOptions,
}
