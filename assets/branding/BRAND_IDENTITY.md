# ClipType brand identity

ClipType uses two related visual assets:

- **Primary application mark:** a blue clipboard-to-input composition representing clipboard text flowing into a typing target.
- **Notification-area mark:** a simplified blue circular badge containing a white clipboard and insertion caret, optimized for 16–32 pixel Windows tray rendering.

## Windows resource rules

- Resource ID `1` is the executable/application icon.
- Resource ID `2` is the notification-area icon.
- The executable icon must provide frames for 16, 20, 24, 32, 48, 64, 128, and 256 pixels.
- The notification-area icon must keep a strong silhouette and avoid motion ribbons, shadows, and tiny keyboard details at small sizes.
- The tray uses the embedded ClipType icon and falls back to the Windows stock application icon only for a development build whose resource compilation failed.

## State presentation

The icon itself does not blink or change to red during normal operation. Enabled, busy, cancelled, and failure states are communicated through the content-free tray menu, tooltip, and notifications.
