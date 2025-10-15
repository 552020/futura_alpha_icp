# Error Handling Improvements for DFX Generate

## Current Problems

### 1. Unsafe Template Rendering

**Location**: `secretus/sdk/src/dfx/src/lib/builders/mod.rs:315`

```rust
// BAD: Will panic on template errors
let new_file_contents = handlebars.render_template(&file_contents, &data).unwrap();
```

### 2. Missing Error Context

The template rendering has no error context, making debugging difficult.

### 3. No Data Validation

No validation that required template variables are present.

## Recommended Fixes

### 1. Proper Error Handling with Context

```rust
// IMPROVED: Proper error handling with context
let new_file_contents = handlebars
    .render_template(&file_contents, &data)
    .with_context(|| {
        format!(
            "Failed to render handlebars template for canister '{}' with template '{}'",
            info.get_name(),
            pathname.display()
        )
    })?;
```

### 2. Data Validation Before Rendering

```rust
// IMPROVED: Validate required data before rendering
fn validate_template_data(data: &BTreeMap<String, &String>, template_name: &str) -> DfxResult {
    let required_fields = ["canister_name", "canister_name_ident"];

    for field in required_fields {
        if !data.contains_key(field) {
            bail!(
                "Missing required template field '{}' for template '{}'",
                field,
                template_name
            );
        }
    }

    Ok(())
}

// Usage:
validate_template_data(&data, pathname.to_str().unwrap_or("unknown"))?;
let new_file_contents = handlebars
    .render_template(&file_contents, &data)
    .with_context(|| {
        format!(
            "Failed to render handlebars template for canister '{}' with template '{}'",
            info.get_name(),
            pathname.display()
        )
    })?;
```

### 3. Complete Improved Implementation

```rust
fn compile_handlebars_files(
    lang: &str,
    info: &CanisterInfo,
    generate_output_dir: &Path,
) -> DfxResult {
    let mut language_bindings = crate::util::assets::language_bindings()
        .context("Failed to get language bindings archive.")?;

    for f in language_bindings
        .entries()
        .context("Failed to read language bindings archive entries.")?
    {
        let mut file = f.context("Failed to read language bindings archive entry.")?;

        let pathname: PathBuf = file
            .path()
            .context("Failed to read language bindings entry path name.")?
            .to_path_buf();
        let file_extension = format!("{}.hbs", lang);
        let is_template = pathname
            .to_str()
            .is_some_and(|name| name.ends_with(&file_extension));

        if is_template {
            let mut file_contents = String::new();
            file.read_to_string(&mut file_contents)
                .with_context(|| {
                    format!(
                        "Failed to read template file '{}' from language bindings archive",
                        pathname.display()
                    )
                })?;

            // Create the handlebars registry with proper error handling
            let handlebars = Handlebars::new();

            let mut data: BTreeMap<String, &String> = BTreeMap::new();

            let canister_name = &info.get_name().to_string();
            let canister_name_ident = &canister_name.replace('-', "_");

            let node_compatibility = info.get_declarations_config().node_compatibility;

            // Insert only if node outputs are specified
            let actor_export = if node_compatibility {
                "".to_string()
            } else {
                format!(
                    r#"

export const {canister_name_ident} = canisterId ? createActor(canisterId) : undefined;"#,
                )
                .to_string()
            };

            data.insert("canister_name".to_string(), canister_name);
            data.insert("canister_name_ident".to_string(), canister_name_ident);
            data.insert("actor_export".to_string(), &actor_export);

            // Environment variable handling with validation
            let process_string_prefix: String = match &info.get_declarations_config().env_override {
                Some(s) => {
                    if s.is_empty() {
                        bail!("env_override cannot be empty for canister '{}'", info.get_name());
                    }
                    format!(r#""{}""#, s.clone())
                },
                None => {
                    format!(
                        "process.env.{}{}",
                        "CANISTER_ID_",
                        &canister_name_ident.to_ascii_uppercase(),
                    )
                }
            };

            data.insert(
                "canister_name_process_env".to_string(),
                &process_string_prefix,
            );

            // Validate template data before rendering
            validate_template_data(&data, pathname.to_str().unwrap_or("unknown"))?;

            // Render template with proper error handling
            let new_file_contents = handlebars
                .render_template(&file_contents, &data)
                .with_context(|| {
                    format!(
                        "Failed to render handlebars template for canister '{}' with template '{}'. Check that all required variables are present.",
                        info.get_name(),
                        pathname.display()
                    )
                })?;

            // Write file with proper error handling
            let new_path = generate_output_dir.join(pathname.with_extension(""));
            std::fs::write(&new_path, new_file_contents)
                .with_context(|| {
                    format!(
                        "Failed to write generated file '{}' for canister '{}'",
                        new_path.display(),
                        info.get_name()
                    )
                })?;

            trace!(logger, "  {}", &new_path.display());
        }
    }
    Ok(())
}

fn validate_template_data(data: &BTreeMap<String, &String>, template_name: &str) -> DfxResult {
    let required_fields = ["canister_name", "canister_name_ident"];

    for field in required_fields {
        if !data.contains_key(field) {
            bail!(
                "Missing required template field '{}' for template '{}'",
                field,
                template_name
            );
        }
    }

    Ok(())
}
```

## Benefits of Improved Error Handling

### 1. **No More Panics**

- Template rendering failures won't crash the application
- Graceful error messages with context

### 2. **Better Debugging**

- Clear error messages indicating what went wrong
- Context about which canister and template failed

### 3. **Data Validation**

- Ensures required template variables are present
- Validates configuration values before use

### 4. **Maintainability**

- Easier to debug template issues
- Clear separation of concerns

## Current Risk Assessment

**HIGH RISK**: The current `.unwrap()` usage means:

- Any template syntax error will crash `dfx generate`
- Missing template variables will cause panics
- No graceful degradation for template issues

## Recommendation

**URGENT**: This should be fixed immediately as it makes the `dfx generate` command fragile and prone to crashes when:

- Template syntax is invalid
- Required data is missing
- File system operations fail
- Configuration is malformed
