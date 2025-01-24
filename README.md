Here's a simplified README focusing on your core requirements:

```markdown
# Rivet (Experimental)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

> **Rivet** - A compile-time dependency injection framework for Rust  
> *Name inspired by mechanical fasteners - symbolizing secure component connections*

## Why Rivet?
- **Zero-cost abstractions** through compile-time magic
- **Spring Boot inspired** DI concepts in Rust-native style
- **No runtime overhead** - All wiring happens during compilation

## Core Features
- 🛠️ **Compile-time component scanning** using Rust's module system
- 🔒 **Safe initialization markers** (`Inited<T>` proof)
- 📦 **Type registry** built via procedural macros
- 🧵 **Thread-safe static access** without locks/atomic

## Design Goals
```rust
// Zero-cost initialization proof
struct Inited<T>(PhantomData<T>);

// Type registry (compile-time generated)
static REGISTRY: Registry = {
    let mut m = HashMap::new();
    // Generated during macro expansion
    m.insert(TypeId::of::<MyBean>(), init_my_bean);
    m
};
```

## How It Works
1. **Component Scanning**  
   Procedural macro scans crate/modules using `syn`:
   ```rust
   #[rivet::bean]
   struct Database {
       url: String
   }
   ```

2. **Compile-Time Registry**  
   Generates initialization map:
   ```rust
   type InitFn = fn() -> Inited<dyn Any>;
   
   struct Registry {
       beans: HashMap<TypeId, InitFn>,
   }
   ```

3. **Safe Access**  
   Runtime access with initialization guarantee:
   ```rust
   let db: &Database = rivet::get::<Database>().expect("Bean initialized");
   ```

## Roadmap
- **MVC Extension**  
  Auto-collect HTTP handlers from annotations
- **Scheduled Tasks**  
  Cron-like syntax with compile-time validation
- **DB Mapper**  
  Type-safe SQL via procedural macros
- **Declarative HTTP Client**  
  Feign-style interface definitions

## Limitations (0.0.1)
- Single-threaded initialization only
- Basic type registry (no dependency ordering)
- Manual component scan registration

## Contribution
Found a bug? Want a feature? Open an issue!  
*See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.*

---

> **Warning**: Experimental prototype - Not production ready  
> Current focus: Prove core zero-cost DI concept
```

Key implementation notes:
1. `Inited<T>` acts as zero-sized initialization proof
2. Type registry uses `TypeId` as key with generated init functions
3. Procedural macro generates registry population code during compilation
4. Static access uses Rust's type system instead of runtime checks

Would you like me to elaborate on any specific aspect of the implementation?
