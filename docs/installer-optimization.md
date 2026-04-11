# 安装界面优化说明

## 概述

本文档说明 iApi 应用在各个平台上的安装界面优化内容。

## 优化内容

### 1. macOS DMG 安装界面

#### 背景图
- **文件**: `src-tauri/icons/dmg-background.png`
- **尺寸**: 660x400 像素（标准）+ 1320x800 像素（Retina）
- **设计风格**: 深色主题，配合品牌蓝色（#3b82f6）
- **元素**:
  - 渐变背景（#0a0e27 → #1a1f3a → #0f1428）
  - 网格背景效果
  - 发光效果
  - 中英文提示文字

#### 配置
```json
{
  "dmg": {
    "background": "icons/dmg-background.png",
    "windowSize": { "width": 660, "height": 400 },
    "appPosition": { "x": 180, "y": 170 },
    "applicationFolderPosition": { "x": 480, "y": 170 }
  }
}
```

### 2. Windows NSIS 安装程序

#### 横幅图
- **文件**: `src-tauri/icons/nsis-header.png`
- **尺寸**: 499x60 像素
- **位置**: 安装向导顶部
- **设计**: 包含应用图标和标题

#### 侧边栏图
- **文件**: `src-tauri/icons/nsis-sidebar.png`
- **尺寸**: 164x314 像素
- **位置**: 安装向导左侧
- **设计**: 包含应用图标和品牌信息

#### 配置
```json
{
  "nsis": {
    "installerIcon": "icons/icon.ico",
    "headerImage": "icons/nsis-header.png",
    "sidebarImage": "icons/nsis-sidebar.png",
    "languages": ["SimpChinese", "English"],
    "displayLanguageSelector": true
  }
}
```

### 3. 应用元数据

添加了应用的元数据信息：
- **发布者**: iApi Team
- **主页**: https://github.com/iTophua/i-api
- **简短描述**: 轻 量、快速、中文的 API 测试工具
- **详细描述**: iApi 是一款轻量级、快速、中文的 API 测试工具，支持请求发送、环境管理、集合组织、脚本执行等功能。
- **类别**: DeveloperTool

## 生成脚本

### 安装界面资源生成

运行以下命令生成所有安装界面资源：

```bash
npm run generate:installer-assets
```

这个命令会：
1. 生成 macOS DMG 背景图（标准 + Retina）
2. 生成 Windows NSIS 横幅图和侧边栏图

### 单独生成

也可以单独生成某个平台的资源：

```bash
# 仅生成 DMG 背景
node scripts/generate-dmg-background.js

# 仅生成 NSIS 图片
node scripts/generate-nsis-images.js
```

## 设计规范

### 颜色方案
- **主色**: #3b82f6（蓝色）
- **背景渐变**: #0a0e27 → #1a1f3a → #0f1428
- **文字颜色**: 
  - 主要文字: #ffffff
  - 次要文字: rgba(255, 255, 255, 0.6)
  - 提示文字: rgba(255, 255, 255, 0.4)

### 字体
- **系统字体**: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Arial, sans-serif
- **标题**: 粗体，大字号
- **正文**: 常规，中等字号

### 视觉效果
- 渐变背景
- 发光效果（glow）
- 网格背景
- 品牌图标

## 构建流程

### GitHub Actions

GitHub Actions 会自动构建所有平台的安装包：

1. **macOS**: 
   - Apple Silicon (arm64): `.dmg` 文件
   - Intel (x64): `.dmg` 文件

2. **Windows**: 
   - x64: `.msi` 和 `.exe` (NSIS) 文件

3. **Linux**: 
   - x64: `.deb` 和 `.AppImage` 文件

所有安装包都会包含优化后的安装界面。

### 本地构建

```bash
# 开发模式
npm run tauri:dev

# 生产构建
npm run tauri:build
```

## 文件结构

```
src-tauri/
├── icons/
│   ├── dmg-background.png        # macOS DMG 背景
│   ├── dmg-background@2x.png     # macOS DMG 背景 (Retina)
│   ├── nsis-header.png           # Windows NSIS 横幅
│   ├── nsis-sidebar.png          # Windows NSIS 侧边栏
│   ├── icon.icns                 # macOS 图标
│   ├── icon.ico                  # Windows 图标
│   └── ...
├── tauri.conf.json               # Tauri 配置
└── ...

scripts/
├── generate-installer-assets.js  # 统一生成脚本
├── generate-dmg-background.js    # DMG 背景生成
└── generate-nsis-images.js       # NSIS 图片生成
```

## 注意事项

1. **图片格式**: 
   - macOS DMG 背景使用 PNG 格式
   - Windows NSIS 支持 PNG 和 BMP 格式（推荐 PNG）

2. **图片尺寸**: 
   - 确保图片尺寸符合规范，避免拉伸变形
   - 提供 @2x 版本支持高分辨率屏幕

3. **颜色一致性**: 
   - 保持与应用主题颜色一致
   - 使用品牌色系

4. **多语言支持**: 
   - Windows NSIS 支持中英文
   - DMG 背景包含中英文提示

## 更新资源

如果需要更新安装界面资源：

1. 修改对应的 SVG 源文件（`src-tauri/icons/*.svg`）
2. 运行 `npm run generate:installer-assets`
3. 提交生成的 PNG 文件

## 参考资源

- [Tauri Bundler Configuration](https://tauri.app/v1/api/config#bundleconfig)
- [NSIS Documentation](https://nsis.sourceforge.io/Docs/)
- [Apple DMG Design Guidelines](https://developer.apple.com/design/human-interface-guidelines/macos/overview/themes/)
