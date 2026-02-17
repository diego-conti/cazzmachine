# Cazzmachine v0.2.2 Release Checklist

## ✅ COMPLETED

### Code Implementation
- Android foreground service implemented (CrawlService.kt)
- Permissions added to AndroidManifest.xml
- MainActivity.kt modified to start service on launch
- Notification strings added
- Version bumped to 0.2.2
- Code committed and pushed to GitHub

### Build Output
**Unsigned APK:** `src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk` (72MB)

---

## 🔄 REMAINING - Requires User Action

### 1. Sign the APK ⚠️ REQUIRED - DO THIS FIRST

**The unsigned APK must be signed with your private key before distribution.**

**Old signed APK (OUTDATED):**
- Location: `/home/diego/dati/workspace/cazzmachine/cazzmachine-signed.apk`
- Date: Feb 11, 2026, 22:27
- **DO NOT USE** - This is v0.2.1 without the foreground service

**New unsigned APK (CONTAINS FOREGROUND SERVICE):**
- Location: `src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk`
- Size: 72MB
- Built: Feb 12, 2026, 09:16
- **THIS IS v0.2.2** - Contains the Android foreground service

#### Signing Command:

```bash
cd /home/diego/dati/workspace/cazzmachine

# Sign the APK (replace with YOUR keystore details):
jarsigner -keystore /path/to/your/keystore.jks \
  -signedjar cazzmachine-signed.apk \
  src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk \
  your-alias-name

# Example (adjust paths for your system):
# jarsigner -keystore ~/keys/cazzmachine.jks \
#   -signedjar cazzmachine-signed.apk \
#   src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk \
#   cazzmachine
```

#### After Signing:

```bash
# Verify the APK is signed:
jarsigner -verify -verbose cazzmachine-signed.apk

# Should show: "jar verified"

# Install on device for testing:
adb install cazzmachine-signed.apk
```

### 2. Create GitHub Release ⚠️ BLOCKED

Once the APK is signed, create a new GitHub release:

**Release Details:**
- Tag: `v0.2.2`
- Title: "Cazzmachine v0.2.2: Android Background Downloads"
- Assets to upload:
  - `cazzmachine-signed.apk` (signed version of the new APK)
  - Optional: Linux packages (deb, rpm, AppImage) - already built for v0.2.1

**Release Notes:**
```
## What's New in v0.2.2

### Android Background Downloads
- Implemented foreground service for background crawling
- Persistent notification while app is running
- Content continues downloading even when app is closed
- Automatic buffer maintenance (20+ items)

### Bug Fixes
- Fixed: CrawlScheduler now runs continuously in background
- Fixed: Android Doze mode no longer suspends content downloads

### Technical Changes
- Added WAKE_LOCK, FOREGROUND_SERVICE permissions
- Created CrawlService.kt for background operation
- Updated MainActivity.kt for service startup
```

### 3. Device Testing ⚠️ BLOCKED

Once the signed APK is installed on your Android device:

**Test Procedure:**
1. Open the app
2. Verify persistent notification appears: "Cazzmachine is working"
3. Close the app (press home button)
4. Verify notification remains visible
5. Wait 2-15 minutes (depending on throttle level)
6. Open the app again
7. Verify new content was downloaded in background

**Expected Results:**
- ✅ Notification visible while app is closed
- ✅ Buffer maintains 20+ items
- ✅ New content appears after crawl interval
- ✅ No excessive battery drain

---

## 📋 Quick Reference

**GitHub:** https://github.com/diego-conti/cazzmachine
**Current Branch:** main
**Latest Commit:** dc74835 - Release v0.2.2: Android foreground service for background downloads

**APK Locations:**
- Unsigned (needs signing): `src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk`
- To be replaced: `cazzmachine-signed.apk`

---

## 🚀 Next Steps

1. **Sign the APK** with your private key
2. **Replace** `cazzmachine-signed.apk` with the signed version
3. **Create GitHub release** v0.2.2
4. **Test on device** and verify background downloads work
