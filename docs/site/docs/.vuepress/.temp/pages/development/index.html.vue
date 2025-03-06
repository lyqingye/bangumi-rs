<template><div><h1 id="开发指南" tabindex="-1"><a class="header-anchor" href="#开发指南"><span>开发指南</span></a></h1>
<h2 id="项目结构" tabindex="-1"><a class="header-anchor" href="#项目结构"><span>项目结构</span></a></h2>
<div class="language-text line-numbers-mode" data-highlighter="prismjs" data-ext="text"><pre v-pre><code><span class="line">bangumi-rs/</span>
<span class="line">├── src/           # 后端 Rust 源码</span>
<span class="line">├── web/           # 前端 Vue 3 源码</span>
<span class="line">├── docs/          # 项目文档</span>
<span class="line">├── tests/         # 测试代码</span>
<span class="line">└── scripts/       # 辅助脚本</span>
<span class="line"></span></code></pre>
<div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h2 id="技术架构" tabindex="-1"><a class="header-anchor" href="#技术架构"><span>技术架构</span></a></h2>
<h3 id="后端架构" tabindex="-1"><a class="header-anchor" href="#后端架构"><span>后端架构</span></a></h3>
<p><img src="/images/dashboard.png" alt="后端架构"></p>
<ul>
<li><strong>运行时</strong>: tokio 异步运行时</li>
<li><strong>数据存储</strong>: sea-orm + SQLite</li>
<li><strong>错误处理</strong>: anyhow 和 thiserror</li>
<li><strong>日志系统</strong>: tracing</li>
<li><strong>序列化</strong>: serde</li>
<li><strong>API</strong>: RESTful + WebSocket</li>
</ul>
<h3 id="前端架构" tabindex="-1"><a class="header-anchor" href="#前端架构"><span>前端架构</span></a></h3>
<p><img src="/images/settings.png" alt="前端架构"></p>
<ul>
<li><strong>框架</strong>: Vue 3 + TypeScript</li>
<li><strong>UI</strong>: Vuetify 3</li>
<li><strong>构建</strong>: Vite</li>
<li><strong>路由</strong>: Vue Router</li>
<li><strong>HTTP</strong>: Axios</li>
<li><strong>实时通信</strong>: WebSocket</li>
</ul>
<h2 id="开发规范" tabindex="-1"><a class="header-anchor" href="#开发规范"><span>开发规范</span></a></h2>
<h3 id="代码风格" tabindex="-1"><a class="header-anchor" href="#代码风格"><span>代码风格</span></a></h3>
<ul>
<li>后端遵循 Rust 标准代码风格</li>
<li>前端使用 ESLint + Prettier</li>
<li>提交前运行 <code v-pre>make lint</code> 检查代码风格</li>
</ul>
<h3 id="git-提交规范" tabindex="-1"><a class="header-anchor" href="#git-提交规范"><span>Git 提交规范</span></a></h3>
<p>提交信息格式：</p>
<div class="language-text line-numbers-mode" data-highlighter="prismjs" data-ext="text"><pre v-pre><code><span class="line">&lt;type>(&lt;scope>): &lt;subject></span>
<span class="line"></span>
<span class="line">&lt;body></span>
<span class="line"></span>
<span class="line">&lt;footer></span>
<span class="line"></span></code></pre>
<div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><p>类型（type）：</p>
<ul>
<li>feat: 新功能</li>
<li>fix: 修复</li>
<li>docs: 文档</li>
<li>style: 格式</li>
<li>refactor: 重构</li>
<li>test: 测试</li>
<li>chore: 构建</li>
</ul>
<h3 id="测试要求" tabindex="-1"><a class="header-anchor" href="#测试要求"><span>测试要求</span></a></h3>
<ul>
<li>新功能必须包含单元测试</li>
<li>修复 bug 必须包含相关测试用例</li>
<li>运行 <code v-pre>make test</code> 执行所有测试</li>
</ul>
<h2 id="贡献指南" tabindex="-1"><a class="header-anchor" href="#贡献指南"><span>贡献指南</span></a></h2>
<ol>
<li>Fork 项目</li>
<li>创建特性分支</li>
<li>提交变更</li>
<li>推送到分支</li>
<li>创建 Pull Request</li>
</ol>
<h2 id="调试技巧" tabindex="-1"><a class="header-anchor" href="#调试技巧"><span>调试技巧</span></a></h2>
<h3 id="后端调试" tabindex="-1"><a class="header-anchor" href="#后端调试"><span>后端调试</span></a></h3>
<div class="language-bash line-numbers-mode" data-highlighter="prismjs" data-ext="sh"><pre v-pre><code><span class="line"><span class="token comment"># 开发模式运行</span></span>
<span class="line"><span class="token function">make</span> run-dev</span>
<span class="line"></span>
<span class="line"><span class="token comment"># 查看日志</span></span>
<span class="line"><span class="token function">tail</span> <span class="token parameter variable">-f</span> logs/bangumi.log</span>
<span class="line"></span></code></pre>
<div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="前端调试" tabindex="-1"><a class="header-anchor" href="#前端调试"><span>前端调试</span></a></h3>
<div class="language-bash line-numbers-mode" data-highlighter="prismjs" data-ext="sh"><pre v-pre><code><span class="line"><span class="token comment"># 启动开发服务器</span></span>
<span class="line"><span class="token builtin class-name">cd</span> web</span>
<span class="line"><span class="token function">npm</span> run dev</span>
<span class="line"></span>
<span class="line"><span class="token comment"># 构建并查看分析报告</span></span>
<span class="line"><span class="token function">npm</span> run build -- <span class="token parameter variable">--report</span></span>
<span class="line"></span></code></pre>
<div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h2 id="api-文档" tabindex="-1"><a class="header-anchor" href="#api-文档"><span>API 文档</span></a></h2>
<p>API 文档使用 Swagger 生成，访问 http://localhost:3001/swagger-ui/ 查看。</p>
</div></template>


