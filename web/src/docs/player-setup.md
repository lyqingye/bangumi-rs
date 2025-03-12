# 播放器设置指南

## VLC 播放器设置 (macOS)

1. 首先确保已经安装了 VLC 播放器
2. 打开终端 (Terminal)
3. 复制粘贴以下命令：

```bash
# 注册 VLC URL scheme
defaults write org.videolan.vlc URLhandler -dict-add vlc YES

# 更新 Launch Services 数据库
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -kill -r -domain local -domain system -domain user

# 重启 Launch Services
killall Finder
killall Dock
```

4. 重启 VLC 播放器

如果上述方法不起作用，可以尝试以下替代方案：

1. 打开 Finder
2. 找到 VLC.app（通常在应用程序文件夹中）
3. 右键点击 VLC.app，选择"显示包内容"
4. 导航到 Contents/Info.plist
5. 确认文件中包含以下内容（如果没有，需要手动添加）：

```xml
<key>CFBundleURLTypes</key>
<array>
    <dict>
        <key>CFBundleURLName</key>
        <string>VLC media player</string>
        <key>CFBundleURLSchemes</key>
        <array>
            <string>vlc</string>
        </array>
    </dict>
</array>
```

## IINA 播放器设置

IINA 播放器通常会自动注册 URL scheme，无需额外设置。

## Infuse 播放器设置

Infuse 播放器也会自动注册 URL scheme，无需额外设置。

## 故障排除

如果播放器仍然无法通过 URL scheme 打开：

1. 检查系统偏好设置 > 安全性与隐私 > 通用，确保允许从"App Store 和被认可的开发者"下载的应用
2. 尝试重新安装播放器
3. 如果使用的是最新版 macOS，可能需要在"系统设置 > 隐私与安全性"中允许应用程序打开外部链接