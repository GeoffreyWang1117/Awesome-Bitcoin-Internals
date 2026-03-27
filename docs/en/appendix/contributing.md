# Contributing Guide

Thank you for your interest in the SimpleBTC project! All forms of contribution are welcome.

## Ways to Contribute

### 1. Report Bugs

Submit a bug report on [GitHub Issues](https://github.com/GeoffreyWang1117/SimpleBTC/issues).

**Include the following information**:
- Operating system and version
- Rust version (`rustc --version`)
- Error message
- Steps to reproduce
- Relevant code snippets

### 2. Suggest Features

Submit a feature request in Issues, explaining:
- Feature description
- Use case
- Implementation ideas (optional)

### 3. Contribute Code

1. **Fork the project**
```bash
# Fork on GitHub
# Clone your fork
git clone https://github.com/YOUR_USERNAME/SimpleBTC.git
cd SimpleBTC
```

2. **Create a feature branch**
```bash
git checkout -b feature/your-feature-name
```

3. **Write code**
   - Follow the Rust style guide
   - Add tests
   - Update documentation

4. **Submit a Pull Request**
```bash
git add .
git commit -m "Add: your feature description"
git push origin feature/your-feature-name
```

Create a Pull Request on GitHub.

### 4. Improve Documentation

Documentation matters just as much as code!
- Fix errors
- Add examples
- Improve explanations
- Translate documentation

## Development Guide

### Code Style

```bash
# Format code
cargo fmt

# Run lint checks
cargo clippy
```

### Testing

```bash
# Run all tests
cargo test

# Add tests
#[cfg(test)]
mod tests {
    #[test]
    fn test_something() {
        // ...
    }
}
```

### Documentation Comments

```rust
/// Brief description of the function
///
/// Detailed explanation...
///
/// # Parameters
/// * `param1` - parameter description
///
/// # Return Value
/// Description of the return value
///
/// # Examples
/// \```
/// let result = function(arg);
/// \```
pub fn function(param1: Type) -> ReturnType {
    // ...
}
```

## Pull Request Checklist

Before submitting a PR, ensure:

- [ ] Code is formatted with `cargo fmt`
- [ ] Code passes `cargo clippy` checks
- [ ] All tests pass with `cargo test`
- [ ] Necessary tests have been added
- [ ] Relevant documentation has been updated
- [ ] Commit messages are clear and descriptive

## Community Guidelines

- Be friendly and respectful
- Keep discussions constructive
- Welcome newcomers
- Stay focused on technical topics

## License

Code contributed to this project will be released under the project's MIT license.

---

Thank you for your contribution!
