# 贡献指南

感谢您对SimpleBTC项目的关注！我们欢迎各种形式的贡献。

## 贡献方式

### 1. 报告Bug

在[GitHub Issues](https://github.com/GeoffreyWang1117/SimpleBTC/issues)提交Bug报告。

**包含以下信息**:
- 操作系统和版本
- Rust版本（`rustc --version`）
- 错误信息
- 复现步骤
- 相关代码片段

### 2. 提出功能建议

在Issues中提交功能请求，说明：
- 功能描述
- 使用场景
- 实现思路（可选）

### 3. 贡献代码

1. **Fork项目**
```bash
# 在GitHub上Fork
# 克隆你的Fork
git clone https://github.com/YOUR_USERNAME/SimpleBTC.git
cd SimpleBTC
```

2. **创建功能分支**
```bash
git checkout -b feature/your-feature-name
```

3. **编写代码**
   - 遵循Rust风格指南
   - 添加测试
   - 更新文档

4. **提交Pull Request**
```bash
git add .
git commit -m "Add: your feature description"
git push origin feature/your-feature-name
```

在GitHub上创建Pull Request。

### 4. 改进文档

文档同样重要！
- 修正错误
- 添加示例
- 改进说明
- 翻译文档

## 开发指南

### 代码风格

```bash
# 格式化代码
cargo fmt

# 检查lint
cargo clippy
```

### 测试

```bash
# 运行所有测试
cargo test

# 添加测试
#[cfg(test)]
mod tests {
    #[test]
    fn test_something() {
        // ...
    }
}
```

### 文档注释

```rust
/// 函数的简短描述
///
/// 详细说明...
///
/// # 参数
/// * `param1` - 参数说明
///
/// # 返回值
/// 返回值说明
///
/// # 示例
/// \```
/// let result = function(arg);
/// \```
pub fn function(param1: Type) -> ReturnType {
    // ...
}
```

## Pull Request检查清单

提交PR前确保：

- [ ] 代码通过`cargo fmt`格式化
- [ ] 代码通过`cargo clippy`检查
- [ ] 所有测试通过`cargo test`
- [ ] 添加了必要的测试
- [ ] 更新了相关文档
- [ ] 提交信息清晰明确

## 社区准则

- 友好尊重
- 建设性讨论
- 欢迎新手
- 专注技术

## License

贡献的代码将采用项目的MIT许可证。

---

感谢您的贡献！🎉
