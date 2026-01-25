/**
 * hyprlog C++ FFI Example
 *
 * This demonstrates how the C header (hyprlog.h) connects to
 * the Rust implementation (libhyprlog.so).
 *
 * Build flow:
 *   1. Rust: cargo build --release
 *      -> compiles lib.rs to libhyprlog.so (actual code)
 *
 *   2. C++: cmake && make
 *      -> includes hyprlog.h (declarations only)
 *      -> links against libhyprlog.so (implementation)
 *
 *   3. Runtime: LD_LIBRARY_PATH points to libhyprlog.so
 *      -> dynamic linker loads the Rust library
 *      -> function calls jump into Rust code
 */

#include <iostream>
#include <string>

// This header only contains DECLARATIONS:
// - function signatures (what parameters, what return type)
// - struct declarations (opaque pointer: HyprlogContext*)
// - constants (HYPRLOG_LEVEL_*)
//
// NO actual code - just tells the compiler "trust me, these exist"
#include "hyprlog.h"

void demonstrate_basic_logging() {
    std::cout << "\n=== Basic Logging ===" << std::endl;
    std::cout << "Creating context with hyprlog_init()..." << std::endl;

    // This call:
    // 1. C++ compiler sees declaration in hyprlog.h
    // 2. Linker finds symbol 'hyprlog_init' in libhyprlog.so
    // 3. At runtime: jumps into Rust code, executes Logger::builder()...
    // 4. Returns pointer to heap-allocated HyprlogContext (Box::into_raw)
    HyprlogContext* ctx = hyprlog_init();

    if (!ctx) {
        std::cerr << "Failed to initialize logger!" << std::endl;
        return;
    }

    std::cout << "Context created at address: " << ctx << std::endl;
    std::cout << "\nLogging messages at different levels:\n" << std::endl;

    // Each of these calls:
    // 1. Passes C strings to Rust (const char* -> CStr)
    // 2. Rust converts to &str, calls Logger::log()
    // 3. Output goes to terminal/file based on config
    hyprlog_trace(ctx, "DEMO", "This is a TRACE message (level 0)");
    hyprlog_debug(ctx, "DEMO", "This is a DEBUG message (level 1)");
    hyprlog_info(ctx, "DEMO", "This is an INFO message (level 2)");
    hyprlog_warn(ctx, "DEMO", "This is a WARN message (level 3)");
    hyprlog_error(ctx, "DEMO", "This is an ERROR message (level 4)");

    // This call:
    // 1. Rust receives the pointer
    // 2. Box::from_raw() reclaims ownership
    // 3. drop() deallocates the HyprlogContext
    std::cout << "\nFreeing context with hyprlog_free()..." << std::endl;
    hyprlog_free(ctx);
    std::cout << "Context freed. Pointer is now invalid!" << std::endl;
}

void demonstrate_simple_init() {
    std::cout << "\n=== Simple Init (No Config File) ===" << std::endl;

    // hyprlog_init_simple bypasses config file loading
    // Parameters: level (0-4), colors (0=off, 1=on)
    HyprlogContext* ctx = hyprlog_init_simple(HYPRLOG_LEVEL_DEBUG, 1);

    if (!ctx) {
        std::cerr << "Failed to create simple logger!" << std::endl;
        return;
    }

    hyprlog_info(ctx, "SIMPLE", "Logger created with DEBUG level and colors ON");
    hyprlog_debug(ctx, "SIMPLE", "This debug message should appear");
    hyprlog_trace(ctx, "SIMPLE", "This trace message should NOT appear (below DEBUG)");

    hyprlog_free(ctx);
}

void demonstrate_generic_log() {
    std::cout << "\n=== Generic Log Function ===" << std::endl;

    HyprlogContext* ctx = hyprlog_init_simple(HYPRLOG_LEVEL_TRACE, 1);
    if (!ctx) return;

    // hyprlog_log() takes level as parameter
    // Useful when level is determined at runtime
    for (int level = HYPRLOG_LEVEL_TRACE; level <= HYPRLOG_LEVEL_ERROR; ++level) {
        std::string msg = "Message at level " + std::to_string(level);
        hyprlog_log(ctx, level, "LOOP", msg.c_str());
    }

    hyprlog_free(ctx);
}

void demonstrate_error_handling() {
    std::cout << "\n=== Error Handling ===" << std::endl;

    HyprlogContext* ctx = hyprlog_init_simple(HYPRLOG_LEVEL_INFO, 0);
    if (!ctx) return;

    // Buffer for error messages
    char error_buffer[256];

    // Check for errors (there shouldn't be any yet)
    int err_len = hyprlog_get_last_error(ctx, error_buffer, sizeof(error_buffer));

    if (err_len == 0) {
        std::cout << "No errors recorded (expected)" << std::endl;
    } else if (err_len > 0) {
        std::cout << "Error: " << error_buffer << std::endl;
    } else {
        std::cout << "Error retrieval failed" << std::endl;
    }

    // Flush ensures all buffered output is written
    int flush_result = hyprlog_flush(ctx);
    std::cout << "Flush result: " << flush_result << " (0 = success)" << std::endl;

    hyprlog_free(ctx);
}

void demonstrate_null_safety() {
    std::cout << "\n=== Null Safety ===" << std::endl;

    // All functions handle NULL gracefully (Rust checks is_null())
    std::cout << "Calling functions with NULL context..." << std::endl;

    hyprlog_log(nullptr, HYPRLOG_LEVEL_INFO, "TEST", "This won't crash");
    hyprlog_info(nullptr, "TEST", "Neither will this");
    hyprlog_free(nullptr);  // Safe to call with NULL

    std::cout << "No crashes! Rust FFI handles NULL correctly." << std::endl;
}

void explain_memory_model() {
    std::cout << "\n=== Memory Model Explanation ===" << std::endl;
    std::cout << R"(
    C++ Side                           Rust Side (libhyprlog.so)
    =========                          ========================

    HyprlogContext* ctx;               pub struct HyprlogContext {
         |                                 logger: Logger,
         |                                 last_error: RefCell<Option<String>>,
         |                             }
         |
         v
    ctx = hyprlog_init();  ------>     Box::new(HyprlogContext { ... })
         |                             Box::into_raw(ctx)  // returns *mut
         |                                    |
         |<----- raw pointer ----------------+
         |
    hyprlog_info(ctx, ...)  ------>    unsafe { &*ctx }.logger.log(...)
         |                                    |
         |                             (borrows, doesn't own)
         |
    hyprlog_free(ctx);  ---------->    Box::from_raw(ctx)  // reclaims ownership
                                       drop()              // destructor runs

    Key Points:
    - C++ only holds a raw pointer (no ownership concept)
    - Rust Box manages the actual memory
    - hyprlog_free() MUST be called to avoid memory leak
    - After free(), the pointer is dangling - don't use it!
    )" << std::endl;
}

int main() {
    std::cout << "========================================" << std::endl;
    std::cout << "   hyprlog C++ FFI Demonstration" << std::endl;
    std::cout << "========================================" << std::endl;

    explain_memory_model();
    demonstrate_basic_logging();
    demonstrate_simple_init();
    demonstrate_generic_log();
    demonstrate_error_handling();
    demonstrate_null_safety();

    std::cout << "\n========================================" << std::endl;
    std::cout << "   Demonstration Complete" << std::endl;
    std::cout << "========================================" << std::endl;

    return 0;
}
