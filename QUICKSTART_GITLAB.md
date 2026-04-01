# Quick Start: GitLab Integration

## Step 1: Enable Forge Mode

Edit `Settings.toml`:

```toml
[forge]
enabled = true
```

Restart Sashiko.

## Step 2: Verify Server is Ready

```bash
./check_server_config.sh
```

Expected output: "✓ Server is ready for GitLab webhooks!"

## Step 3: Test with a Specific MR

```bash
# Replace 123 with an actual MR number
./trigger_gitlab_mr_review.sh 123
```

This script will:
1. Fetch MR details from GitLab API
2. Send a simulated webhook to Sashiko
3. Show the response

## Step 4: Configure GitLab Webhook (Optional)

For automatic reviews on every MR update:

1. Go to: https://gitlab.com/redhat/centos-stream/src/kernel/centos-stream-10/-/settings/integrations

2. Add webhook:
   - **URL:** `http://localhost:9080/api/webhook/gitlab`
   - **Trigger:** Merge request events
   - **SSL:** Disable (using HTTP)

3. Test the webhook using GitLab's test feature

## Monitoring Reviews

View reviews at: http://localhost:9080/

## Troubleshooting

| Issue | Solution |
|-------|----------|
| 403 Forbidden | Set `forge.enabled = true` and restart |
| 400 Bad Request | Check MR number exists and is accessible |
| Reviews not appearing | Check Sashiko logs, verify git repo is accessible |
| Webhook test fails | Ensure server is reachable from GitLab |

## Full Documentation

See `GITLAB_SETUP.md` for complete setup instructions.
