# GitHub Integration Setup

Complete guide for configuring Sashiko to automatically review GitHub Pull Requests via webhooks.

## Prerequisites

- Sashiko running in server mode (daemon)
- Server accessible from GitHub (public URL or tunneling solution like ngrok)
- Admin rights on GitHub repository (to configure webhooks)
- Sashiko configured with valid LLM API credentials

## Configuration

### 1. Configure Server Settings

Sashiko's webhook endpoints are available by default when running the daemon. The server configuration in `Settings.toml`:

```toml
[server]
host = "127.0.0.1"
port = 8080
```

**Security Note:** By default, Sashiko only accepts webhook requests from `localhost` for security. To accept webhooks from GitHub's servers, you have two options:

**Option A: SSH Tunnel (Recommended for testing)**
```bash
# On your development machine
ssh -R 9080:localhost:8080 your-public-server.com

# GitHub webhook URL would be:
# http://your-public-server.com:9080/api/webhook/github
```

**Option B: Allow all submit (Use with caution)**
```bash
# Start Sashiko with the unsafe flag
cargo run --release -- --enable-unsafe-all-submit
```

**⚠️ WARNING:** Option B allows anyone who can reach your server to trigger reviews. Only use this in trusted networks or behind authentication layers (reverse proxy with auth, firewall rules, etc.).

### 2. Set up GitHub Webhook

1. Navigate to your repository on GitHub
2. Go to **Settings** → **Webhooks** → **Add webhook**
3. Configure the webhook:
   - **Payload URL:** `http://your-server:8080/api/webhook/github`
     - Replace `your-server` with your actual server address
     - Use port `8080` (or your configured server port)
   - **Content type:** `application/json`
   - **Secret:** Leave empty (signature validation not yet implemented)
   - **Which events would you like to trigger this webhook?**
     - Select: **Let me select individual events**
     - Check: ✓ **Pull requests**
     - Uncheck all other events
   - **Active:** ✓ Enabled

4. Click **Add webhook**

### 3. Verify Webhook Delivery

After creating the webhook, GitHub will immediately send a test `ping` event:

1. In GitHub webhook settings, click on your newly created webhook
2. Navigate to the **Recent Deliveries** tab
3. You should see a `ping` event with a green checkmark
4. If the delivery failed, check:
   - Server is running: `curl http://localhost:8080/`
   - Firewall allows incoming connections
   - URL is correct and accessible from the internet

### 4. Test the Integration

**Option A: Use the test script**
```bash
./test_github_webhook.sh
```

This sends a synthetic webhook payload to verify the endpoint is working.

**Option B: Trigger a real PR review**
```bash
./trigger_github_pr_review.sh owner/repo 123
```

This fetches real PR data from GitHub's API and triggers a review.

**Option C: Open a real Pull Request**

The most authentic test - simply open a new PR on your repository. Sashiko should:
1. Receive the webhook
2. Log the PR details
3. Queue the review
4. Fetch the commits
5. Start the AI review process

Check the web UI at `http://localhost:8080/` to see the review progress.

## Troubleshooting

### Webhook not received

**Symptoms:** No logs in Sashiko output when PR is opened

**Solutions:**
- Verify server is running: `curl http://localhost:8080/`
- Check firewall allows incoming connections on port 8080
- Verify webhook URL is accessible from internet (use ngrok or similar for testing)
- Check GitHub webhook delivery status in repository settings

### Permission denied (403 Forbidden)

**Symptoms:** GitHub shows delivery failed with 403 status

**Cause:** Sashiko's default security blocks non-localhost requests

**Solutions:**
1. **Recommended:** Use SSH tunnel or reverse proxy from localhost
2. **Quick test:** Run with `--enable-unsafe-all-submit` flag
3. **Production:** Set up reverse proxy with TLS and authentication

### Webhook received but review not starting

**Symptoms:** Logs show webhook received but no review begins

**Solutions:**
- Check LLM API key is configured: `echo $LLM_API_KEY`
- Verify git repository is accessible: check `git.repository_path` in `Settings.toml`
- Look for errors in Sashiko logs
- Check that the commit hash exists in your configured repository

### Review fails immediately

**Symptoms:** Review status shows "failed" in web UI

**Solutions:**
- Check Sashiko logs for specific error messages
- Verify git repository has the referenced commits
- Ensure LLM provider is accessible and API key is valid
- Check disk space in `worktree_dir` directory

## Security Considerations

**⚠️ IMPORTANT:** GitHub webhook signature validation is not yet implemented.

For production deployments:

1. **Use HTTPS:** Set up a reverse proxy with TLS
   ```nginx
   # Example nginx config
   server {
       listen 443 ssl;
       server_name sashiko.example.com;

       ssl_certificate /path/to/cert.pem;
       ssl_certificate_key /path/to/key.pem;

       location /api/webhook/ {
           proxy_pass http://localhost:8080;
           proxy_set_header X-Real-IP $remote_addr;
       }
   }
   ```

2. **Implement webhook secrets:** Future enhancement - see GitHub's [webhook security guide](https://docs.github.com/en/webhooks/using-webhooks/validating-webhook-deliveries)

3. **Network isolation:** Run Sashiko on private network and use SSH tunneling or VPN

4. **Rate limiting:** Configure reverse proxy or firewall to prevent abuse

## Advanced Configuration

### Custom Port

Modify `Settings.toml`:
```toml
[server]
host = "0.0.0.0"  # Listen on all interfaces
port = 9080       # Custom port
```

Then update webhook URL: `http://your-server:9080/api/webhook/github`

### Multiple Repositories

Sashiko can handle webhooks from multiple repositories. Simply add the same webhook configuration to each repository you want to monitor.

**Note:** All repositories must be accessible from the git repository configured in `Settings.toml` or Sashiko will fail to fetch the commits.

## Webhook Payload Reference

Sashiko processes the following fields from GitHub's `pull_request` webhook:

```json
{
  "action": "opened",
  "pull_request": {
    "number": 123,
    "title": "Fix memory leak in driver",
    "html_url": "https://github.com/owner/repo/pull/123",
    "head": {
      "sha": "abc123..."
    },
    "base": {
      "sha": "def456..."
    }
  }
}
```

Supported actions: `opened`, `reopened`, `synchronize` (new commits pushed)

## See Also

- [QUICKSTART_GITHUB.md](QUICKSTART_GITHUB.md) - Quick start guide for rapid setup
- [README.md](README.md) - Main project documentation
- [Settings.toml](Settings.toml) - Configuration reference

## Getting Help

- **Mailing List:** sashiko@lists.linux.dev ([lore archive](https://lore.kernel.org/sashiko))
- **GitHub Issues:** [Report bugs or request features](https://github.com/sashiko-dev/sashiko/issues)
