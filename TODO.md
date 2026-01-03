# q-explore TODO

Code review findings organized into actionable chunks.

---

## Chunk 1: Critical Fixes
**Status**: complete

### 1.1 Add SSL warning for ANU free tier
- **File**: `src/qrng/anu.rs:125-128`
- **Issue**: SSL verification disabled without warning
- **Fix**: Add eprintln warning when using free tier, document security implications

### 1.2 Validate grid_resolution > 0
- **File**: `src/server/routes.rs:129-171`
- **Issue**: grid_resolution=0 causes division by zero
- **Fix**: Add validation in generate handler, return proper error

### 1.3 Handle history save errors
- **File**: `src/cli/generate.rs:169-173`
- **Issue**: `let _ = history.save()` silently drops errors
- **Fix**: Log warning on save failure

---

## Chunk 2: Constants Consolidation
**Status**: complete

### 2.1 Create src/constants.rs
- **Issue**: Constants scattered across files (METERS_PER_DEG_LAT in 3 places, API URLs, cache TTL)
- **Fix**: Centralize in `src/constants.rs` with organized submodules

### 2.2 Replace duplicated constants
- **Files**: `src/coord/density.rs`, `src/coord/flower.rs`, `src/geo/*.rs`
- **Fix**: Import from constants module

---

## Chunk 3: Config DRY Refactor
**Status**: complete

### 3.1 Macro for default value functions
- **File**: `src/config/mod.rs:112-157`
- **Issue**: 12 trivial wrapper functions
- **Fix**: Create macro or inline the constants

### 3.2 Simplify get/set methods
- **File**: `src/config/mod.rs:271-363`
- **Issue**: 90+ lines of repetitive match arms
- **Fix**: Consider registry pattern or macro generation

---

## Chunk 4: Grid Resolution Config
**Status**: complete

### 4.1 Add grid_resolution to config defaults
- **File**: `src/config/mod.rs`, `src/config/defaults.rs`
- **Issue**: Hard-coded as 50 in CLI
- **Fix**: Add `defaults.grid_resolution` config key

### 4.2 Add --grid-resolution CLI arg
- **File**: `src/cli/generate.rs`
- **Fix**: Add optional arg, fall back to config default

---

## Chunk 5: Helper Extractions
**Status**: complete

### 5.1 Extract z-score formatting helper
- **Files**: `src/cli/history.rs:114-116`, `src/format/text.rs:43-46`, `src/format/gpx.rs:78-79`
- **Issue**: Same formatting logic in 3 places
- **Fix**: Add `Point::format_z_score()` method

### 5.2 Extract location resolution function
- **File**: `src/cli/generate.rs:97-127`
- **Issue**: 31 lines of nested conditionals
- **Fix**: Create `async fn resolve_location()` helper

### 5.3 Fix GPX capitalize function
- **File**: `src/format/gpx.rs:64-75`
- **Issue**: Assumes ASCII, fragile for Unicode
- **Fix**: Use proper `chars().next()` pattern

---

## Chunk 6: Server Routes Refactor
**Status**: skipped (large structural change, defer to future)

### 6.1 Split routes.rs into modules
- **File**: `src/server/routes.rs` (724 lines)
- **Fix**: Create `src/server/routes/` directory with:
  - `mod.rs` - route setup
  - `handlers.rs` - handler functions
  - `models.rs` - request/response DTOs
  - `errors.rs` - error conversions

---

## Chunk 7: Error Handling Consistency
**Status**: complete

### 7.1 Add Error::error_code() method
- **File**: `src/server/routes.rs:112-124`
- **Issue**: Error code matching duplicated
- **Fix**: Move to `impl Error` method

### 7.2 Establish error handling policy
- **Issue**: Mix of silent failures, loud errors, and propagation
- **Fix**: Document policy: primary ops propagate, secondary ops warn

---

## Chunk 8: Race Condition Fix
**Status**: complete

### 8.1 Fix config read race in generate handler
- **File**: `src/server/routes.rs:145-156`
- **Issue**: Two separate lock acquisitions allow race
- **Fix**: Read config once, extract both backend_name and api_key

---

## Chunk 9: Testing Improvements
**Status**: pending

### 9.1 Add boundary case tests
- **Files**: `src/coord/point.rs`, `src/coord/density.rs`
- **Cases**: radius < 1m, grid_resolution = 1, point count = 0

### 9.2 Add error path tests
- **Cases**: Invalid JSON from APIs, network timeouts, corrupted history

### 9.3 Add integration tests
- **Fix**: Create `tests/` directory with CLI and API integration tests

---

## Chunk 10: Documentation
**Status**: pending

### 10.1 Add module docs
- **Files**: `src/format/gpx.rs`, `src/format/url.rs`, `src/server/state.rs`

### 10.2 Document config keys
- **File**: `src/config/mod.rs`
- **Fix**: Add doc comments listing valid values for each key

### 10.3 Add algorithm reference
- **File**: `src/coord/point.rs:40-92`
- **Fix**: Add link to mathematical reference, explain approach choice

---

## Completed

(Items move here when done)

---
