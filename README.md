# ss-proto

把编译后的 `.pb`（`FileDescriptorSet`）反向还原成可再次编译的 `.proto`。

## 构建

```bash
cargo build --release # 产物：target/release/pb2proto(.exe)
```

## 用法

### 1. 还原成单个 .proto

```bash
pb2proto -i proto/table.pb --single        # 默认输出 output/table.proto
pb2proto -i proto/table.pb -o table.proto  # 指定输出路径
```

### 2. 按文件拆分（保留各自 package / import）

```bash
pb2proto -i proto/proto.pb                 # 默认输出到 output/
pb2proto -i proto/proto.pb --out-dir dist/ # 指定输出目录
```

### 3. 常用选项

```bash
pb2proto -i x.pb -o x.proto --syntax proto3   # 强制语法 proto2 / proto3
pb2proto -i x.pb -o x.proto --package a.b.c   # 覆盖包名
pb2proto -i x.pb --raw -o guess.proto         # 非 descriptor 数据，启发式还原
pb2proto -i x.pb --single --stdout            # 同时打印到 stdout
pb2proto --help                               # 查看全部参数
```
