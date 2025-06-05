# Contributing to fetchttp

Thank you for considering contributing to fetchttp! This document provides guidelines for contributing to the project.

## 🚀 Getting Started

### Prerequisites

- **Rust**: Install the latest stable Rust toolchain from [rustup.rs](https://rustup.rs/)
- **Git**: For version control
- **Editor**: Any editor with Rust support (VS Code with rust-analyzer recommended)

### Setting Up the Development Environment

1. **Fork and Clone**
   ```bash
   git clone https://github.com/MuntasirSZN/fetchttp.git
   cd fetchttp
   ```

2. **Install Dependencies**
   ```bash
   cargo build
   ```

3. **Run Tests**
   ```bash
   cargo test
   ```

4. **Check Formatting and Linting**
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   ```

## 📋 Development Workflow

### Branch Naming

- `feature/description` - New features
- `fix/description` - Bug fixes  
- `docs/description` - Documentation changes
- `refactor/description` - Code refactoring
- `test/description` - Test improvements

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Test additions/modifications
- `chore`: Maintenance tasks
MuntasirSZN*
```bash
feat(headers): add support for custom header validation
fix(client): handle connection timeout properly
docs(readme): update installation instructions
test(integration): add tests for abort functionality
```

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test headers

# Run integration tests
cargo test --test integration

# Run with output
cargo test -- --nocapture

# Run performance tests
cargo test --test performance --release
```

### Writing Tests

1. **Unit Tests**: Place in the same file as the code being tested
2. **Integration Tests**: Place in `tests/` directory
3. **Benchmarks**: Place in `benches/` directory

**Test Structure:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = "test";
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_async_function() {
        // Test async functions
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### Test Guidelines

- **Test Names**: Use descriptive names that explain what is being tested
- **Test Isolation**: Each test should be independent
- **Test Data**: Use realistic test data when possible
- **Error Cases**: Test both success and failure scenarios
- **Edge Cases**: Test boundary conditions and edge cases

## 📝 Documentation

### Code Documentation

- **Public APIs**: Must have comprehensive documentation with examples
- **Modules**: Include module-level documentation explaining purpose
- **Examples**: Provide working examples in doc comments
- **Error Conditions**: Document when functions can fail

**Documentation Format:**
```rust
/// Brief description of what the function does.
/// 
/// Longer description with more details about behavior,
/// edge cases, and usage patterns.
/// 
/// # Arguments
/// 
/// * `param1` - Description of first parameter
/// * `param2` - Description of second parameter
/// 
/// # Returns
/// 
/// Description of what is returned, including error conditions.
/// 
/// # Errors
/// 
/// This function will return an error if:
/// - Condition 1
/// - Condition 2
/// 
/// # Examples
/// 
/// ```rust
/// use fetchttp::*;
/// 
/// let result = function_name("example").unwrap();
/// assert_eq!(result, "expected");
/// ```
pub fn function_name(param1: &str, param2: u32) -> Result<String> {
    // Implementation
}
```

### Updating Documentation

- Update README.md for user-facing changes
- Update CHANGELOG.md for all changes
- Add examples to doc comments
- Update API documentation as needed

## 🏗️ Code Standards

### Formatting

Use `rustfmt` for consistent formatting:
```bash
cargo fmt
```

### Linting

Use `clippy` for code quality:
```bash
cargo clippy -- -D warnings
```

### Style Guidelines

1. **Naming Conventions**
   - Functions: `snake_case`
   - Types: `PascalCase`
   - Constants: `SCREAMING_SNAKE_CASE`
   - Modules: `snake_case`

2. **Error Handling**
   - Use `Result<T, E>` for fallible operations
   - Prefer custom error types over `Box<dyn Error>`
   - Document error conditions

3. **Async Code**
   - Use `async/await` syntax
   - Avoid blocking operations in async contexts
   - Use appropriate async runtimes (Tokio)

4. **Performance**
   - Avoid unnecessary allocations
   - Use zero-copy operations when possible
   - Profile performance-critical code

## 🐛 Bug Reports

### Before Reporting

1. Search existing issues
2. Verify on latest version
3. Create minimal reproduction case

### Bug Report Template

```markdown
**Describe the Bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. See error

**Expected Behavior**
What you expected to happen.

**Actual Behavior**
What actually happened.

**Environment**
- OS: [e.g. Windows 10, macOS 11, Ubuntu 20.04]
- Rust Version: [e.g. 1.70.0]
- fetchttp Version: [e.g. 0.1.0]

**Additional Context**
Any other context about the problem.

**Code Example**
```rust
// Minimal code example that reproduces the issue
```

## 💡 Feature Requests

### Before Requesting

1. Check if feature aligns with WHATWG Fetch specification
2. Search existing issues for similar requests
3. Consider if feature belongs in this library

### Feature Request Template

```markdown
**Feature Description**
A clear description of the feature you'd like added.

**Use Case**
Describe the problem this feature would solve.

**Proposed API**
If applicable, show what the API might look like.

**Alternatives**
Describe alternatives you've considered.

**WHATWG Compliance**
How does this relate to the WHATWG Fetch specification?
```

## 🔄 Pull Requests

### Before Submitting

1. **Fork** the repository
2. **Create** a feature branch
3. **Make** your changes
4. **Add** tests for new functionality
5. **Update** documentation
6. **Run** the full test suite
7. **Check** formatting and linting

### Pull Request Process

1. **Create** the pull request with a clear title and description
2. **Link** related issues using keywords (fixes #123)
3. **Request** review from maintainers
4. **Address** review feedback
5. **Wait** for approval and merge

### Pull Request Template

```markdown
**Description**
Brief description of changes.

**Related Issues**
Fixes #123

**Type of Change**
- [ ] Bug fix
- [ ] New feature  
- [ ] Breaking change
- [ ] Documentation update

**Testing**
- [ ] Tests pass locally
- [ ] Added tests for new functionality
- [ ] Integration tests pass

**Checklist**
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No breaking changes (or marked as such)
```

## 📊 Performance Considerations

### Benchmarking

```bash
# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench headers

# Generate benchmark report
cargo bench -- --output-format html
```

### Performance Guidelines

1. **Avoid Allocations**: Use zero-copy operations when possible
2. **Connection Reuse**: Leverage connection pooling
3. **Streaming**: Use streaming for large bodies
4. **Caching**: Cache expensive computations
5. **Profiling**: Use profiling tools to identify bottlenecks

## 🚦 Release Process

### Version Numbering

Follow [Semantic Versioning](https://semver.org/):
- **MAJOR**: Incompatible API changes
- **MINOR**: Backwards-compatible functionality additions
- **PATCH**: Backwards-compatible bug fixes

### Release Checklist

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run full test suite
4. Create release PR
5. Tag release after merge
6. Publish to crates.io

## 📞 Getting Help

### Community

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and community discussion

### Maintainers

- **@MuntasirSZN**: Project maintainer

## 📜 Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/2/1/code_of_conduct/).

### Our Pledge

We pledge to make participation in our community a harassment-free experience for everyone.

### Expected Behavior

- Use welcoming and inclusive language
- Be respectful of differing viewpoints
- Accept constructive criticism gracefully
- Focus on what is best for the community

## 🙏 Recognition

Contributors will be recognized in:
- CONTRIBUTORS.md file
- Release notes
- Project documentation

Thank you for contributing to fetchttp! 🎉
