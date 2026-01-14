# URL Scheme Registration

The `ano7` CLI uses the `ao7://` custom URL scheme to enable interactive checkin actions directly from calendar event notifications.

## URL Format

```
ao7://checkin?id=<work_item_id>&action=<action>
```

**Actions:**
- `continue` - Continue working on the task
- `blocked` - Mark as blocked and log interruption
- `stop` - Stop timer and mark task complete

## Platform-Specific Registration

### Linux (xdg-open)

Create a `.desktop` file:

```bash
# ~/.local/share/applications/ao7-handler.desktop
[Desktop Entry]
Name=AO7 URL Handler
Exec=/path/to/ano7 handle-url %u
Type=Application
NoDisplay=true
MimeType=x-scheme-handler/ao7;
```

Register it:
```bash
xdg-mime default ao7-handler.desktop x-scheme-handler/ao7
```

### macOS

Add to your app's `Info.plist` or create a helper app with:

```xml
<key>CFBundleURLTypes</key>
<array>
  <dict>
    <key>CFBundleURLSchemes</key>
    <array>
      <string>ao7</string>
    </array>
    <key>CFBundleURLName</key>
    <string>AO7 Checkin Handler</string>
  </dict>
</array>
```

### Windows

Run as Administrator:

```powershell
# Register URL protocol
New-Item -Path "HKCU:\Software\Classes\ao7" -Force
Set-ItemProperty -Path "HKCU:\Software\Classes\ao7" -Name "(Default)" -Value "URL:AO7 Protocol"
Set-ItemProperty -Path "HKCU:\Software\Classes\ao7" -Name "URL Protocol" -Value ""
New-Item -Path "HKCU:\Software\Classes\ao7\shell\open\command" -Force
Set-ItemProperty -Path "HKCU:\Software\Classes\ao7\shell\open\command" -Name "(Default)" -Value '"C:\path\to\ano7.exe" handle-url "%1"'
```

## CLI Handler (Future)

The `ano7 handle-url` command will parse the URL and execute the appropriate action:

```bash
ano7 handle-url "ao7://checkin?id=123&action=continue"
# Equivalent to: ano7 checkin 123 --action continue
```

> **Note:** The `handle-url` command is planned for a future release. 
> Currently, clicking the links in Outlook will prompt you to install a handler.
> Users can manually run `ano7 checkin` when focus blocks end.
