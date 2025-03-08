import{_ as i,c as a,o as t,ag as l}from"./chunks/framework.Ckikgd0K.js";const E=JSON.parse('{"title":"代理配置","description":"","frontmatter":{},"headers":[],"relativePath":"configuration/proxy.md","filePath":"configuration/proxy.md","lastUpdated":1741430793000}'),n={name:"configuration/proxy.md"};function e(h,s,p,o,k,r){return t(),a("div",null,s[0]||(s[0]=[l(`<h1 id="代理配置" tabindex="-1">代理配置 <a class="header-anchor" href="#代理配置" aria-label="Permalink to &quot;代理配置&quot;">​</a></h1><p>代理配置部分控制 Bangumi-rs 如何通过代理服务器访问网络资源，特别是在某些网络环境下无法直接访问资源站点的情况。</p><h2 id="配置概述" tabindex="-1">配置概述 <a class="header-anchor" href="#配置概述" aria-label="Permalink to &quot;配置概述&quot;">​</a></h2><p>代理配置位于配置文件的 <code>[proxy]</code> 部分：</p><div class="language-toml vp-adaptive-theme"><button title="Copy Code" class="copy"></button><span class="lang">toml</span><pre class="shiki shiki-themes github-light github-dark vp-code" tabindex="0"><code><span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">[</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">proxy</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">]</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">enabled = </span><span style="--shiki-light:#005CC5;--shiki-dark:#79B8FF;">false</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">http = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;http://127.0.0.1:7890&quot;</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">https = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;http://127.0.0.1:7890&quot;</span></span></code></pre></div><h2 id="基本配置项" tabindex="-1">基本配置项 <a class="header-anchor" href="#基本配置项" aria-label="Permalink to &quot;基本配置项&quot;">​</a></h2><h3 id="启用状态-enabled" tabindex="-1">启用状态 (enabled) <a class="header-anchor" href="#启用状态-enabled" aria-label="Permalink to &quot;启用状态 (enabled)&quot;">​</a></h3><ul><li><strong>说明</strong>: 是否启用代理</li><li><strong>默认值</strong>: <code>false</code></li><li><strong>格式</strong>: 布尔值 (<code>true</code> 或 <code>false</code>)</li><li><strong>示例</strong>: <code>enabled = true</code></li></ul><div class="tip custom-block"><p class="custom-block-title">提示</p><p>只有在设置为 <code>true</code> 时，代理设置才会生效。这允许你在配置文件中保留代理设置，但只在需要时启用。</p></div><h3 id="http-代理-http" tabindex="-1">HTTP 代理 (http) <a class="header-anchor" href="#http-代理-http" aria-label="Permalink to &quot;HTTP 代理 (http)&quot;">​</a></h3><ul><li><strong>说明</strong>: HTTP 协议的代理服务器地址</li><li><strong>格式</strong>: URL 字符串</li><li><strong>示例</strong>: <code>http = &quot;http://127.0.0.1:7890&quot;</code></li></ul><div class="tip custom-block"><p class="custom-block-title">提示</p><p>HTTP 代理用于访问 HTTP 协议的网站和 API。</p></div><h3 id="https-代理-https" tabindex="-1">HTTPS 代理 (https) <a class="header-anchor" href="#https-代理-https" aria-label="Permalink to &quot;HTTPS 代理 (https)&quot;">​</a></h3><ul><li><strong>说明</strong>: HTTPS 协议的代理服务器地址</li><li><strong>格式</strong>: URL 字符串</li><li><strong>示例</strong>: <code>https = &quot;http://127.0.0.1:7890&quot;</code></li></ul><div class="tip custom-block"><p class="custom-block-title">提示</p><p>HTTPS 代理用于访问 HTTPS 协议的网站和 API。通常与 HTTP 代理设置为相同的地址。</p></div><h3 id="不使用代理的地址-no-proxy" tabindex="-1">不使用代理的地址 (no_proxy) <a class="header-anchor" href="#不使用代理的地址-no-proxy" aria-label="Permalink to &quot;不使用代理的地址 (no_proxy)&quot;">​</a></h3><ul><li><strong>说明</strong>: 不使用代理的地址列表</li><li><strong>格式</strong>: 字符串数组</li><li><strong>示例</strong>: <code>no_proxy = [&quot;localhost&quot;, &quot;127.0.0.1&quot;, &quot;.local&quot;]</code></li></ul><div class="tip custom-block"><p class="custom-block-title">提示</p><p>对于这些地址的请求将直接发送，不通过代理服务器。这对于访问本地服务或内部网络很有用。</p></div><h2 id="高级配置" tabindex="-1">高级配置 <a class="header-anchor" href="#高级配置" aria-label="Permalink to &quot;高级配置&quot;">​</a></h2><h3 id="代理认证-auth" tabindex="-1">代理认证 (auth) <a class="header-anchor" href="#代理认证-auth" aria-label="Permalink to &quot;代理认证 (auth)&quot;">​</a></h3><p>代理认证配置位于 <code>[proxy.auth]</code> 部分：</p><div class="language-toml vp-adaptive-theme"><button title="Copy Code" class="copy"></button><span class="lang">toml</span><pre class="shiki shiki-themes github-light github-dark vp-code" tabindex="0"><code><span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">[</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">proxy</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">.</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">auth</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">]</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">username = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;user&quot;</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">password = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;pass&quot;</span></span></code></pre></div><ul><li><p><strong>用户名 (username)</strong></p><ul><li><strong>说明</strong>: 代理服务器的认证用户名</li><li><strong>格式</strong>: 字符串</li><li><strong>示例</strong>: <code>username = &quot;proxyuser&quot;</code></li></ul></li><li><p><strong>密码 (password)</strong></p><ul><li><strong>说明</strong>: 代理服务器的认证密码</li><li><strong>格式</strong>: 字符串</li><li><strong>示例</strong>: <code>password = &quot;proxypass&quot;</code></li></ul></li></ul><div class="warning custom-block"><p class="custom-block-title">注意</p><p>代理认证信息是敏感数据，不要将其提交到版本控制系统。建议使用环境变量注入。</p></div><h3 id="socks5-代理-socks5" tabindex="-1">SOCKS5 代理 (socks5) <a class="header-anchor" href="#socks5-代理-socks5" aria-label="Permalink to &quot;SOCKS5 代理 (socks5)&quot;">​</a></h3><p>SOCKS5 代理配置位于 <code>[proxy.socks5]</code> 部分：</p><div class="language-toml vp-adaptive-theme"><button title="Copy Code" class="copy"></button><span class="lang">toml</span><pre class="shiki shiki-themes github-light github-dark vp-code" tabindex="0"><code><span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">[</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">proxy</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">.</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">socks5</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">]</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">enabled = </span><span style="--shiki-light:#005CC5;--shiki-dark:#79B8FF;">false</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">address = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;127.0.0.1:1080&quot;</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">username = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;user&quot;</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">password = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;pass&quot;</span></span></code></pre></div><ul><li><p><strong>启用状态 (enabled)</strong></p><ul><li><strong>说明</strong>: 是否启用 SOCKS5 代理</li><li><strong>默认值</strong>: <code>false</code></li><li><strong>格式</strong>: 布尔值 (<code>true</code> 或 <code>false</code>)</li></ul></li><li><p><strong>地址 (address)</strong></p><ul><li><strong>说明</strong>: SOCKS5 代理服务器地址</li><li><strong>格式</strong>: <code>&quot;主机:端口&quot;</code></li><li><strong>示例</strong>: <code>address = &quot;127.0.0.1:1080&quot;</code></li></ul></li><li><p><strong>用户名 (username)</strong></p><ul><li><strong>说明</strong>: SOCKS5 代理服务器的认证用户名</li><li><strong>格式</strong>: 字符串</li><li><strong>示例</strong>: <code>username = &quot;socksuser&quot;</code></li></ul></li><li><p><strong>密码 (password)</strong></p><ul><li><strong>说明</strong>: SOCKS5 代理服务器的认证密码</li><li><strong>格式</strong>: 字符串</li><li><strong>示例</strong>: <code>password = &quot;sockspass&quot;</code></li></ul></li></ul><h2 id="代理选择策略" tabindex="-1">代理选择策略 <a class="header-anchor" href="#代理选择策略" aria-label="Permalink to &quot;代理选择策略&quot;">​</a></h2><p>代理选择策略配置位于 <code>[proxy.strategy]</code> 部分：</p><div class="language-toml vp-adaptive-theme"><button title="Copy Code" class="copy"></button><span class="lang">toml</span><pre class="shiki shiki-themes github-light github-dark vp-code" tabindex="0"><code><span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">[</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">proxy</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">.</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">strategy</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">]</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">mode = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;all&quot;</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">fallback = </span><span style="--shiki-light:#005CC5;--shiki-dark:#79B8FF;">true</span></span></code></pre></div><ul><li><p><strong>模式 (mode)</strong></p><ul><li><strong>说明</strong>: 代理使用模式</li><li><strong>默认值</strong>: <code>&quot;all&quot;</code></li><li><strong>可选值</strong>: <ul><li><code>&quot;all&quot;</code>: 所有请求都使用代理</li><li><code>&quot;selective&quot;</code>: 仅特定站点使用代理</li></ul></li><li><strong>示例</strong>: <code>mode = &quot;selective&quot;</code></li></ul></li><li><p><strong>失败回退 (fallback)</strong></p><ul><li><strong>说明</strong>: 代理失败时是否尝试直接连接</li><li><strong>默认值</strong>: <code>true</code></li><li><strong>格式</strong>: 布尔值 (<code>true</code> 或 <code>false</code>)</li><li><strong>示例</strong>: <code>fallback = true</code></li></ul></li></ul><h3 id="选择性代理配置" tabindex="-1">选择性代理配置 <a class="header-anchor" href="#选择性代理配置" aria-label="Permalink to &quot;选择性代理配置&quot;">​</a></h3><p>当代理模式设置为 <code>&quot;selective&quot;</code> 时，可以配置哪些站点使用代理：</p><div class="language-toml vp-adaptive-theme"><button title="Copy Code" class="copy"></button><span class="lang">toml</span><pre class="shiki shiki-themes github-light github-dark vp-code" tabindex="0"><code><span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">[</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">proxy</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">.</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">sites</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">]</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">mikan = </span><span style="--shiki-light:#005CC5;--shiki-dark:#79B8FF;">true</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">bangumi_tv = </span><span style="--shiki-light:#005CC5;--shiki-dark:#79B8FF;">false</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">tmdb = </span><span style="--shiki-light:#005CC5;--shiki-dark:#79B8FF;">false</span></span></code></pre></div><ul><li><p><strong>Mikan (mikan)</strong></p><ul><li><strong>说明</strong>: 是否对 Mikan 站点使用代理</li><li><strong>默认值</strong>: <code>true</code></li><li><strong>格式</strong>: 布尔值 (<code>true</code> 或 <code>false</code>)</li></ul></li><li><p><strong>Bangumi.tv (bangumi_tv)</strong></p><ul><li><strong>说明</strong>: 是否对 Bangumi.tv 站点使用代理</li><li><strong>默认值</strong>: <code>false</code></li><li><strong>格式</strong>: 布尔值 (<code>true</code> 或 <code>false</code>)</li></ul></li><li><p><strong>TMDB (tmdb)</strong></p><ul><li><strong>说明</strong>: 是否对 TMDB 站点使用代理</li><li><strong>默认值</strong>: <code>false</code></li><li><strong>格式</strong>: 布尔值 (<code>true</code> 或 <code>false</code>)</li></ul></li></ul><h2 id="环境变量" tabindex="-1">环境变量 <a class="header-anchor" href="#环境变量" aria-label="Permalink to &quot;环境变量&quot;">​</a></h2><p>你可以使用环境变量覆盖配置文件中的代理设置：</p><ul><li><p><strong>基本配置</strong>:</p><ul><li><code>BANGUMI_PROXY_ENABLED</code>: 是否启用代理</li><li><code>BANGUMI_PROXY_HTTP</code>: HTTP 代理地址</li><li><code>BANGUMI_PROXY_HTTPS</code>: HTTPS 代理地址</li><li><code>BANGUMI_PROXY_NO_PROXY</code>: 不使用代理的地址列表，用逗号分隔</li></ul></li><li><p><strong>认证配置</strong>:</p><ul><li><code>BANGUMI_PROXY_AUTH_USERNAME</code>: 代理认证用户名</li><li><code>BANGUMI_PROXY_AUTH_PASSWORD</code>: 代理认证密码</li></ul></li><li><p><strong>SOCKS5 配置</strong>:</p><ul><li><code>BANGUMI_PROXY_SOCKS5_ENABLED</code>: 是否启用 SOCKS5 代理</li><li><code>BANGUMI_PROXY_SOCKS5_ADDRESS</code>: SOCKS5 代理地址</li><li><code>BANGUMI_PROXY_SOCKS5_USERNAME</code>: SOCKS5 认证用户名</li><li><code>BANGUMI_PROXY_SOCKS5_PASSWORD</code>: SOCKS5 认证密码</li></ul></li></ul><h2 id="常见代理类型" tabindex="-1">常见代理类型 <a class="header-anchor" href="#常见代理类型" aria-label="Permalink to &quot;常见代理类型&quot;">​</a></h2><h3 id="http-代理" tabindex="-1">HTTP 代理 <a class="header-anchor" href="#http-代理" aria-label="Permalink to &quot;HTTP 代理&quot;">​</a></h3><p>HTTP 代理是最常见的代理类型，支持 HTTP 和 HTTPS 协议：</p><div class="language-toml vp-adaptive-theme"><button title="Copy Code" class="copy"></button><span class="lang">toml</span><pre class="shiki shiki-themes github-light github-dark vp-code" tabindex="0"><code><span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">[</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">proxy</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">]</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">enabled = </span><span style="--shiki-light:#005CC5;--shiki-dark:#79B8FF;">true</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">http = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;http://127.0.0.1:7890&quot;</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">https = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;http://127.0.0.1:7890&quot;</span></span></code></pre></div><h3 id="socks5-代理" tabindex="-1">SOCKS5 代理 <a class="header-anchor" href="#socks5-代理" aria-label="Permalink to &quot;SOCKS5 代理&quot;">​</a></h3><p>SOCKS5 代理提供更通用的代理功能，支持多种协议：</p><div class="language-toml vp-adaptive-theme"><button title="Copy Code" class="copy"></button><span class="lang">toml</span><pre class="shiki shiki-themes github-light github-dark vp-code" tabindex="0"><code><span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">[</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">proxy</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">]</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">enabled = </span><span style="--shiki-light:#005CC5;--shiki-dark:#79B8FF;">true</span></span>
<span class="line"></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">[</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">proxy</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">.</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">socks5</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">]</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">enabled = </span><span style="--shiki-light:#005CC5;--shiki-dark:#79B8FF;">true</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">address = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;127.0.0.1:1080&quot;</span></span></code></pre></div><h3 id="clash-代理" tabindex="-1">Clash 代理 <a class="header-anchor" href="#clash-代理" aria-label="Permalink to &quot;Clash 代理&quot;">​</a></h3><p><a href="https://github.com/Dreamacro/clash" target="_blank" rel="noreferrer">Clash</a> 是一个流行的代理工具，默认配置如下：</p><div class="language-toml vp-adaptive-theme"><button title="Copy Code" class="copy"></button><span class="lang">toml</span><pre class="shiki shiki-themes github-light github-dark vp-code" tabindex="0"><code><span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">[</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">proxy</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">]</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">enabled = </span><span style="--shiki-light:#005CC5;--shiki-dark:#79B8FF;">true</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">http = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;http://127.0.0.1:7890&quot;</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">https = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;http://127.0.0.1:7890&quot;</span></span></code></pre></div><h3 id="v2ray-代理" tabindex="-1">V2Ray 代理 <a class="header-anchor" href="#v2ray-代理" aria-label="Permalink to &quot;V2Ray 代理&quot;">​</a></h3><p><a href="https://github.com/v2fly/v2ray-core" target="_blank" rel="noreferrer">V2Ray</a> 是另一个流行的代理工具，默认配置如下：</p><div class="language-toml vp-adaptive-theme"><button title="Copy Code" class="copy"></button><span class="lang">toml</span><pre class="shiki shiki-themes github-light github-dark vp-code" tabindex="0"><code><span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">[</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">proxy</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">]</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">enabled = </span><span style="--shiki-light:#005CC5;--shiki-dark:#79B8FF;">true</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">http = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;http://127.0.0.1:10809&quot;</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">https = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;http://127.0.0.1:10809&quot;</span></span></code></pre></div><h2 id="最佳实践" tabindex="-1">最佳实践 <a class="header-anchor" href="#最佳实践" aria-label="Permalink to &quot;最佳实践&quot;">​</a></h2><ol><li><p><strong>安全性</strong>:</p><ul><li>使用环境变量存储代理认证信息</li><li>避免在公共网络上使用不安全的代理</li><li>定期更新代理服务器和客户端</li></ul></li><li><p><strong>性能优化</strong>:</p><ul><li>选择地理位置接近资源站点的代理服务器</li><li>使用选择性代理模式，只对需要的站点启用代理</li><li>配置 <code>no_proxy</code> 避免对本地资源使用代理</li></ul></li><li><p><strong>可靠性</strong>:</p><ul><li>启用失败回退功能，确保在代理不可用时仍能访问资源</li><li>定期测试代理连接</li><li>准备备用代理服务器</li></ul></li></ol><h2 id="配置示例" tabindex="-1">配置示例 <a class="header-anchor" href="#配置示例" aria-label="Permalink to &quot;配置示例&quot;">​</a></h2><h3 id="基本-http-代理" tabindex="-1">基本 HTTP 代理 <a class="header-anchor" href="#基本-http-代理" aria-label="Permalink to &quot;基本 HTTP 代理&quot;">​</a></h3><div class="language-toml vp-adaptive-theme"><button title="Copy Code" class="copy"></button><span class="lang">toml</span><pre class="shiki shiki-themes github-light github-dark vp-code" tabindex="0"><code><span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">[</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">proxy</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">]</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">enabled = </span><span style="--shiki-light:#005CC5;--shiki-dark:#79B8FF;">true</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">http = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;http://127.0.0.1:7890&quot;</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">https = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;http://127.0.0.1:7890&quot;</span></span></code></pre></div><h3 id="带认证的代理" tabindex="-1">带认证的代理 <a class="header-anchor" href="#带认证的代理" aria-label="Permalink to &quot;带认证的代理&quot;">​</a></h3><div class="language-toml vp-adaptive-theme"><button title="Copy Code" class="copy"></button><span class="lang">toml</span><pre class="shiki shiki-themes github-light github-dark vp-code" tabindex="0"><code><span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">[</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">proxy</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">]</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">enabled = </span><span style="--shiki-light:#005CC5;--shiki-dark:#79B8FF;">true</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">http = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;http://127.0.0.1:7890&quot;</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">https = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;http://127.0.0.1:7890&quot;</span></span>
<span class="line"></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">[</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">proxy</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">.</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">auth</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">]</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">username = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;\${PROXY_USERNAME}&quot;</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">password = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;\${PROXY_PASSWORD}&quot;</span></span></code></pre></div><h3 id="选择性代理" tabindex="-1">选择性代理 <a class="header-anchor" href="#选择性代理" aria-label="Permalink to &quot;选择性代理&quot;">​</a></h3><div class="language-toml vp-adaptive-theme"><button title="Copy Code" class="copy"></button><span class="lang">toml</span><pre class="shiki shiki-themes github-light github-dark vp-code" tabindex="0"><code><span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">[</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">proxy</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">]</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">enabled = </span><span style="--shiki-light:#005CC5;--shiki-dark:#79B8FF;">true</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">http = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;http://127.0.0.1:7890&quot;</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">https = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;http://127.0.0.1:7890&quot;</span></span>
<span class="line"></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">[</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">proxy</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">.</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">strategy</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">]</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">mode = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;selective&quot;</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">fallback = </span><span style="--shiki-light:#005CC5;--shiki-dark:#79B8FF;">true</span></span>
<span class="line"></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">[</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">proxy</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">.</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">sites</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">]</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">mikan = </span><span style="--shiki-light:#005CC5;--shiki-dark:#79B8FF;">true</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">bangumi_tv = </span><span style="--shiki-light:#005CC5;--shiki-dark:#79B8FF;">false</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">tmdb = </span><span style="--shiki-light:#005CC5;--shiki-dark:#79B8FF;">false</span></span></code></pre></div><h3 id="socks5-代理-1" tabindex="-1">SOCKS5 代理 <a class="header-anchor" href="#socks5-代理-1" aria-label="Permalink to &quot;SOCKS5 代理&quot;">​</a></h3><div class="language-toml vp-adaptive-theme"><button title="Copy Code" class="copy"></button><span class="lang">toml</span><pre class="shiki shiki-themes github-light github-dark vp-code" tabindex="0"><code><span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">[</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">proxy</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">]</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">enabled = </span><span style="--shiki-light:#005CC5;--shiki-dark:#79B8FF;">true</span></span>
<span class="line"></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">[</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">proxy</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">.</span><span style="--shiki-light:#6F42C1;--shiki-dark:#B392F0;">socks5</span><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">]</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">enabled = </span><span style="--shiki-light:#005CC5;--shiki-dark:#79B8FF;">true</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">address = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;127.0.0.1:1080&quot;</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">username = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;\${SOCKS_USERNAME}&quot;</span></span>
<span class="line"><span style="--shiki-light:#24292E;--shiki-dark:#E1E4E8;">password = </span><span style="--shiki-light:#032F62;--shiki-dark:#9ECBFF;">&quot;\${SOCKS_PASSWORD}&quot;</span></span></code></pre></div>`,63)]))}const c=i(n,[["render",e]]);export{E as __pageData,c as default};
