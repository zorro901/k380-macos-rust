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

