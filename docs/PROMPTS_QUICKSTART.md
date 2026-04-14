# Prompts Customization Quick Start

This guide gets you started customizing Sashiko's review prompts in under 5 minutes.

## Quick Examples

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

### 6. Use Remote Prompts (Team Sharing)

Share prompts via Git repository. In `Settings.toml`:

```toml
[prompts]
directory = "git://github.com/myteam/kernel-review-prompts.git"
```

Sashiko will clone and cache the repository automatically.

## Common Customization Patterns

### Security-Focused Configuration

```toml
[prompts]
directory = "./security-prompts"

[prompts.variables]
threat_model = "untrusted network input"
focus_area = "memory safety, integer overflows, TOCTOU"
compliance = "CWE Top 25"
```

### Subsystem-Specific Review

```toml
[prompts.variables]
subsystem = "networking"
maintainer_concerns = "packet processing efficiency, protocol compliance"
reference_code = "net/core/skbuff.c"
```

### Minimalist Review (Fast)

In `stages.toml`:
```toml
# Enable only critical stages
[[stages]]
number = 2
enabled = true

[[stages]]
number = 4
enabled = true

[[stages]]
number = 6
enabled = true

[[stages]]
number = 9
enabled = true

# Disable others
[[stages]]
number = 1
enabled = false

[[stages]]
number = 3
enabled = false

[[stages]]
number = 5
enabled = false

[[stages]]
number = 7
enabled = false

[[stages]]
number = 8
enabled = false
```

## Troubleshooting

### Variables not substituting
Check syntax:
```markdown
✅ Correct: {{project_name}}
❌ Wrong:   {project_name}
❌ Wrong:   $project_name
```

### Custom prompts not loading
```bash
# Check path exists
ls -la ./my-prompts

# Check Settings.toml
grep prompts Settings.toml

# Check logs
grep "Loading stages" sashiko.log
```

### Stage disabled error
If you see "Stage N is disabled", check `stages.toml`:
```toml
[[stages]]
number = N
enabled = true  # Make sure this is true
```

## Next Steps

- Read the full guide: [docs/PROMPTS.md](PROMPTS.md)
- See all stages: `ls third_party/prompts/kernel/stages/`
- Check supporting files: `ls third_party/prompts/kernel/`
- Learn about tools: [docs/TOOLS.md](TOOLS.md)

## Tips

1. **Test changes** - Run a review after modifying prompts to verify behavior
2. **Version control** - Keep custom prompts in git for team sharing
3. **Start small** - Customize one stage at a time
4. **Use variables** - Avoid duplicating text across prompts
5. **Document changes** - Add comments explaining customizations

## Need Help?

- Full documentation: [docs/PROMPTS.md](PROMPTS.md)
- Mailing list: sashiko@lists.linux.dev
- GitHub issues: https://github.com/sashiko-dev/sashiko/issues
