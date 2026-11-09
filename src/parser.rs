use prost::Message;
use prost_types::{FileDescriptorProto, FileDescriptorSet};

pub enum Parsed {
    Set(Box<FileDescriptorSet>),
    Single(Box<FileDescriptorProto>),
}

// 解析 pb 数据
pub fn parse(data: &[u8]) -> anyhow::Result<Parsed> {
    if let Ok(set) = FileDescriptorSet::decode(data)
        && !set.file.is_empty()
    {
        return Ok(Parsed::Set(Box::new(set)));
    }
    if let Ok(fd) = FileDescriptorProto::decode(data)
        && (fd.name.is_some()
            || fd.package.is_some()
            || !fd.message_type.is_empty()
            || !fd.enum_type.is_empty())
    {
        return Ok(Parsed::Single(Box::new(fd)));
    }
    anyhow::bail!(
        "Input is not a FileDescriptorSet or FileDescriptorProto.\n\
         If it is a plain serialized message rather than a schema, retry with --raw (best-effort decoding)."
    )
}
