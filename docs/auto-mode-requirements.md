# 自動実行モード要件・設計メモ

## 機能要件
- K380 が Bluetooth で接続された瞬間に F キー切替シーケンスを送信すること。
- 実行中にキーボードが切断された場合は待機状態に戻り、再接続で再度シーケンスを送ること。
- CLI から `--auto on|off` を指定して希望する F キー状態を選択できること。
- 手動実行時 (`-f on|off`) と挙動が一貫し、失敗時には終了コードおよび標準エラーで原因を通知すること。
- 常駐時の CPU / メモリ消費が無視できる水準 (イベント駆動でポーリングなし) であること。

## 非機能要件
- IOKit API を利用する処理は macOS 13 以降で動作確認する。
- 失敗時リトライは 1 秒以上のバックオフを設け、ログを標準出力へ残す。
- launchd から起動された場合も対話不要で完結する。

## IOKit による接続監視方式
1. `IOHIDManagerCreate(kCFAllocatorDefault, kIOHIDOptionsTypeNone)` でマネージャを生成する。
2. `IOServiceMatching(kIOUSBDeviceClassName)` ベースではなく、`IOHIDManagerSetDeviceMatching` に VendorID / ProductID を指定した辞書を設定する。
3. `IOHIDManagerRegisterDeviceMatchingCallback` と `IOHIDManagerRegisterDeviceRemovalCallback` で接続/切断イベントを受信する。
4. 登録後に `IOHIDManagerScheduleWithRunLoop(CFRunLoopGetCurrent(), kCFRunLoopDefaultMode)` を呼び出し、`CFRunLoopRun` でイベントループを開始する。
5. 接続コールバック内で `hidapi::HidApi::new()` を生成し、`api.open(HID_VENDOR_ID_LOGITECH, HID_DEVICE_ID_K380)` でデバイスハンドルを取得する。
6. 取得に成功したら既存の `write` ロジックを呼び出し、切替結果をログに出力する。
7. 切断コールバックでは内部状態を更新し、次回接続で再度実行できるようにする。
8. エラーが発生した場合は run loop を継続しつつ、再接続時にリトライする。

## 残課題
- hidapi と IOKit を同時に利用する際のライフサイクル管理 (グローバル化 vs 毎回生成) のベストプラクティス調査。
- テスト戦略 (モックデバイス or 実機のみ) の検討。
