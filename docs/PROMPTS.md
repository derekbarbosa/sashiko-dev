# Prompt Customization Guide

## Quick Start (5 Minutes)

Get started customizing Sashiko's review prompts quickly.

### 1. Edit a Single Stage (Easiest)

Edit an existing stage instruction directly:

```bash
# Edit stage 1 (architectural review)
vim third_party/prompts/kernel/stages/01-analyze-goal.md

# Changes apply immediately - no rebuild needed
```

### 2. Add Template Variables

Customize prompts with variables. Edit `Settings.toml`:

```toml
[prompts.variables]
project_name = "My Kernel Module"
focus_area = "memory safety"
```

Then use in prompts:
```markdown
# Stage 1: Review for {{project_name}}
Focus specifically on {{focus_area}}.
Review date: {{date}}
```

### 3. Create Custom Prompts Directory

Copy and customize all prompts:

```bash
# Copy default prompts
cp -r third_party/prompts/kernel my-prompts

# Edit your copies
vim my-prompts/stages/01-analyze-goal.md

# Configure Sashiko to use them
echo '[prompts]
directory = "./my-prompts"' >> Settings.toml
```

### 4. Disable a Stage

Create `third_party/prompts/kernel/stages.toml`:

```toml
# Disable hardware review (stage 7)
[[stages]]
number = 7
enabled = false
```

### 5. Add a Custom Stage

In `stages.toml`:

```toml
[[stages]]
number = 10
name = "Performance analysis"
instruction_file = "custom/performance.md"
enabled = true
```

Create `third_party/prompts/kernel/custom/performance.md`:

```markdown
# Stage 10: Performance Analysis

Analyze the patch for performance implications:
- Algorithmic complexity (O(n), O(n²), etc.)
- Cache efficiency
- Lock contention
- Memory allocation patterns
```

## Overview

Sashiko's review process uses a multi-stage prompt system. You can customize:
- Individual stage instructions
- Supporting guidance files
- Stage order and selection
- Template variables

## Review Stages

### Default Stages

1. **Analyze commit main goal** - Architectural review
2. **Implementation verification** - High-level correctness
3. **Control flow analysis** - Execution flow
4. **Resource management** - Memory, locks, handles
5. **Locking and synchronization** - Concurrency safety
6. **Security audit** - Vulnerability scanning
7. **Hardware review** - Hardware-specific concerns
8. **Verification** - Severity estimation
9. **Report generation** - LKML-friendly output

## Customization Methods

### Method 1: Edit Stage Files

Files in `third_party/prompts/kernel/stages/`:
```bash
vim third_party/prompts/kernel/stages/01-analyze-goal.md
```

Changes take effect immediately (no recompilation).

### Method 2: Custom Prompts Directory

Edit `Settings.toml`:
```toml
[prompts]
directory = "./my-custom-prompts"
```

Your directory structure:
```
my-custom-prompts/
  stages/
    01-analyze-goal.md
    02-implementation.md
    ...
  technical-patterns.md
  stages.toml
```

### Method 3: Remote Prompts

Use prompts from a remote source:
```toml
[prompts]
directory = "https://example.com/sashiko-prompts.tar.gz"
# or
directory = "git://github.com/yourteam/sashiko-prompts.git"
```

Sashiko will download and cache the prompts in `.sashiko-cache/prompts/`.

**Note:** Remote HTTP download is currently a placeholder implementation and requires archive extraction support. Git cloning is fully implemented.

### Method 4: stages.toml Configuration

Create `stages.toml` in your prompts directory:
```toml
[[stages]]
number = 1
instruction_file = "stages/01-custom.md"
supporting_files = ["patterns.md"]
enabled = true

# Add custom stage
[[stages]]
number = 11
name = "Performance analysis"
instruction_file = "custom/performance.md"
enabled = true

# Disable a stage
[[stages]]
number = 7
enabled = false
```

## Template Variables

### Using Variables

In your prompt files:
```markdown
# Review for {{project_name}}

Focus on {{custom_guidelines}}.
Analyze the {{subsystem}} subsystem.
```

Configure in Settings.toml:
```toml
[prompts.variables]
project_name = "My Project"
custom_guidelines = "security and performance"
subsystem = "networking"
```

### Built-in Variables

- `{{date}}` - Current date (YYYY-MM-DD)
- `{{year}}` - Current year

## Examples

### Security-Focused Review

```toml
[prompts]
directory = "./security-prompts"

[prompts.variables]
focus_area = "memory safety and input validation"
threat_model = "untrusted user input from network"
```

### Performance-Focused Review

```toml
[prompts.variables]
focus_area = "algorithmic complexity and cache efficiency"
performance_target = "sub-millisecond latency"
```

### Subsystem-Specific Prompts

```toml
[prompts]
directory = "git://github.com/myorg/networking-prompts.git"

[prompts.variables]
subsystem = "networking"
key_concerns = "packet processing efficiency and protocol compliance"
```

## Best Practices

1. **Start with defaults** - Copy existing `stages/` directory as template
2. **Iterative refinement** - Test prompts, measure results, refine
3. **Version control** - Keep custom prompts in git
4. **Document rationale** - Comment your customizations
5. **Test thoroughly** - Verify AI understands custom instructions

## Directory Structure

A complete custom prompts directory should have:

```
my-prompts/
├── stages.toml              # Stage configuration (optional)
├── stages/                  # Stage instruction files
│   ├── 01-analyze-goal.md
│   ├── 02-implementation.md
│   ├── 03-control-flow.md
│   ├── 04-resource-mgmt.md
│   ├── 05-locking.md
│   ├── 06-security.md
│   ├── 07-hardware.md
│   ├── 08-verification.md
│   └── 09-report.md
├── subsystem/               # Subsystem-specific guidelines
│   ├── locking.md
│   └── ...
├── patterns/                # Pattern files
│   └── ...
├── callstack.md             # Supporting files
├── technical-patterns.md
├── false-positive-guide.md
├── severity.md
├── inline-template.md
└── tool.md
```

## stages.toml Reference

### Full Schema

```toml
[[stages]]
number = 1                           # Stage number (1-9 are defaults)
name = "Custom stage name"           # Optional: Display name
instruction_file = "path/to/file.md" # Path relative to prompts directory
supporting_files = [                 # Optional: Additional context files
    "patterns.md",
    "subsystem/locking.md"
]
enabled = true                       # Optional: Enable/disable stage (default: true)
```

### Example: Reordering Stages

```toml
# Run security audit before control flow
[[stages]]
number = 3
instruction_file = "stages/06-security.md"
enabled = true

[[stages]]
number = 6
instruction_file = "stages/03-control-flow.md"
enabled = true
```

### Example: Adding Custom Stages

```toml
# Add a custom performance analysis stage
[[stages]]
number = 10
name = "Performance analysis"
instruction_file = "custom/performance.md"
supporting_files = ["custom/perf-patterns.md", "custom/hotpaths.md"]
enabled = true
```

## Troubleshooting

### Prompt not loading
- Check file path in `stages.toml` is relative to prompts directory
- Verify `directory` setting in `Settings.toml`
- Check logs for parsing errors: `grep "Loading stages" sashiko.log`

### Variables not substituting
- Verify syntax: `{{variable}}` not `{variable}`
- Check variables defined in `Settings.toml` under `[prompts.variables]`
- Ensure no typos in variable names (case-sensitive)

### Stage skipped
- Check `enabled = true` in `stages.toml`
- Verify stage number matches
- Check logs: disabled stages will show error message

### Git clone failing
- Ensure git is installed and in PATH
- Check repository URL is accessible
- Verify network connectivity
- Check `.sashiko-cache/prompts/` permissions

### Custom directory not found
- Use absolute paths or paths relative to current working directory
- Check directory exists: `ls -la ./my-prompts`
- Verify case sensitivity of directory name

## Advanced Topics

### Caching

Remote prompts (HTTP/Git) are cached in `.sashiko-cache/prompts/` with a hash of the URL as the directory name. To force re-download:

```bash
rm -rf .sashiko-cache/prompts/*
```

### Multiple Prompt Sets

Switch between different prompt configurations by changing `Settings.toml`:

```toml
# For security audits
[prompts]
directory = "./prompts-security"

# For performance reviews
# [prompts]
# directory = "./prompts-performance"
```

### Sharing Prompt Configurations

1. Create a git repository with your custom prompts
2. Push to GitHub/GitLab
3. Share the repository URL:
   ```toml
   [prompts]
   directory = "git://github.com/myorg/sashiko-kernel-prompts.git"
   ```

## See Also

- [TOOLS.md](TOOLS.md) - Configure AI tools
- [TRANSFORMATION_SUMMARY.md](TRANSFORMATION_SUMMARY.md) - System overview
- [Settings.toml](../Settings.toml) - Configuration file reference
