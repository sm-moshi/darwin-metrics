# Rust API Guidelines Checklist

## Checklist

### Naming

- [x] **[C-CASE](https://rust-lang.github.io/api-guidelines/naming.html#c-case)**: Types, traits, and enum variants use `UpperCamelCase`; modules, functions, methods, and macro names use `snake_case`; constants use `SCREAMING_SNAKE_CASE`
- [x] **[C-CONV](https://rust-lang.github.io/api-guidelines/naming.html#c-conv)**: Methods on collections that produce iterators follow naming conventions: `iter()`, `iter_mut()`, `into_iter()`
- [x] **[C-ITER](https://rust-lang.github.io/api-guidelines/naming.html#c-iter)**: Iterator type names match the methods that produce them
- [x] **[C-ITER-TY](https://rust-lang.github.io/api-guidelines/naming.html#c-iter-ty)**: Iterators implement the right iterator traits: `Iterator`, `DoubleEndedIterator`, `ExactSizeIterator`
- [x] **[C-GETTER](https://rust-lang.github.io/api-guidelines/naming.html#c-getter)**: Methods that get a single property have naming conventions: prefer `name()` not `get_name()`
- [x] **[C-BOOL](https://rust-lang.github.io/api-guidelines/naming.html#c-bool)**: Methods returning `bool` have names matching `is_*`, `has_*`, `can_*`, etc.
- [x] **[C-WORD-ORDER](https://rust-lang.github.io/api-guidelines/naming.html#c-word-order)**: Types use consistent word order in names: `*Map` not `Map*`

### Interoperability

- [x] **[C-COMMON-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html#c-common-traits)**: Implement standard/common traits where appropriate: `Copy`, `Clone`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`, `Hash`, `Debug`, `Display`
- [x] **[C-CONV-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html#c-conv-traits)**: Implement conversion traits when appropriate: `From`, `TryFrom`, `AsRef`, `AsMut`
- [x] **[C-COLLECT](https://rust-lang.github.io/api-guidelines/interoperability.html#c-collect)**: Implement `FromIterator` for collections
- [x] **[C-SERDE](https://rust-lang.github.io/api-guidelines/interoperability.html#c-serde)**: If types support serialization/deserialization, they use Serde instead of ad-hoc approaches
- [x] **[C-STRING-DISPLAY](https://rust-lang.github.io/api-guidelines/interoperability.html#c-string-display)**: Display-formatted strings avoid non-standard formatting tokens
- [x] **[C-SEND-SYNC](https://rust-lang.github.io/api-guidelines/interoperability.html#c-send-sync)**: Types are threadsafe (`Send`/`Sync`) when possible
- [x] **[C-SEND-SYNC-ERR](https://rust-lang.github.io/api-guidelines/interoperability.html#c-send-sync-err)**: Error types are `Send` and `Sync`

### Macros

- [x] **[C-EVOCATIVE](https://rust-lang.github.io/api-guidelines/macros.html#c-evocative)**: Macro names are evocative and clear, with `!` part of the name
- [x] **[C-MACRO-ATTR](https://rust-lang.github.io/api-guidelines/macros.html#c-macro-attr)**: Complex functionality is exposed via attributes, not function-like macros
- [x] **[C-WORD-ORDER-MACRO](https://rust-lang.github.io/api-guidelines/macros.html#c-word-order-macro)**: Macros follow consistent naming conventions like types/functions

### Documentation

- [x] **[C-CRATE-DOC](https://rust-lang.github.io/api-guidelines/documentation.html#c-crate-doc)**: Crate has non-empty top-level documentation explaining the crate's purpose and usage
- [x] **[C-EXAMPLE](https://rust-lang.github.io/api-guidelines/documentation.html#c-example)**: Examples show how to use the crate's functionality
- [x] **[C-FEATURE-DOC](https://rust-lang.github.io/api-guidelines/documentation.html#c-feature-doc)**: All optional features listed in Cargo.toml are documented in crate-level docs
- [x] **[C-QUESTION-MARK](https://rust-lang.github.io/api-guidelines/documentation.html#c-question-mark)**: Documentation examples use `?` instead of `try!`, `unwrap()`, or `expect()`
- [x] **[C-ERROR-DOC](https://rust-lang.github.io/api-guidelines/documentation.html#c-error-doc)**: Error conditions documented in function docs
- [x] **[C-PANIC-DOC](https://rust-lang.github.io/api-guidelines/documentation.html#c-panic-doc)**: Functions that panic explicitly documented
- [x] **[C-LINK](https://rust-lang.github.io/api-guidelines/documentation.html#c-link)**: Docs use hyperlinks to relevant types and functions
- [x] **[C-CI](https://rust-lang.github.io/api-guidelines/documentation.html#c-ci)**: Examples in documentation work and are tested
- [x] **[C-HTML-ROOT](https://rust-lang.github.io/api-guidelines/documentation.html#c-html-root)**: Crate sets html_root_url in documentation (`#![doc(html_root_url = "...")]`)
- [x] **[C-DOC-RUST](https://rust-lang.github.io/api-guidelines/documentation.html#c-doc-rust)**: Rust code in documentation is correct and compiles

### Predictability

- [x] **[C-SMART-PTR](https://rust-lang.github.io/api-guidelines/predictability.html#c-smart-ptr)**: Smart pointers follow standard library patterns: `Deref`, `AsRef`
- [x] **[C-CONV-SPECIFIC](https://rust-lang.github.io/api-guidelines/predictability.html#c-conv-specific)**: Conversions are specific and use the appropriate traits
- [x] **[C-METHOD](https://rust-lang.github.io/api-guidelines/predictability.html#c-method)**: Methods are taken by self, not specific types
- [x] **[C-IMPLEMENTS](https://rust-lang.github.io/api-guidelines/predictability.html#c-implements)**: Functions that implement traits use the trait's name in their name
- [x] **[C-OBJECT-ORIENT](https://rust-lang.github.io/api-guidelines/predictability.html#c-object-orient)**: Prefer methods over functions if there's a clear receiver
- [x] **[C-RAII](https://rust-lang.github.io/api-guidelines/predictability.html#c-raii)**: Types provide RAII (Resource Acquisition Is Initialization) guarantee
- [x] **[C-STRUCT-PRIVATE](https://rust-lang.github.io/api-guidelines/predictability.html#c-struct-private)**: Structs have private fields with public getters/setters when needed
- [x] **[C-NEWTYPE](https://rust-lang.github.io/api-guidelines/predictability.html#c-newtype)**: Consider using newtypes to enforce invariants
- [x] **[C-SEALED](https://rust-lang.github.io/api-guidelines/predictability.html#c-sealed)**: Traits are "sealed" (private) when not designed for implementation outside the crate

### Flexibility

- [x] **[C-INTERMEDIATE](https://rust-lang.github.io/api-guidelines/flexibility.html#c-intermediate)**: Functions expose intermediate types for flexibility
- [x] **[C-CALLER-CONTROL](https://rust-lang.github.io/api-guidelines/flexibility.html#c-caller-control)**: Control flow provided to caller instead of callback functions when possible
- [x] **[C-GENERIC](https://rust-lang.github.io/api-guidelines/flexibility.html#c-generic)**: Free functions are generic over input when possible
- [x] **[C-IMPL-TRAIT](https://rust-lang.github.io/api-guidelines/flexibility.html#c-impl-trait)**: Methods use `impl Trait` when appropriate to hide implementation details

### Type Safety

- [x] **[C-NEWTYPE-HIDE](https://rust-lang.github.io/api-guidelines/type-safety.html#c-newtype-hide)**: Newtypes are used to hide implementation details
- [x] **[C-OWNERSHIP](https://rust-lang.github.io/api-guidelines/type-safety.html#c-ownership)**: APIs use Rust's ownership system to ensure correct use
- [x] **[C-VALIDATE](https://rust-lang.github.io/api-guidelines/type-safety.html#c-validate)**: Validate input and fail early when invalid
- [x] **[C-DOWNCAST](https://rust-lang.github.io/api-guidelines/type-safety.html#c-downcast)**: Avoid unnecessary type downcasting
- [x] **[C-OPAQUE](https://rust-lang.github.io/api-guidelines/type-safety.html#c-opaque)**: Use opaque types like `&Path` instead of concrete ones like `&str` when possible

### Dependability

- [x] **[C-STABLE](https://rust-lang.github.io/api-guidelines/dependability.html#c-stable)**: Public API uses stable features when possible
- [x] **[C-PREFER-RESULT](https://rust-lang.github.io/api-guidelines/dependability.html#c-prefer-result)**: Use `Result` instead of panics for errors
- [x] **[C-NO-PANIC](https://rust-lang.github.io/api-guidelines/dependability.html#c-no-panic)**: Functions don't panic under normal operation
- [x] **[C-ERROR-TYPE](https://rust-lang.github.io/api-guidelines/dependability.html#c-error-type)**: Error types are meaningful and well-structured
- [x] **[C-FALLIBLE-CLOSE](https://rust-lang.github.io/api-guidelines/dependability.html#c-fallible-close)**: Fallible resource closures propagate errors
- [x] **[C-FUTURE-PROOFING](https://rust-lang.github.io/api-guidelines/dependability.html#c-future-proofing)**: Use non-exhaustive enums for future-proofing
- [x] **[C-REPR-TRANSPARENT](https://rust-lang.github.io/api-guidelines/dependability.html#c-repr-transparent)**: Single-field wrapper structs use `#[repr(transparent)]` when appropriate

### Debuggability

- [x] **[C-DEBUG](https://rust-lang.github.io/api-guidelines/debuggability.html#c-debug)**: All public types implement `Debug`
- [x] **[C-DEBUG-DERIVE](https://rust-lang.github.io/api-guidelines/debuggability.html#c-debug-derive)**: Derive `Debug` when possible
- [x] **[C-DEBUG-BUILDER](https://rust-lang.github.io/api-guidelines/debuggability.html#c-debug-builder)**: Use debug builder API for complex `Debug` implementations

### Future Proofing

- [x] **[C-EDITION](https://rust-lang.github.io/api-guidelines/future-proofing.html#c-edition)**: Specify Rust edition in Cargo.toml
- [x] **[C-SEMVER](https://rust-lang.github.io/api-guidelines/future-proofing.html#c-semver)**: Respect semantic versioning
- [x] **[C-SVO](https://rust-lang.github.io/api-guidelines/future-proofing.html#c-svo)**: Sealed Vtable Optimization (avoid boxing trait objects when possible)
- [x] **[C-BLANKET](https://rust-lang.github.io/api-guidelines/future-proofing.html#c-blanket)**: Avoid overly generic blanket implementations

### Necessities

- [x] **[C-FILENAME](https://rust-lang.github.io/api-guidelines/necessities.html#c-filename)**: File names follow crate name in `kebab-case`
- [x] **[C-METADATA](https://rust-lang.github.io/api-guidelines/necessities.html#c-metadata)**: Include all useful metadata in Cargo.toml
- [x] **[C-DOCS-RS](https://rust-lang.github.io/api-guidelines/necessities.html#c-docs-rs)**: Ensure docs.rs documentation builds successfully with all features
- [x] **[C-NO-STD](https://rust-lang.github.io/api-guidelines/necessities.html#c-no-std)**: Document `no_std` support appropriately if provided
- [x] **[C-REEXPORT](https://rust-lang.github.io/api-guidelines/necessities.html#c-reexport)**: Re-export important types at crate root or appropriate modules
- [x] **[C-PERMISSIVE](https://rust-lang.github.io/api-guidelines/necessities.html#c-permissive)**: Ensure license is permissive enough for users
- [x] **[C-CLIPPY](https://rust-lang.github.io/api-guidelines/necessities.html#c-clippy)**: Code passes Clippy checks within reason
- [x] **[C-DENY-WARNINGS](https://rust-lang.github.io/api-guidelines/necessities.html#c-deny-warnings)**: Do not use
  `#![deny(warnings)]` in published code


