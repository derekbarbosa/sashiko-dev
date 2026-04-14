# AI Tools Configuration

This document describes Sashiko's AI tools system and how to configure which tools are available to the AI agent during code reviews.

## Overview

Sashiko provides 14 built-in tools that enable the AI agent to interact with your codebase. By default, all tools are enabled, but you can selectively enable or disable tools via `Settings.toml` configuration.

## Built-in Tools Reference

### File Operations

#### `read_files`
Read the content of one or more files. Supports "smart" mode which collapses irrelevant code around focus lines.

**Parameters:**
- `files`: Array of file objects with `path`, optional `start_line`, and `end_line`
- `mode`: "raw" (default) or "smart"

**Example use case:** Reading source files to understand implementation details.

#### `list_dir`
List files and directories in a specified path.

**Parameters:**
- `path`: Directory path to list

**Example use case:** Exploring project structure.

#### `find_files`
Find files matching a glob pattern (e.g., `*.rs`, `src/**/mod.rs`).

**Parameters:**
- `pattern`: Glob pattern to match
- `path`: Directory to search in (defaults to root)

**Example use case:** Locating all test files or configuration files.

#### `search_file_content`
Search for a pattern in files using grep-like functionality. Returns matching lines with optional context.

**Parameters:**
- `pattern`: Regex pattern to search for
- `path`: Directory to search in (defaults to root)
- `context_lines`: Number of context lines to show (default 0)

**Example use case:** Finding all occurrences of a function call or API usage.

### Git Operations

#### `git_diff`
Show changes between commits, commit and working tree, etc.

**Parameters:**
- `args`: Array of arguments for git diff (e.g., `['HEAD^', 'HEAD']`)

**Example use case:** Reviewing changes in a patch series.

#### `git_log`
Show commit logs. Supports extensive filtering options.

**Parameters:**
- `args`: Array of arguments for git log (e.g., `['-n', '3', '--oneline']`)

**Important:** When using expensive search flags like `-S` or `-G`, always limit the search range using `--since` (e.g., `--since=1.year.ago`) to avoid timeouts.

**Example use case:** Understanding commit history for changed code.

#### `git_show`
Show various types of git objects (blobs, trees, tags, commits). Supports line filtering for blobs.

**Parameters:**
- `object`: The object to show (e.g., `HEAD:README.md` or `HEAD`)
- `suppress_diff`: Boolean to suppress diff output for commits (shows only metadata)
- `start_line`, `end_line`: Optional line range for blobs

**Example use case:** Viewing a file at a specific commit or examining commit metadata.

#### `git_blame`
Show what revision and author last modified each line of a file.

**Parameters:**
- `path`: Relative path to the file
- `start_line`, `end_line`: Optional line range

**Example use case:** Understanding who introduced specific code and when.

#### `git_status`
Show the working tree status.

**Parameters:** None

**Example use case:** Checking for uncommitted changes.

#### `git_branch`
List both remote-tracking branches and local branches.

**Parameters:** None

**Example use case:** Understanding branch structure.

#### `git_tag`
List tags in the repository.

**Parameters:** None

**Example use case:** Finding version tags or release points.

#### `git_checkout`
Switch branches or restore working tree files.

**Parameters:**
- `target`: The branch or commit to checkout

**Example use case:** Switching to a different branch for comparison.

**Security note:** This tool modifies the working tree. Consider disabling for read-only reviews.

### Documentation Tools

#### `read_prompt`
Read a specific prompt file from the prompt registry.

**Parameters:**
- `name`: Name of the prompt file (e.g., `patterns/BPF-001.md`)

**Availability:** Only enabled when `prompts_path` is configured.

**Example use case:** Loading kernel-specific coding patterns and guidelines.

### Task Management

#### `TodoWrite`
Add a new TODO item to the TODO.md file.

**Parameters:**
- `content`: The TODO item content

**Example use case:** Recording issues discovered during review for later action.

**Security note:** This tool creates/modifies files. Consider disabling for read-only reviews.

## Configuration

### Default Behavior

If the `[tools]` section is omitted from `Settings.toml`, **all tools are enabled**. This maintains backward compatibility.

### Whitelist Mode

Enable only specific tools:

```toml
[tools]
enabled = [
    "read_files",
    "git_diff",
    "git_log",
    "git_show",
    "git_blame",
    "search_file_content"
]
```

This configuration gives the AI agent read-only access to code and history without the ability to modify the repository.

### Blacklist Mode

Disable specific tools while keeping others enabled:

```toml
[tools]
disabled = ["git_checkout", "TodoWrite"]
```

This disables tools that modify the working tree or filesystem while keeping all other tools available.

### Combined Mode

The `disabled` list takes precedence over `enabled`:

```toml
[tools]
enabled = ["read_files", "git_diff", "git_log", "git_checkout"]
disabled = ["git_checkout"]
# Result: read_files, git_diff, git_log are enabled
```

## Use Cases

### Read-Only Reviews

For environments where the AI agent should only analyze code without making changes:

```toml
[tools]
disabled = ["git_checkout", "TodoWrite"]
```

### Minimal Tool Set for Performance

Reduce the number of tools for faster model responses:

```toml
[tools]
enabled = [
    "read_files",
    "git_diff",
    "git_show",
    "search_file_content"
]
```

### Security-Conscious Configuration

Limit tools to those that cannot modify the repository:

```toml
[tools]
enabled = [
    "read_files",
    "list_dir",
    "find_files",
    "search_file_content",
    "git_diff",
    "git_log",
    "git_show",
    "git_blame",
    "git_status",
    "git_branch",
    "git_tag"
]
# Excludes: git_checkout, TodoWrite
```

## Performance Considerations

### Token Usage

Each tool declaration consumes tokens in the AI model's context window. Reducing the number of available tools can:

- Lower initial prompt size
- Allow more room for code content
- Potentially reduce API costs

### Tool Selection Impact

The AI agent learns which tools are available during each review. Providing fewer, more focused tools can:

- Help the agent make more efficient tool choices
- Reduce unnecessary API calls
- Speed up review completion

### Recommended Minimal Set

For most kernel patch reviews, this minimal set is sufficient:

```toml
[tools]
enabled = [
    "read_files",
    "git_diff",
    "git_log",
    "git_show",
    "search_file_content"
]
```

## Security Considerations

### Read-Write Tools

The following tools can modify the repository or filesystem:

- `git_checkout`: Changes working tree state
- `TodoWrite`: Creates/modifies TODO.md file

**Recommendation:** Disable these tools in production environments or when running untrusted patches.

### Path Traversal Protection

All file-access tools include path validation to prevent directory traversal attacks. Paths containing `..` or starting with `/` are rejected.

### Tool Execution Security

Tools execute git commands within the configured worktree directory. The worktree is isolated from the main repository to prevent accidental modifications.

## Troubleshooting

### Tool Not Available Error

If the AI agent reports a tool is not available:

1. Check your `Settings.toml` for the `[tools]` section
2. Verify the tool name is in the `enabled` list (if using whitelist mode)
3. Verify the tool name is NOT in the `disabled` list
4. Check tool names are spelled correctly (case-sensitive)

### Read-Only Tool Configuration

If you want the AI to only read code without modifications:

```toml
[tools]
# Enable all tools except those that modify state
disabled = ["git_checkout", "TodoWrite"]
```

### Minimal Configuration for Testing

For quick testing with minimal tools:

```toml
[tools]
enabled = ["read_files", "git_diff"]
```

## Future Enhancements

### Custom Tool Definitions (Planned)

Future versions may support custom tool definitions via configuration, allowing domain-specific tools without code changes:

```toml
[[tools.custom]]
name = "run_static_analyzer"
description = "Run custom static analysis tool on code"
parameters = """
{
  "type": "OBJECT",
  "properties": {
    "path": { "type": "STRING" }
  }
}
"""
command = "/usr/bin/custom-analyzer --file {path}"
```

This would enable plugin-like extensibility for enterprise deployments with specialized tooling requirements.

## References

- Tool implementation: `src/worker/tools.rs`
- Settings structure: `src/settings.rs`
- Configuration file: `Settings.toml`
