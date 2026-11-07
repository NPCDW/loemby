# USER require
1. 安卓版本 adb 传输速度较慢，打包 apk 时，尽量打一个小包，然后极致压缩后传输并安装
2. 当前服务器为 docker 容器，由 [android-Dockerfile]{./android-Dockerfile} 创建而来，如果服务器有需要更改的地方，同步修改 android-Dockerfile
3. 安卓功能尽量与桌面端的功能复用，而不是新实现一个差不多的功能
4. 打包 apk 后，如果用 adb 没有检测到实机设备，不要直接跳过，先向我弹窗询问，选项为：a.是否已连接设备,重新检测，b.不连接设备,跳过
5. 如果在编码过程中有需要注意需要记忆的事项，写到下方 AGENT memory 区域

# AGENT memory
