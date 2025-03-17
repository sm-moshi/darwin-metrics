refactor: comprehensive module reorganisation and structural improvements

This commit includes a major refactoring of the codebase structure to improve organisation, 
maintainability, and reduce circular dependencies. Key changes include:

1. Traits Reorganisation:
   - Created a centralized `src/traits/` directory
   - Moved hardware monitoring traits from `core/metrics/hardware.rs` to `traits/hardware.rs`
   - Marked old trait locations as deprecated with helpful migration messages
   - Updated example code in trait documentation to reference new locations

2. Hardware Module Structure:
   - Flattened the module hierarchy by moving hardware components to src-level
   - Replaced nested `hardware/battery` with a top-level `battery` module
   - Improved consistency in module organisation across components

3. Monitors Consolidation:
   - Merged monitor directories (e.g., `battery/monitors/`) into single files (e.g., `battery/monitors.rs`)
   - organised code with clear section headers for better readability
   - Maintained all public interfaces for backward compatibility

4. Import/Export Updates:
   - Updated import paths across the codebase
   - Added re-exports to maintain backward compatibility
   - Ensured that public APIs remain stable for consumers

5. Dependency Improvements:
   - Reduced circular dependencies between modules
   - Improved separation of concerns between trait definitions and implementations
   - Better organised related functionality

This refactoring simplifies the codebase structure while maintaining compatibility 
with existing code. It provides a clearer separation between interfaces (traits) 
and implementations, making the code more maintainable and easier to navigate. 