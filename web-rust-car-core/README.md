# web-rust-car-core

> Web 遥控玩具车主控核心，使用 ESP32C3 芯片和 Rust 技术栈。

## 研发

1. 下载 ESP-IDF 开发 SDK 和工具，通过 gitee 加速。
```bash
git clone https://gitee.com/EspressifSystems/esp-gitee-tools
cd esp-gitee-tools

# https://gitee.com/EspressifSystems/esp-gitee-tools/blob/master/docs/README-jihu-mirror.md
./jihu-mirror.sh set
cd ..
git clone --recursive https://github.com/espressif/esp-idf.git
cd esp-idf
git checkout v5.1 # 切换到 5.1 版本，不要用最新的 master 分支。

# https://gitee.com/EspressifSystems/esp-gitee-tools/blob/master/docs/README-submodule-update.md
cd ../esp-gitee-tools
export EGT_PATH=$(pwd)
cd ../esp-idf
$EGT_PATH/submodule-update.sh # 将 submodules 也同步为 5.1 版本

$EGT_PATH/install.sh # 安装 esp-idl sdk 工具
```
2. 安装 rustup 及 rust nightly
3. 修改 `.cargo/config.toml` 文件（非常重要！！）
```toml
# 将下面的 IDF_PATH 修改为第 1 步 clone 下来的 esp-idf 所在目录的绝对路径
IDF_PATH = { value = "/home/xiaoge/esp/esp-idf" } 
```
4. 对于在 windows 系统下的 WSL2，需要将 USB 串口传入 WSL2，详见：https://learn.microsoft.com/en-us/windows/wsl/connect-usb
5. 在项目目录执行 `cargo run`
