# Automation Guide

This guide shows how to automate `ano7` CLI workflows using platform-specific schedulers.

## Prerequisites

Before automating, always test commands with `--dry-run`:
```bash
ano7 export --ids 123,456 -o export.md --dry-run
ano7 calendar schedule 123 --duration 60 --dry-run
```

## Platform-Specific Setup

### Linux/macOS: Cron

**Edit crontab:**
```bash
crontab -e
```

**Example: Daily markdown sync at 6 AM**
```cron
# Export active work items daily
0 6 * * * cd ~/work && /usr/local/bin/ano7 export --ids $(ano7 list --state Active | grep -oP '\d+' | head -5 | tr '\n' ',') -o daily-export.md

# Log worklogs weekly (Fridays at 5 PM)
0 17 * * 5 /usr/local/bin/ano7 worklogs >> ~/logs/ano7-weekly.log
```

**Example: Hourly check-in reminder**
```cron
# Check current task status every hour during work hours
0 9-17 * * 1-5 /usr/local/bin/ano7 current || echo "No active task" | logger -t ano7
```

**Best Practices for Cron:**
- Use absolute paths (`/usr/local/bin/ano7`)
- Set environment variables if needed:
  ```cron
  PATH=/usr/local/bin:/usr/bin:/bin
  0 6 * * * ano7 export --ids 123 -o export.md
  ```
- Redirect output for debugging:
  ```cron
  0 6 * * * ano7 export --ids 123 -o export.md >> /tmp/ano7.log 2>&1
  ```

### Windows: Task Scheduler

**Create scheduled task via PowerShell:**
```powershell
# Daily export at 6 AM
$action = New-ScheduledTaskAction -Execute "ano7" -Argument "export --ids 123,456 -o C:\exports\daily.md"
$trigger = New-ScheduledTaskTrigger -Daily -At 6am
Register-ScheduledTask -Action $action -Trigger $trigger -TaskName "Ano7DailyExport" -Description "Export work items daily"
```

**Create task via GUI:**
1. Open Task Scheduler (`taskschd.msc`)
2. Create Basic Task → Name: "Ano7 Export"
3. Trigger: Daily at 6:00 AM
4. Action: Start a program
   - Program: `C:\Users\YourName\.cargo\bin\ano7.exe`
   - Arguments: `export --ids 123,456 -o C:\exports\daily.md`
5. Finish

**Example: Worklog reporting (Weekly Friday 5 PM)**
```powershell
$action = New-ScheduledTaskAction -Execute "ano7" -Argument "worklogs"
$trigger = New-ScheduledTaskTrigger -Weekly -DaysOfWeek Friday -At 5pm
Register-ScheduledTask -Action $action -Trigger $trigger -TaskName "Ano7WeeklyReport"
```

### macOS: launchd (Alternative to cron)

**Create plist file:** `~/Library/LaunchAgents/com.ano7.daily-export.plist`
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.ano7.daily-export</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/ano7</string>
        <string>export</string>
        <string>--ids</string>
        <string>123,456</string>
        <string>-o</string>
        <string>/Users/yourname/exports/daily.md</string>
    </array>
    <key>StartCalendarInterval</key>
    <dict>
        <key>Hour</key>
        <integer>6</integer>
        <key>Minute</key>
        <integer>0</integer>
    </dict>
</dict>
</plist>
```

**Load the task:**
```bash
launchctl load ~/Library/LaunchAgents/com.ano7.daily-export.plist
```

## Common Automation Workflows

### 1. Daily Work Item Sync
```bash
#!/bin/bash
# sync-workitems.sh

# Export current active items
ano7 export --ids $(ano7 list --state Active | awk '{print $1}' | tr '\n' ',') -o /tmp/active-items.md

# Import any changes (use --validate first!)
if [ -f ~/workitems/changes.md ]; then
    ano7 import ~/workitems/changes.md --validate
    if [ $? -eq 0 ]; then
        ano7 import ~/workitems/changes.md
        mv ~/workitems/changes.md ~/workitems/imported-$(date +%Y%m%d).md
    fi
fi
```

### 2. Automated Check-in Reminder
```bash
#!/bin/bash
# remind-checkin.sh

# Check if there's an active task older than 2 hours
CURRENT_TASK=$(ano7 current)
if [ $? -eq 0 ]; then
    echo "Active task found. Time to check in?"
    # Send notification (macOS example)
    osascript -e 'display notification "Check in on your focus block?" with title "Ano7 Reminder"'
fi
```

### 3. End-of-Day Cleanup
```bash
#!/bin/bash
# eod-cleanup.sh

# Stop any running timers
ano7 stop 2>/dev/null

# Export today's worklogs
ano7 worklogs > ~/logs/worklog-$(date +%Y%m%d).txt

# Clear state (or checkin as Complete)
# ano7 checkin Complete
```

## Error Handling

**Exit Codes:**
- `0`: Success
- `1`: General error (work item not found, API failure)
- >1: Other errors

**Example with error handling:**
```bash
#!/bin/bash

ano7 export --ids 123 -o export.md
EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
    echo "Export successful"
else
    echo "Export failed with code $EXIT_CODE" | logger -t ano7-error
    # Send alert
    mail -s "Ano7 export failed" admin@example.com <<< "Check logs"
fi
```

## Security Best Practices

**1. Credentials:**
- PAT stored in system keyring (automatic with Phase 6)
- Never hardcode credentials in scripts
- Use `ano7 config` to verify setup

**2. File Permissions:**
```bash
# Make scripts executable but readable only by you
chmod 700 ~/scripts/ano7-sync.sh
```

**3. Logging:**
```bash
# Log to dedicated file with rotation
ano7 export --ids 123 -o export.md >> ~/logs/ano7.log 2>&1

# Rotate logs weekly
# Add to cron:
0 0 * * 0 find ~/logs -name "ano7*.log" -mtime +7 -delete
```

## Testing Automation

**Always test with dry-run first:**
```bash
# Test daily export
ano7 export --ids 123,456 -o /tmp/test.md --dry-run

# Test calendar scheduling
ano7 calendar schedule 123 --duration 60 --dry-run
```

**Manual test run:**
```bash
# Run the cron job manually
bash -c "$(crontab -l | grep ano7 | head -1 | awk '{$1=$2=$3=$4=$5=""; print $0}')"
```

## Troubleshooting

**Cron jobs not running:**
- Check syslog: `grep CRON /var/log/syslog`
- Verify permissions: `ls -la /usr/local/bin/ano7`
- Test command in terminal first

**Windows Task Scheduler issues:**
- Check Task History in Task Scheduler GUI
- Run task manually to test
- Ensure user has proper permissions

**Environment issues:**
- Set `HOME` variable in cron if needed
- Verify config file location: `~/.ao-no-out7ook/config.toml`
