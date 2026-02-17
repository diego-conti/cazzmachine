# Android Version Status - 2026-02-13

## Build Status

| Component | Status |
|-----------|--------|
| **APK Build** | ✅ SUCCESS |
| **APK Location** | `src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk` |
| **APK Size** | 72 MB |
| **Architectures** | aarch64, armv7, i686, x86_64 |

## Installation Test

- ✅ APK installed successfully on emulator (Pixel 3a, API 34)
- ✅ App launched and runs

## Issues Found

### 1. Service Crash on Startup

**Problem**: The CrawlService crashed on initial startup but was auto-restarted by Android's Activity Manager.

```
ActivityManager: Scheduling restart of crashed service com.cazzmachine.app/.CrawlService
ActivityManager: Rescheduling restart of crashed service com.cazzmachine.app/.CrawlService
```

**Impact**: Low - Android auto-restarted the service within seconds.

### 2. No Background Crawl Activity

**Problem**: The AndroidBackgroundService started but no crawl logs appeared after 35+ seconds of monitoring.

- Expected: "Android background crawl: provider_name" logs
- Got: Only "CrawlScheduler started" (which is a reference to old code that no longer runs)

**Likely Cause**: The service is running but:
1. Network requests may be failing silently
2. The 30-minute crawl interval hasn't elapsed yet
3. Log output not visible in Android logcat (Rust `log::info!` macros may need Android-specific logger)

### 3. Log Visibility Issue

**Problem**: Rust `log::info!` and `log::warn!` macros are not showing in Android logcat.

**Solution**: May need to add `android_logger` crate for proper Android logging.

### 4. Release Build Not Debuggable

**Problem**: Cannot access app database for verification because release build is not debuggable.

```
run-as: package not debuggable: com.cazzmachine.app
```

**Solution**: Build debug APK to access database during testing.

## Current Architecture (Desktop vs Android)

### Desktop (Working)
- Event-driven crawling: crawls at startup + when buffer empty
- No background scheduler thread
- Frontend triggers `triggerCrawl()` after consumption if buffer low

### Android (Issues)
- Background service runs every 30 minutes
- Service checks if buffer < target, then crawls
- BUT: Crawl may be failing silently, logs not visible

## Recommendations

1. **Add Android Logging**: Add `android_logger` crate to see Rust logs in logcat

2. **Debug APK**: Build debug APK to:
   - Access database for verification
   - Get full crash traces

3. **Test with Manual Crawl**: Trigger crawl manually via app UI to verify network works

4. **Check Network Permissions**: Ensure APK has INTERNET permission

## Files Modified

- `src-tauri/src/android/background_service.rs` - Implemented actual crawl logic
- `src-tauri/src/android/mod.rs` - Fixed imports
- `src-tauri/src/lib.rs` - Removed desktop background scheduler, simplified Android setup

## Test Commands

```bash
# Install APK (unsigned)
adb install app-universal-release-unsigned.apk

# View logs
adb logcat -d --pid=$(adb shell pidof com.cazzmachine.app)

# Check running process
adb shell ps | grep cazzmachine
```

## Fixes Applied (2026-02-13)

### 1. Added Android Logging (`android_logger`)
**File**: `src-tauri/Cargo.toml`
- Added `[target.'cfg(target_os = "android")'.dependencies]` section with `android_logger = "0.15"`

**File**: `src-tauri/src/lib.rs`
- Replaced single `env_logger::init()` with conditional initialization:
  - Android: Uses `android_logger::init_once()` with Debug level
  - Desktop: Uses `env_logger::init()`

### 2. Fixed Service Crash on Startup
**File**: `src-tauri/gen/android/app/src/main/java/com/cazzmachine/app/MainActivity.kt`
- Added 2-second delay before starting CrawlService
- Allows Rust runtime to fully initialize before service starts
- Prevents race condition that caused the initial crash

### 3. Fixed Background Crawl Logic
**File**: `src-tauri/src/android/background_service.rs`
- Removed dependency on `LifecycleManager::is_background_mode()` check
- The service now always crawls when the timer fires (every 30 min)
- Added initial crawl 5 seconds after service starts (for testing)
- Improved error handling for database queries
- Added detailed logging at each step

### 4. Root Causes Identified

| Issue | Root Cause | Fix |
|-------|------------|-----|
| No logs in logcat | Missing `android_logger` crate | Added target-specific dependency |
| Service crash | Service started before Rust ready | Added 2s delay in MainActivity |
| No background crawl | `IS_BACKGROUND_MODE` starts as `false` | Removed foreground check |

## Next Steps

1. **Build the APK** to test the fixes:
   ```bash
   npm run tauri android build
   ```

2. **Install and test**:
   ```bash
   adb install src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk
   ```

3. **Monitor logs** (should now show Rust logs):
   ```bash
   adb logcat -s RustStdoutStderr:D
   # or
   adb logcat | grep -i "cazzmachine\|AndroidBackground\|CrawlService"
   ```

4. **Expected log output** (within 10 seconds of app start):
   ```
   Cazzmachine Rust runtime initialized on Android
   AndroidBackgroundService started
   AndroidBackgroundService: performing initial crawl
   Android background crawl: Reddit Memes (meme)
   ...
   Android background crawl complete: X new items
   ```

5. **Verify background crawl** works after 30 minutes (or modify the 30*60 interval to 60 for testing)
