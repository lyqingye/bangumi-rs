<div align="center">

# 🌟 Bangumi-rs

</div>

<div align="center">

[![Lint](https://github.com/lyqingye/bangumi-rs/actions/workflows/checks.yml/badge.svg)](https://github.com/lyqingye/bangumi-rs/actions/workflows/checks.yml) [![Release](https://github.com/lyqingye/bangumi-rs/actions/workflows/release.yml/badge.svg)](https://github.com/lyqingye/bangumi-rs/actions/workflows/release.yml) [![Docker](https://github.com/lyqingye/bangumi-rs/actions/workflows/docker.yml/badge.svg)](https://github.com/lyqingye/bangumi-rs/actions/workflows/docker.yml) ![Version](https://img.shields.io/github/v/release/lyqingye/bangumi-rs.svg?color=blue&logo=github) ![License](https://img.shields.io/badge/license-MIT-green.svg)

</div>

<div align="center">

<a href="https://lyqingye.github.io/bangumi-rs/">
  <img src="https://img.shields.io/badge/官方文档-4285F4?style=for-the-badge&logo=google-docs&logoColor=white&labelColor=4285F4">
</a>
<a href="https://lyqingye.github.io/bangumi-rs/quickstart/">
  <img src="https://img.shields.io/badge/快速开始-34A853?style=for-the-badge&logo=clockify&logoColor=white">
</a>
<a href="https://github.com/lyqingye/bangumi-rs/issues">
  <img src="https://img.shields.io/badge/问题反馈-EA4335?style=for-the-badge&logo=github&logoColor=white">
</a>

</div>

<br>

<div align="center">
<img src="docs/public/screenshot/home.png" width="90%" alt="home" style="box-shadow: 0 12px 32px rgba(0, 0, 0, 0.7); border-radius: 12px; margin: 20px 0;" />
</div>

<br>
<p align="center">
<b>Bangumi-rs</b> 是一款动漫追番工具，支持订阅番剧、智能下载、在线播放等功能。<br>
采用 <b>Rust</b> 后端 + <b>Vue 3</b> 前端开发，高效稳定，界面美观。
</p>
<br>

<h2 align="center">✨ 功能特性</h2>

<table>
<tr>
<td width="50%">

<div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 30px; margin: 30px 0;">
  <div>
    <h3>🎯 番剧订阅</h3>
    <ul>
      <li>✅ 支持订阅/取消订阅番剧</li>
      <li>✅ 可配置分辨率、字幕组、字幕语言过滤条件</li>
      <li>✅ 支持自定义订阅更新间隔</li>
    </ul>
  </div>

  <div>
    <h3>📥 智能下载</h3>
    <ul>
      <li>✅ 支持剧集偏移</li>
      <li>✅ 自动选择最佳种子</li>
      <li>✅ 支持用户手动选择要下载的剧集种子</li>
      <li>✅ 支持 115 网盘离线下载</li>
      <li>✅ 下载失败后会自动尝试其它种子</li>
    </ul>
  </div>

  <div>
    <h3>🌐 资源站点</h3>
    <ul>
      <li>✅ Mikan (<a href="https://mikanani.me/">https://mikanani.me/</a>)</li>
      <li>🔜 更多站点支持中...</li>
    </ul>
  </div>

  <div>
    <h3>🎬 在线播放</h3>
    <ul>
      <li>✅ 支持 IINA, Infuse 播放器在线播放</li>
    </ul>
  </div>

  <div>
    <h3>🔍 资源解析</h3>
    <ul>
      <li>✅ 采用传统方式解析文件名</li>
      <li>✅ 采用 ChatGPT 进行解析</li>
      <li>✅ 支持多个 API 服务提供商: SiliconFlow、OpenAI、DeepSeek</li>
    </ul>
  </div>

  <div>
    <h3>📚 元数据管理</h3>
    <ul>
      <li>✅ 自动获取番剧信息</li>
      <li>✅ 支持从多个数据源获取(TMDB、Bangumi.tv、Mikan)</li>
      <li>✅ 支持手动刷新元数据</li>
      <li>✅ 支持剧集、海报墙、封面等信息显示</li>
    </ul>
  </div>
</div>

<div style="margin: 30px 0;">
  <h3 align="center">🔔 通知提醒</h3>
  <p align="center">✅ 支持 Telegram 通知</p>
</div>
</table>

<div align="center">
  <h2>🖼️ 精彩截图</h2>
  <p>优雅的界面设计，流畅的用户体验</p>
</div>

<div align="center">
<table>
<tr>
<td><img src="docs/public/screenshot/detail.png" alt="详情页" style="border-radius: 12px; box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);"/></td>
<td><img src="docs/public/screenshot/subscribe.png" alt="订阅页" style="border-radius: 12px; box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);"/></td>
</tr>
<tr>
<td><img src="docs/public/screenshot/settings.png" alt="设置页" style="border-radius: 12px; box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);"/></td>
<td><img src="docs/public/screenshot/download.png" alt="下载页" style="border-radius: 12px; box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);"/></td>
</tr>
</table>
</div>

<br>

<div align="center">
  <h2>🤝 贡献</h2>
  <p>我们欢迎各种形式的贡献，一起让 Bangumi-rs 变得更好！</p>
  
  <a href="https://github.com/lyqingye/bangumi-rs/pulls">
    <img src="https://img.shields.io/badge/提交PR-2ea44f?style=for-the-badge&logo=git&logoColor=white">
  </a>
  <a href="https://github.com/lyqingye/bangumi-rs/issues/new">
    <img src="https://img.shields.io/badge/报告问题-1d76db?style=for-the-badge&logo=github&logoColor=white">
  </a>
  <a href="https://github.com/lyqingye/bangumi-rs/discussions">
    <img src="https://img.shields.io/badge/参与讨论-8250df?style=for-the-badge&logo=github&logoColor=white">
  </a>
</div>

<br>

<div align="center">
  <h2>📜 许可证</h2>
  <p>本项目采用 MIT 许可证</p>
  
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/MIT License-yellow?style=for-the-badge&logo=license&logoColor=white">
  </a>
</div>

