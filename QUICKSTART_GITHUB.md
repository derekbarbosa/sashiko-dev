# GitHub Integration Quickstart

Get Sashiko reviewing GitHub Pull Requests in under 5 minutes.

## Prerequisites

- Rust toolchain installed
- Git repository (local or remote)
- LLM API key (Gemini or Claude)

## Step 1: Configure LLM Provider

Set your API key:
```bash
# For Gemini
export LLM_API_KEY="your-gemini-api-key"

# For Claude
export ANTHROPIC_API_KEY="your-claude-api-key"
```

Verify `Settings.toml` has your LLM provider configured:
```toml
[ai]
provider = "gemini"  # or "claude"
model = "gemini-3.1-pro-preview"  # or "claude-sonnet-4-6"
```

## Step 2: Start Sashiko Server

```bash
cargo run --release
```

You should see output like:
```
Server running on http://127.0.0.1:8080
```

**Verify it's running:**
```bash
curl http://localhost:8080/
```

## Step 3: Test with a Pull Request

### Option A: Test with Real GitHub PR

Use the trigger script to review any public GitHub PR:
```bash
./trigger_github_pr_review.sh torvalds/linux 12345
```

Replace `torvalds/linux` with `owner/repo` and `12345` with a PR number.

### Option B: Test with Synthetic Webhook

Send a test webhook payload to verify the endpoint:
```bash
./test_github_webhook.sh
```

### Option C: Test with Local Commits

Review your own local changes:
```bash
# Review last 3 commits
cargo run --bin sashiko-cli -- submit HEAD~3..HEAD
```

## Step 4: Monitor Progress

Open your browser to the web UI:
```
http://localhost:8080/
```

You'll see:
- Review queue status
- Current review progress
- Completed reviews with findings

## Step 5: Configure Webhook (Optional)

To automatically review PRs when they're opened on GitHub:

1. Go to your GitHub repository → Settings → Webhooks
2. Add webhook:
   - **URL:** `http://your-server:8080/api/webhook/github`
   - **Content type:** `application/json`
   - **Events:** Pull requests only
3. For local development, use ngrok or SSH tunnel:
   ```bash
   # Using ngrok
   ngrok http 8080
   # Use the ngrok URL in webhook configuration
   ```

See [GITHUB_SETUP.md](GITHUB_SETUP.md) for detailed webhook setup.

## Next Steps

### Customize Review Settings

Edit `Settings.toml`:
```toml
[review]
concurrency = 20          # Parallel reviews
worktree_dir = "review_trees"
timeout_seconds = 7200    # 2 hours
```

### Review Your Own Patches

Set your development repository:
```toml
[git]
repository_path = "/home/user/src/linux"
```

Then submit ranges:
```bash
sashiko-cli submit HEAD~5..HEAD
```

### Enable Production Mode

For accepting webhooks from GitHub:
```bash
cargo run --release -- --enable-unsafe-all-submit
```

**⚠️ Warning:** Only use this behind a firewall or reverse proxy!

## Troubleshooting

### Server won't start
- Check port 8080 is not in use: `lsof -i :8080`
- Verify Settings.toml is valid TOML format
- Check git repository path exists

### API key errors
- Verify environment variable is set: `echo $LLM_API_KEY`
- Check API key is valid (test with provider's API)
- Ensure provider matches configured model

### PR not fetched
- Verify git repository has the commit
- Check GitHub API rate limits
- Ensure repository is accessible (public or credentials configured)

### No review starts
- Check logs for specific errors: `RUST_LOG=info cargo run`
- Verify LLM provider is reachable
- Ensure worktree_dir has write permissions

## What's Next?

- Read [GITHUB_SETUP.md](GITHUB_SETUP.md) for production deployment
- Check [README.md](README.md) for advanced features
- Join the mailing list: sashiko@lists.linux.dev

## Need Help?

- Mailing list: sashiko@lists.linux.dev ([archive](https://lore.kernel.org/sashiko))
- GitHub Issues: [Report bugs or request features](https://github.com/sashiko-dev/sashiko/issues)
