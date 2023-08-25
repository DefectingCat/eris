## Eris

图文模板批量处理

### Install

```bash
cargo install --path .
```

### Usage

下载所有图文模板压缩包到一个目录中 `./templates`。

```bash
eris -d ./templates
```

### Others

```bash
❯ ./target/release/eris -h
HTML Template processer

Usage: eris [OPTIONS]

Options:
  -d, --directory <DIRECTORY>  Target directory [default: .]
  -h, --help                   Print help
  -V, --version                Print version
```

### Build from source

```bash
cargo build --relase
```

For windows

```bash
cargo build --target=x86_64-pc-windows-gnu --release
```
