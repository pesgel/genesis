# genesis

### 规范

#### 安装 VSCode 插件

- crates: Rust 包管理
- Even Better TOML: TOML 文件支持
- Better Comments: 优化注释显示
- Error Lens: 错误提示优化
- GitLens: Git 增强
- Github Copilot: 代码提示
- indent-rainbow: 缩进显示优化
- Prettier - Code formatter: 代码格式化
- REST client: REST API 调试
- rust-analyzer: Rust 语言支持
- Rust Test lens: Rust 测试支持
- Rust Test Explorer: Rust 测试概览
- TODO Highlight: TODO 高亮
- vscode-icons: 图标优化
- YAML: YAML 文件支持

#### 安装 pre-commit

pre-commit 是一个代码检查工具，可以在提交代码前进行代码检查。

```bash
pipx install pre-commit
```

安装成功后运行 `pre-commit install` 即可。

#### 安装 Cargo deny

Cargo deny 是一个 Cargo 插件，可以用于检查依赖的安全性。

```bash
cargo install --locked cargo-deny
```

#### 安装 typos

typos 是一个拼写检查工具。

```bash
cargo install typos-cli
```

#### 安装 git cliff

git cliff 是一个生成 changelog 的工具。

```bash
cargo install git-cliff
```
使用
```bash
git cliff -o CHANGELOG.md
```

#### 安装 cargo nextest

cargo nextest 是一个 Rust 增强测试工具。

```bash
cargo install cargo-nextest --locked
```

使用:
```bash
cargo nextest run --nocapture
# 运行特定test
cargo nextest run -p genesis-ssh test_remote_client
```
- --nocapture 运行所有测试并显示测试输出
- --package 运行特定模块

#### 安装 cargo install cargo-tarpaulin

cargo-tarpaulin 测试覆盖率工具
安装:
```bash
cargo install cargo-tarpaulin
```
使用:
```bash
cargo tarpaulin --out Html
```
#### GIT提交规范
- feat: 增加新功能（feature）。
- fix: 修复 bug。
- docs: 仅仅修改文档，如 README 或注释。
- style: 不影响代码运行的更改（代码格式、空格、缺少分号等）。
- refactor: 代码重构，既不是修复 bug，也不是添加新功能。
- perf: 提高性能的代码更改。
- test: 增加或修改测试用例。
- chore: 其他非业务相关的修改（构建过程、依赖管理等）。
- revert: 回滚之前的提交。
