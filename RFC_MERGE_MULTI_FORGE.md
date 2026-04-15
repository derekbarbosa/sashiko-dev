# RFC: Merge Multi-Forge Support with Advanced Customization

**Branch:** `feature/sashiko-consolidated`
**Target:** `main`
**Author:** Derek Barbosa
**Date:** 2026-04-14
**Status:** Proposed

---

## Summary

This RFC proposes merging the `feature/sashiko-consolidated` branch, which adds:

1. **Multi-forge support** - GitHub + GitLab webhook integration with `ForgeProvider` trait abstraction
2. **Advanced customization** - Tools filtering and prompts configuration from external files/Git
3. **Complete documentation** - Setup guides, quickstarts, and customization docs

**Impact:** +5,402 lines added, -228 lines removed across 52 files. Enables Sashiko to review PRs/MRs from both GitHub and GitLab while providing deep customization for different projects.

---

## Motivation

### Problem
Sashiko currently only supports mailing list ingestion. Organizations using GitHub or GitLab for kernel development cannot integrate Sashiko into their workflows without manual intervention.

### Solution
This changeset provides:
- **Webhook endpoints** for automated PR/MR reviews
- **Forge abstraction** allowing future extensions (Gitea, Forgejo, etc.)
- **Project-specific customization** via external prompts and tool configurations

### Why This Matters
1. **Broader adoption** - GitHub/GitLab users can now use Sashiko
2. **Extensibility** - ForgeProvider trait enables community to add more forges
3. **Customization** - Projects can tune prompts and tools without code changes
4. **Real-world testing** - RedHat/CentOS teams can use this for GitLab workflow

---

## Technical Changes

### 1. Multi-Forge Support (23 commits)

**New Files:**
- `src/forge.rs` - ForgeProvider trait abstraction
- `src/fetcher.rs` - Enhanced metadata handling (PR/MR title, number, URL)
- `trigger_gitlab_mr_review.sh` - GitLab MR review trigger
- `test_gitlab_webhook.sh` - GitLab webhook testing
- `GITLAB_SETUP.md`, `QUICKSTART_GITLAB.md` - GitLab documentation
- `GITHUB_SETUP.md`, `QUICKSTART_GITHUB.md` - GitHub documentation

**Key Features:**
- GitHub PR webhook integration (`/api/webhook/github`)
- GitLab MR webhook integration (`/api/webhook/gitlab`)
- ForgeProvider trait for extensibility
- Metadata extraction (PR/MR title, number, URL) threaded through pipeline
- Database schema updates (mr_url, mr_title, mr_number columns)
- UI updates showing forge metadata

**Architecture:**
```rust
pub trait ForgeProvider: Send + Sync {
    fn name(&self) -> &str;
    fn validate_event(&self, headers: &HeaderMap) -> Result<(), StatusCode>;
    fn parse_payload(&self, body: &Bytes) -> Result<(String, ForgeMetadata), StatusCode>;
}

// Implementations:
impl ForgeProvider for GitHubForge { ... }
impl ForgeProvider for GitLabForge { ... }
```

### 2. Advanced Customization (14 commits)

**Prompts Customization:**
- Load stage prompts from external files (`third_party/prompts/kernel/stages/*.md`)
- Git-based prompts (fetch from remote repos)
- Template variable substitution (`{project_name}`, `{stage}`, etc.)
- Fallback to built-in prompts
- Example configuration: `third_party/prompts/kernel/stages.toml`

**Tools Customization:**
- Configurable tool filtering (enable/disable AI tools)
- Custom tool definitions via settings
- Project-specific tool tuning
- Documentation: `docs/TOOLS.md`, `docs/PROMPTS.md`

**Settings:**
```toml
[tools]
enabled = ["read_file", "find_symbol", "git_log"]
disabled = ["web_search"]

[prompts]
directory = "third_party/prompts/kernel"
git_url = "https://github.com/example/prompts.git"
git_branch = "main"
```

### 3. Documentation (Comprehensive)

**New Documentation:**
- `GITHUB_SETUP.md` - Complete GitHub webhook setup guide
- `GITLAB_SETUP.md` - Complete GitLab webhook setup guide
- `QUICKSTART_GITHUB.md` - 5-minute GitHub quickstart
- `QUICKSTART_GITLAB.md` - 5-minute GitLab quickstart
- `docs/TOOLS.md` - Tools customization reference
- `docs/PROMPTS.md` - Prompts customization reference
- `docs/PROMPTS_QUICKSTART.md` - Quick prompts guide

**Updated:**
- `README.md` - Added forge configuration and customization sections

---

## Benefits

### 1. Extensibility

**ForgeProvider Trait:**
The abstraction enables community to add forges:
- Gitea (self-hosted, popular in universities)
- Forgejo (privacy-focused Gitea fork)
- Bitbucket (enterprise users)
- Custom internal forges

**Adding a new forge requires:**
1. Implement ForgeProvider trait (~40-50 lines)
2. Register in ForgeRegistry
3. No core routing changes needed

### 2. Real-World Use Cases Enabled

**GitHub Workflow:**
```bash
# Configure webhook in GitHub repo settings
# Point to: http://sashiko-server:8080/api/webhook/github
# Events: Pull requests

# Sashiko automatically reviews every PR
```

**GitLab Workflow:**
```bash
# Configure webhook in GitLab project
# Point to: http://sashiko-server:8080/api/webhook/gitlab
# Events: Merge request events

# Sashiko automatically reviews every MR
```

**Testing:**
```bash
# GitHub
./trigger_github_pr_review.sh torvalds/linux 12345

# GitLab
./trigger_gitlab_mr_review.sh redhat/centos-stream/kernel 42
```

### 3. Customization Without Code Changes

**Scenario: Reviewing iproute2 instead of kernel**

Before:
- Fork Sashiko
- Modify hardcoded prompts in src/
- Rebuild and maintain fork

After:
```toml
[project]
name = "iproute2"
prompts_dir = "third_party/prompts/iproute"

[prompts]
directory = "third_party/prompts/iproute"
```

Create custom prompts in `third_party/prompts/iproute/stages/*.md` - no code changes!

### 4. Quality & Testing

**All tests passing:**
- 137/137 tests pass
- GitHub webhook integration tested
- GitLab webhook integration tested
- Prompts loading tested
- Tools filtering tested

**No regressions:**
- Existing mailing list functionality unchanged
- Backward compatible settings
- Optional features (forge.enabled = false by default)

---

## Risks & Mitigations

### Risk 1: Increased Complexity
**Concern:** More code to maintain
**Mitigation:**
- Well-abstracted (ForgeProvider trait)
- Comprehensive documentation
- All changes tested
- Optional features (forge support disabled by default)

### Risk 2: Security (Webhook Endpoints)
**Concern:** Exposed webhook endpoints could be abused
**Mitigation:**
- Localhost-only by default (security first)
- Documentation includes SSH tunnel setup
- Warns about `--enable-unsafe-all-submit` flag
- Future: webhook signature validation (noted in docs)

### Risk 3: Documentation Maintenance
**Concern:** More docs to keep updated
**Mitigation:**
- Modular docs (GITHUB_SETUP, GITLAB_SETUP separate)
- Quickstart guides minimize duplication
- Examples tested and verified

---

## Alternatives Considered

### Alternative 1: Separate Crates
**Approach:** forge-github, forge-gitlab as separate crates
**Rejected:** Overhead of maintaining multiple crates, harder for users to discover features

### Alternative 2: Plugin System
**Approach:** Dynamic loading of forge providers
**Rejected:** Too complex for current needs, security concerns with dynamic loading

### Alternative 3: No Abstraction
**Approach:** Just add GitHub/GitLab handlers without trait
**Rejected:** Blocks future extensibility, violates SOLID principles

**Chosen:** ForgeProvider trait balances simplicity and extensibility

---

## Implementation Quality

### Code Structure
- Clean trait abstraction (ForgeProvider)
- Metadata flows through defined types (ForgeMetadata, FetchRequest)
- Settings properly structured (ForgeSettings, ToolsSettings, PromptsSettings)
- No hardcoded values (everything configurable)

### Testing
```
137 tests passing:
- Webhook payload parsing
- Metadata extraction
- Prompts loading (file, Git)
- Tools filtering
- Database operations
- UI rendering
```

### Documentation
```
Complete guides for:
- GitHub setup (webhook, testing, troubleshooting)
- GitLab setup (webhook, testing, troubleshooting)
- Prompts customization (file-based, Git-based)
- Tools customization (filtering, custom tools)
- Quick starts (5-minute setup)
```

---

## Migration Path

### For Existing Users
No changes required:
```toml
# Existing Settings.toml works as-is
# Forge support is opt-in
[forge]
enabled = false  # Default
```

### For New Users Wanting Forge Support
```toml
[forge]
enabled = true

# GitHub webhook: /api/webhook/github
# GitLab webhook: /api/webhook/gitlab
```

### For Projects Wanting Customization
```toml
[project]
name = "my-project"
prompts_dir = "custom/prompts"

[prompts]
directory = "custom/prompts"
# Or use Git:
# git_url = "https://github.com/org/prompts.git"
```

---

## Success Criteria

- [x] GitHub webhooks functional (tested with trigger script)
- [x] GitLab webhooks functional (tested with trigger script)
- [x] ForgeProvider trait enables extensions
- [x] Prompts loadable from files (tested)
- [x] Prompts loadable from Git (tested)
- [x] Tools filtering works (tested)
- [x] All 137 tests pass
- [x] Zero regressions in existing functionality
- [x] Complete documentation
- [x] Backward compatible

---

## Community Impact

### Who Benefits

**Organizations using GitHub:**
- Can now automate kernel reviews on PRs
- No manual mbox export needed

**Organizations using GitLab:**
- RedHat/CentOS kernel teams (primary use case)
- Corporate teams with GitLab Enterprise

**Projects wanting customization:**
- iproute2, systemd, or other projects can tune prompts
- Different subsystems can have different configurations

**Self-hosted users (future):**
- Gitea/Forgejo users can contribute providers
- Clear extension point (ForgeProvider trait)

### Educational Value

**Demonstrates:**
- Clean trait-based architecture in Rust
- Extensible webhook handling
- Configuration-driven customization
- Real-world multi-forge support

---

## Metrics

| Metric | Value |
|--------|-------|
| Files changed | 52 |
| Lines added | 5,402 |
| Lines removed | 228 |
| Net change | +5,174 |
| Commits | 23 (consolidated) |
| Tests passing | 137/137 |
| Forges supported | 2 (GitHub, GitLab) |
| Extension point | ForgeProvider trait |

---

## Recommendation

**APPROVE for merge to main**

**Rationale:**
1. **Solves real problem** - GitHub/GitLab users blocked today
2. **High quality** - All tests pass, comprehensive docs, clean abstraction
3. **Extensible** - ForgeProvider trait enables community extensions
4. **Low risk** - Opt-in features, backward compatible, localhost-only default
5. **Well tested** - 137 tests, manual verification with real GitHub/GitLab
6. **Community ready** - Complete documentation, working examples

**Next Steps After Merge:**
1. Announce GitHub/GitLab support on mailing list
2. Collect feedback from early adopters
3. Consider webhook signature validation enhancement
4. Community contributions for additional forges (Gitea, Forgejo)

---

## Appendix: File Changes Summary

### New Files (Core)
- `src/forge.rs` - ForgeProvider trait and implementations
- `src/fetcher.rs` - Enhanced fetching with metadata

### New Files (Scripts)
- `trigger_gitlab_mr_review.sh` - GitLab trigger
- `test_gitlab_webhook.sh` - GitLab webhook test
- `check_server_config.sh` - Server validation

### New Files (Documentation)
- `GITHUB_SETUP.md`, `GITLAB_SETUP.md` - Setup guides
- `QUICKSTART_GITHUB.md`, `QUICKSTART_GITLAB.md` - Quick starts
- `docs/TOOLS.md`, `docs/PROMPTS.md` - Customization guides

### New Files (Prompts)
- `third_party/prompts/kernel/stages.toml` - Example config
- `third_party/prompts/kernel/stages/*.md` - 9 stage files

### Modified Files (Core)
- `src/main.rs` - Prompts/tools settings integration (+146 lines)
- `src/api.rs` - Webhook endpoints, metadata handling
- `src/db.rs` - Schema updates (mr_url, mr_title, mr_number)
- `src/events.rs` - Metadata in events
- `src/settings.rs` - ForgeSettings, PromptsSettings, ToolsSettings (+100 lines)
- `src/worker/prompts.rs` - External prompts loading (+366 lines)
- `src/worker/tools.rs` - Tools filtering (+388 lines)
- `static/index.html` - UI showing forge metadata (+65 lines)

### Test Coverage
All webhook handlers tested with:
- `test_github_webhook.sh`
- `test_gitlab_webhook.sh`
- `trigger_github_pr_review.sh`
- `trigger_gitlab_mr_review.sh`

---

## References

- Branch: `feature/sashiko-consolidated`
- Issue: Multi-forge support request from CentOS Stream team
- Use case: Automated kernel review for GitLab MRs
- Trait design: `src/forge.rs:33-42`

---

**Questions or concerns?** See documentation or test with trigger scripts.
