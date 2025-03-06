<template><div><h1 id="快速开始" tabindex="-1"><a class="header-anchor" href="#快速开始"><span>快速开始</span></a></h1>
<h2 id="环境要求" tabindex="-1"><a class="header-anchor" href="#环境要求"><span>环境要求</span></a></h2>
<ul>
<li>Docker (推荐)</li>
<li>或者
<ul>
<li>Rust 1.75+</li>
<li>Node.js 18+</li>
<li>MySQL 8.0+</li>
</ul>
</li>
</ul>
<h2 id="docker-部署-推荐" tabindex="-1"><a class="header-anchor" href="#docker-部署-推荐"><span>Docker 部署（推荐）</span></a></h2>
<ol>
<li>克隆项目</li>
</ol>
<div class="language-bash line-numbers-mode" data-highlighter="prismjs" data-ext="sh"><pre v-pre><code><span class="line"><span class="token function">git</span> clone https://github.com/lyqingye/bangumi-rs.git</span>
<span class="line"><span class="token builtin class-name">cd</span> bangumi-rs</span>
<span class="line"></span></code></pre>
<div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0"><div class="line-number"></div><div class="line-number"></div></div></div><ol start="2">
<li>配置文件</li>
</ol>
<div class="language-bash line-numbers-mode" data-highlighter="prismjs" data-ext="sh"><pre v-pre><code><span class="line"><span class="token function">cp</span> config.example.toml config.toml</span>
<span class="line"></span></code></pre>
<div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0"><div class="line-number"></div></div></div><ol start="3">
<li>
<p>修改配置文件
根据你的需求修改 <code v-pre>config.toml</code> 文件，详细配置说明请参考<RouteLink to="/config/">配置文档</RouteLink>。</p>
</li>
<li>
<p>启动服务</p>
</li>
</ol>
<div class="language-bash line-numbers-mode" data-highlighter="prismjs" data-ext="sh"><pre v-pre><code><span class="line"><span class="token function">docker-compose</span> up <span class="token parameter variable">-d</span></span>
<span class="line"></span></code></pre>
<div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0"><div class="line-number"></div></div></div><ol start="5">
<li>访问服务
打开浏览器访问 http://localhost:80</li>
</ol>
<h2 id="手动部署" tabindex="-1"><a class="header-anchor" href="#手动部署"><span>手动部署</span></a></h2>
<h3 id="后端部署" tabindex="-1"><a class="header-anchor" href="#后端部署"><span>后端部署</span></a></h3>
<ol>
<li>安装 Rust</li>
</ol>
<div class="language-bash line-numbers-mode" data-highlighter="prismjs" data-ext="sh"><pre v-pre><code><span class="line"><span class="token function">curl</span> <span class="token parameter variable">--proto</span> <span class="token string">'=https'</span> <span class="token parameter variable">--tlsv1.2</span> <span class="token parameter variable">-sSf</span> https://sh.rustup.rs <span class="token operator">|</span> <span class="token function">sh</span></span>
<span class="line"></span></code></pre>
<div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0"><div class="line-number"></div></div></div><ol start="2">
<li>构建后端</li>
</ol>
<div class="language-bash line-numbers-mode" data-highlighter="prismjs" data-ext="sh"><pre v-pre><code><span class="line"><span class="token function">make</span> build-release  <span class="token comment"># 发布模式构建</span></span>
<span class="line"></span></code></pre>
<div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0"><div class="line-number"></div></div></div><ol start="3">
<li>运行后端</li>
</ol>
<div class="language-bash line-numbers-mode" data-highlighter="prismjs" data-ext="sh"><pre v-pre><code><span class="line">./target/release/bangumi-rs</span>
<span class="line"></span></code></pre>
<div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0"><div class="line-number"></div></div></div><h3 id="前端部署" tabindex="-1"><a class="header-anchor" href="#前端部署"><span>前端部署</span></a></h3>
<ol>
<li>安装依赖</li>
</ol>
<div class="language-bash line-numbers-mode" data-highlighter="prismjs" data-ext="sh"><pre v-pre><code><span class="line"><span class="token builtin class-name">cd</span> web</span>
<span class="line"><span class="token function">npm</span> <span class="token function">install</span></span>
<span class="line"></span></code></pre>
<div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0"><div class="line-number"></div><div class="line-number"></div></div></div><ol start="2">
<li>构建前端</li>
</ol>
<div class="language-bash line-numbers-mode" data-highlighter="prismjs" data-ext="sh"><pre v-pre><code><span class="line"><span class="token function">npm</span> run build</span>
<span class="line"></span></code></pre>
<div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0"><div class="line-number"></div></div></div><ol start="3">
<li>配置 Nginx</li>
</ol>
<div class="language-nginx line-numbers-mode" data-highlighter="prismjs" data-ext="nginx"><pre v-pre><code><span class="line"><span class="token directive"><span class="token keyword">server</span></span> <span class="token punctuation">{</span></span>
<span class="line">    <span class="token directive"><span class="token keyword">listen</span> <span class="token number">80</span></span><span class="token punctuation">;</span></span>
<span class="line">    <span class="token directive"><span class="token keyword">server_name</span> localhost</span><span class="token punctuation">;</span></span>
<span class="line"></span>
<span class="line">    <span class="token directive"><span class="token keyword">location</span> /</span> <span class="token punctuation">{</span></span>
<span class="line">        <span class="token directive"><span class="token keyword">root</span> /path/to/bangumi-rs/web/dist</span><span class="token punctuation">;</span></span>
<span class="line">        <span class="token directive"><span class="token keyword">try_files</span> <span class="token variable">$uri</span> <span class="token variable">$uri</span>/ /index.html</span><span class="token punctuation">;</span></span>
<span class="line">    <span class="token punctuation">}</span></span>
<span class="line"></span>
<span class="line">    <span class="token directive"><span class="token keyword">location</span> /api</span> <span class="token punctuation">{</span></span>
<span class="line">        <span class="token directive"><span class="token keyword">proxy_pass</span> http://localhost:3001</span><span class="token punctuation">;</span></span>
<span class="line">    <span class="token punctuation">}</span></span>
<span class="line"><span class="token punctuation">}</span></span>
<span class="line"></span></code></pre>
<div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h2 id="下一步" tabindex="-1"><a class="header-anchor" href="#下一步"><span>下一步</span></a></h2>
<ul>
<li>查看<RouteLink to="/config/">配置说明</RouteLink>了解如何配置系统</li>
<li>了解<RouteLink to="/guide/features.html">功能特性</RouteLink></li>
<li>参与<RouteLink to="/development/">项目开发</RouteLink></li>
</ul>
</div></template>


