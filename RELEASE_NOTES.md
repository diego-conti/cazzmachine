# Cazzmachine v0.2.1 Release Notes

## What's New

### Multithreaded Consumption (🎉 NEW!)
- **Heat map style ThreadSlider**: 8 color-coded bars (blue→cyan→teal→green→yellow→orange→pink→red)
- **Dynamic labels**: single_cazz (1), double_cazz (2), multi_cazz (3-4), high_cazz (5-7), full_cazz (8)
- **Plus/minus buttons**: Easy thread count adjustment
- **Color-coded status messages**: With multipliers for thread count
- **Thread notifications**: [cazz_thread N] format
- **System status**: Always visible even with zero items

### Android Background Service
- **Foreground service**: Keeps CrawlScheduler running when app is backgrounded
- **Persistent downloads**: Buffer maintains 20+ items even when app is closed
- **Android Doze mode**: Solves the problem of suspended tokio runtime
- **Notification**: Shows "Cazzmachine is running" status

### User Interface Improvements
- **Startup notification**: "Cazzmachine ready. Go back to work; I've got your doomscrolling covered"
- **Thread notifications**: Clear indication of which thread is consuming
- **Enhanced status messages**: More informative and colorful

## Installation

### Linux
```bash
# Install the Debian package
sudo dpkg -i cazzmachine_0.2.1_amd64.deb
# Fix any dependency issues
sudo apt-get install -f
```

### Android
1. **Install the signed APK**: `adb install cazzmachine-background-service.apk`
2. **Grant notification permission** when prompted
3. **Enjoy background doomscrolling!**

The app will now automatically download content in the background, even when you close it.

**Note**: The Android APK needs to be signed with your keystore password. Run:
```bash
cd /home/diego/dati/workspace/cazzmachine
jarsigner -keystore /home/diego/dati/workspace/cazzmachine/cazzmachine-keystore.jks -signedjar cazzmachine-background-service.apk app-universal-release-unsigned.apk cazzmachine-key
```
Then install with:
```bash
adb install cazzmachine-background-service.apk
```

## Technical Details

### Android Background Service
- **Service name**: `CrawlService`
- **Permissions**: WAKE_LOCK, FOREGROUND_SERVICE, POST_NOTIFICATIONS
- **Foreground notification**: "Cazzmachine is running - Downloading content in the background"
- **Rust backend**: CrawlScheduler runs as tokio task in background

### Thread Management
- **Buffer size**: 20 items × thread count
- **Color coding**: Blue (1) → Cyan (2) → Teal (3) → Green (4) → Yellow (5) → Orange (6) → Pink (7) → Red (8)
- **Labels**: single_cazz, double_cazz, multi_cazz, high_cazz, full_cazz
- **Multipliers**: Thread count affects consumption speed and buffer size

## Bug Fixes

- **Android background downloads**: Fixed issue where downloads stopped when app was closed
- **Thread notifications**: Now show clear thread identification
- **Status visibility**: System status always visible regardless of item count

## Performance Improvements

- **Background efficiency**: Service runs with low priority to save battery
- **Buffer management**: Automatic scaling based on thread count
- **Notification optimization**: Minimal impact on system resources

## Known Issues

- **Gradle deprecation warnings**: Build uses some deprecated Gradle features (will be fixed in next release)
- **Android TV support**: Leanback launcher is available but not fully tested

## Support

If you encounter any issues with the background service:
1. Check that the notification is visible in your notification shade
2. Ensure the app has notification permissions
3. Verify the APK is properly signed
4. Check device battery optimization settings (may need to exclude Cazzmachine)

## Acknowledgments

Built with:
- [Tauri](https://tauri.app/) - Rust-based desktop framework
- [React](https://react.dev/) - UI library
- [Tailwind CSS](https://tailwindcss.com/) - Styling
- [Zustand](https://github.com/pmndrs/zustand) - State management
- [Kotlin](https://kotlinlang.org/) - Android service implementation

---

**Cazzmachine v0.2.1** - Now with 100% more background doomscrolling!
Let the machine toil while you work!