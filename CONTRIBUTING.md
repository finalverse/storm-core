# Contributing to StormCore

Thank you for your interest in contributing to StormCore! This document provides guidelines and information for contributors.

## 🌟 Ways to Contribute

- **🐛 Bug Reports**: Report issues and bugs
- **💡 Feature Requests**: Suggest new features and improvements
- **📝 Documentation**: Improve docs, guides, and examples
- **🔧 Code Contributions**: Fix bugs, add features, optimize performance
- **🧪 Testing**: Add tests, improve coverage, benchmark performance
- **🎨 Examples**: Create demos and usage examples

## 🚀 Getting Started

### Development Environment

1. **Install Rust 1.70+**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup update stable
   ```

2. **Clone and Build**
   ```bash
   git clone https://github.com/finalverse/storm-core.git
   cd storm-core
   cargo build --workspace
   ```

3. **Install Development Tools**
   ```bash
   cargo install cargo-watch cargo-expand cargo-tarpaulin
   rustup component add clippy rustfmt
   ```

### Development Workflow

1. **Create a Branch**
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/issue-description
   ```

2. **Make Changes**
    - Follow the coding standards below
    - Add tests for new functionality
    - Update documentation as needed

3. **Test Your Changes**
   ```bash
   # Run all tests
   cargo test --workspace
   
   # Check formatting
   cargo fmt --all --check
   
   # Run lints
   cargo clippy --workspace -- -D warnings
   
   # Build for all targets
   cargo build --workspace
   ```

4. **Commit and Push**
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   git push origin feature/your-feature-name
   ```

5. **Create Pull Request**
    - Use a descriptive title
    - Reference any related issues
    - Provide detailed description of changes

## 📋 Coding Standards

### Rust Guidelines

- **Follow Rust conventions**: Use `cargo fmt` and `cargo clippy`
- **Error handling**: Use `Result<T, E>` and `anyhow` for error propagation
- **Documentation**: Add doc comments for public APIs
- **Testing**: Include unit tests for new functionality
- **Safety**: Minimize `unsafe` code, document when necessary

### Code Style

```rust
// ✅ Good: Clear naming and documentation
/// Processes incoming network packets for a specific protocol
pub async fn process_packets(&mut self, protocol: ProtocolType) -> Result<usize> {
    let packet_count = self.packet_queue.len();
    
    for packet in self.packet_queue.drain(..) {
        self.handle_packet(packet).await?;
    }
    
    Ok(packet_count)
}

// ❌ Avoid: Unclear naming and no documentation
pub async fn proc(&mut self, p: u8) -> Result<usize> {
    // unclear implementation
}
```

### Architecture Principles

- **Modularity**: Keep crates focused and loosely coupled
- **Async-first**: Use `tokio` for async operations
- **Error propagation**: Use `?` operator and proper error types
- **Performance**: Profile critical paths, avoid premature optimization
- **Cross-platform**: Consider all target platforms in design

## 🧪 Testing Guidelines

### Test Categories

1. **Unit Tests**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_component_creation() {
           let transform = Transform::default();
           assert_eq!(transform.position, [0.0, 0.0, 0.0]);
       }
       
       #[tokio::test]
       async fn test_async_operation() {
           let result = async_function().await;
           assert!(result.is_ok());
       }
   }
   ```

2. **Integration Tests**
   ```rust
   // tests/integration_test.rs
   use storm_core::*;
   
   #[tokio::test]
   async fn test_full_engine_lifecycle() {
       let config = StormConfig::default();
       let engine = StormCore::new(config).await.unwrap();
       // Test complete workflows
   }
   ```

3. **Benchmarks**
   ```rust
   // benches/performance.rs
   use criterion::{criterion_group, criterion_main, Criterion};
   
   fn benchmark_ecs_query(c: &mut Criterion) {
       c.bench_function("ecs_query_1000_entities", |b| {
           b.iter(|| {
               // Benchmark code
           });
       });
   }
   ```

### Test Coverage

- Aim for 80%+ test coverage on new code
- Include edge cases and error conditions
- Test async code with `tokio-test`
- Add performance benchmarks for critical paths

## 📚 Documentation

### API Documentation

```rust
/// Connects to a virtual world using the specified configuration.
/// 
/// This function establishes a connection to either an OpenSim grid or
/// Finalverse server based on the protocol specified in the world config.
/// 
/// # Arguments
/// 
/// * `world_config` - Configuration containing connection details
/// 
/// # Returns
/// 
/// Returns `Ok(())` on successful connection, or a `StormError` on failure.
/// 
/// # Examples
/// 
/// ```rust
/// use storm_core::*;
/// 
/// let config = WorldConfig::opensim("Test Grid", "http://grid.example.com");
/// engine.connect_to_world(&config).await?;
/// ```
/// 
/// # Errors
/// 
/// This function will return an error if:
/// - Network connection fails
/// - Authentication is rejected
/// - Protocol version is incompatible
pub async fn connect_to_world(&self, world_config: &WorldConfig) -> StormResult<()> {
    // Implementation
}
```

### Architecture Documentation

- Document design decisions and trade-offs
- Include diagrams for complex systems
- Explain FFI interfaces and safety considerations
- Provide examples for common use cases

## 🎯 Contribution Areas

### High Priority

- **Performance Optimization**: ECS queries, network processing, AI inference
- **Platform Support**: iOS, Android, WebAssembly improvements
- **Protocol Implementation**: OpenSim LLUDP, Finalverse enhancements
- **AI Features**: Local ML models, Grok integration improvements
- **Documentation**: API docs, architecture guides, examples

### Good First Issues

Look for issues labeled `good first issue` or `help wanted`:

- Bug fixes in specific crates
- Documentation improvements
- Test coverage improvements
- Example applications
- Performance benchmarks

## 🔄 Review Process

### Code Review Guidelines

- **Functionality**: Does the code work as intended?
- **Performance**: Are there any obvious performance issues?
- **Safety**: Are `unsafe` blocks justified and documented?
- **Testing**: Is the code adequately tested?
- **Documentation**: Are public APIs documented?
- **Style**: Does the code follow project conventions?

### Review Checklist

- [ ] Code builds successfully on all platforms
- [ ] All tests pass
- [ ] New functionality is tested
- [ ] Documentation is updated
- [ ] Breaking changes are documented
- [ ] Performance impact is considered

## 🚨 Issue Reporting

### Bug Reports

Use the bug report template and include:

- **Environment**: OS, Rust version, target platform
- **Steps to reproduce**: Minimal example
- **Expected behavior**: What should happen
- **Actual behavior**: What actually happens
- **Logs**: Relevant error messages or logs

### Feature Requests

Use the feature request template and include:

- **Problem**: What problem does this solve?
- **Solution**: Proposed implementation approach
- **Alternatives**: Other solutions considered
- **Impact**: Who benefits and how?

## 🎖️ Recognition

Contributors are recognized in:

- Release notes for significant contributions
- README acknowledgments section
- Special contributor badges
- Community highlights

## 📞 Communication

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Design discussions and questions
- **Discord**: Real-time chat (coming soon)
- **Email**: security@finalverse.com for security issues

## 📖 Resources

- **[Rust Book](https://doc.rust-lang.org/book/)** - Learn Rust fundamentals
- **[Tokio Tutorial](https://tokio.rs/tokio/tutorial)** - Async Rust programming
- **[Legion Guide](https://docs.rs/legion/)** - ECS architecture
- **[FFI Guide](https://doc.rust-lang.org/nomicon/ffi.html)** - Foreign function interface

Thank you for contributing to StormCore! 🌪️