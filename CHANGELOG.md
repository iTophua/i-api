# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-05-12

### Added

- **快捷键自定义配置** - 设置面板新增快捷键标签页，支持可视化录制、重置和禁用快捷键
- **新增全局快捷键** - Ctrl+W 关闭标签页、Ctrl+Shift+I 导入 cURL、Ctrl+Shift+E 打开环境管理、Ctrl+Shift+H 打开历史记录、Ctrl+Shift+S 打开设置
- **cURL 导入增强** - 导入时自动解析 URL 中的查询参数并填充到 params 表格
- **URL 与参数表格双向同步** - 在 URL 中输入参数自动同步到下方表格，表格中修改参数也会同步回 URL
- **设置面板记忆标签页** - 自动记住上次打开的设置标签页

### Fixed

- 修复 URL 参数输入时禁用参数被错误保留的问题（输入 `?te` 时不再残留多余的临时参数）
- 修复用户手动禁用的参数在 URL 编辑时被意外移除的问题

## [0.1.0] - 2026-04-11

### Added

- HTTP 请求发送与响应查看
- 环境变量管理（增删改查、复制、快速切换）
- 集合与文件夹组织（创建、重命名、删除、拖拽排序）
- 前置/后置脚本执行（JavaScript）
- cURL 命令导入
- OpenAPI/HAR 文件导入
- Postman 集合导入/导出
- 代码生成器（多语言请求代码）
- 快捷键支持
- 深色/浅色主题
- CI/CD 自动化构建（GitHub Actions）
