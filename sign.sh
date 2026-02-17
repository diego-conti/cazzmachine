apksigner sign --ks cazzmachine.keystore --out cazzmachine-signed.apk /home/diego/dati/workspace/cazzmachine/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk
gh release upload $(gh release list --limit 1 --json name | jq -r '.[0].name') cazzmachine-signed.apk --clobber
