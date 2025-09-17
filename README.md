### Logitech K380 Bluetooth Keyboard F-n keys mode switcher for MacOS

F-n keys are working in non-standard mode by default on this keyboard.
I do not want to mess with "Logi Options" just for switching so I made this little app.

_It's a Rust rewrite of C based version here: https://github.com/faust93/k380-macos_

Check prebuilt binaries for Apple M & Intel platforms in **bin** folder:
```
k380-macos_darwin-x86_64
k380-macos_darwin-aarch64
```

#### How to use
$ sudo ./k380-macos_darwin-x86_64 -f on|off

- `-f on|off` : 手動で F キーの標準モードを切り替え
- `--auto on|off` : 常駐して接続を監視し、自動でシーケンスを送信
- `-l` : 接続中の Logitech HID デバイス一覧を表示

自動モード例:

```
$ sudo ./k380-macos_darwin-x86_64 --auto on
```

Logitech Options を使わなくても、キーボード接続時に F キーを標準動作へ切り替えられます。

#### 自動起動 (launchd) 設定例
launchd で root 権限だけを使う LaunchDaemon (system ドメイン) だと、GUI セッション外のため `IOHIDManagerOpen` が `0xe00002e2 (kIOReturnNotPermitted)` を返し接続時の制御に失敗します。ログインユーザの LaunchAgent から `sudo` 経由で実行する方法が動作確認済みです。

1. バイナリを `/usr/local/bin/k380-macos` など任意のパスへ配置し、`chmod 755` にしておきます。
2. `sudo visudo -f /etc/sudoers.d/k380` で以下を追加し、パスワード無しで常駐コマンドを許可します。

   ```
   your_username ALL=(root) NOPASSWD: /usr/local/bin/k380-macos --auto on
   ```

   ユーザ名やコマンドラインは環境に合わせて調整してください。`sudo` で別オプションを使う場合はその分も追加が必要です。
3. `~/Library/LaunchAgents/com.example.k380-auto.plist` を作成します。所有者は必ず対象ユーザとし（例: `sudo chown your_username:staff ~/Library/LaunchAgents/com.example.k380-auto.plist`）、内容は次のとおりです。

   ```
   <?xml version="1.0" encoding="UTF-8"?>
   <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
   <plist version="1.0">
   <dict>
       <key>Label</key>
       <string>com.example.k380-auto</string>
       <key>ProgramArguments</key>
       <array>
           <string>/usr/bin/sudo</string>
           <string>/usr/local/bin/k380-macos</string>
           <string>--auto</string>
           <string>on</string>
       </array>
       <key>RunAtLoad</key>
       <true/>
       <key>KeepAlive</key>
       <true/>
       <key>StandardOutPath</key>
       <string>/tmp/k380-auto.log</string>
       <key>StandardErrorPath</key>
       <string>/tmp/k380-auto.log</string>
   </dict>
   </plist>
   ```

4. バイナリ（または `sudo` を呼び出すターミナル）を「システム設定 → プライバシーとセキュリティ → 入力監視」に追加し、チェックを入れます。設定後は一度ログアウト→再ログインしてください。
5. root で次のコマンドを実行して登録します。

   ```
   sudo launchctl bootstrap gui/$(id -u your_username) ~/Library/LaunchAgents/com.example.k380-auto.plist
   sudo launchctl kickstart -k gui/$(id -u your_username)/com.example.k380-auto
   ```

   停止時は `sudo launchctl bootout gui/$(id -u your_username) ~/Library/LaunchAgents/com.example.k380-auto.plist` を使います。ログは `/tmp/k380-auto.log` に追記されます。
