# Discord Rich Presence Assets

Discord Rich Presence cannot read these local files directly. Upload these images to the Discord
application asset list with the matching asset keys before shipping a bundled build.

If an asset key is missing in the Discord Developer Portal, Discord shows a broken `?` image. The
files are kept in this repository as the source assets for that upload step; they are not loaded
from the bundled app at runtime.

| State | File | Asset key |
| --- | --- | --- |
| Work running | `pomoshumai_work_running.png` | `pomoshumai_work_running` |
| Break running | `pomoshumai_break_running.png` | `pomoshumai_break_running` |
| Work idle | `pomoshumai_work_idle.png` | `pomoshumai_work_idle` |
| Break idle | `pomoshumai_break_idle.png` | `pomoshumai_break_idle` |
| Paused | `pomoshumai_paused.png` | `pomoshumai_paused` |
