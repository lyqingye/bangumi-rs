<template><div><h1 id="配置说明" tabindex="-1"><a class="header-anchor" href="#配置说明"><span>配置说明</span></a></h1>
<p>Bangumi-rs 使用 TOML 格式的配置文件，默认配置文件路径为 <code v-pre>config.toml</code>。本节将详细介绍各个配置项的含义和用法。</p>
<h2 id="配置文件结构" tabindex="-1"><a class="header-anchor" href="#配置文件结构"><span>配置文件结构</span></a></h2>
<p>配置文件主要包含以下几个部分：</p>
<ul>
<li><RouteLink to="/config/server.html">服务器配置</RouteLink></li>
<li><RouteLink to="/config/proxy.html">代理配置</RouteLink></li>
<li><RouteLink to="/config/api.html">API 配置</RouteLink></li>
<li><RouteLink to="/config/downloader.html">下载器配置</RouteLink></li>
<li><RouteLink to="/config/notify.html">通知配置</RouteLink></li>
<li><RouteLink to="/config/parser.html">解析器配置</RouteLink></li>
</ul>
<h2 id="配置示例" tabindex="-1"><a class="header-anchor" href="#配置示例"><span>配置示例</span></a></h2>
<div class="language-toml line-numbers-mode" data-highlighter="prismjs" data-ext="toml"><pre v-pre><code><span class="line"><span class="token punctuation">[</span><span class="token table class-name">server</span><span class="token punctuation">]</span></span>
<span class="line"><span class="token key property">listen_addr</span> <span class="token punctuation">=</span> <span class="token string">"127.0.0.1:3001"</span>  <span class="token comment"># 服务监听地址</span></span>
<span class="line"><span class="token key property">database_url</span> <span class="token punctuation">=</span> <span class="token string">"mysql://user:pass@localhost:3306/bangumi"</span>  <span class="token comment"># 数据库连接 URL</span></span>
<span class="line"><span class="token key property">assets_path</span> <span class="token punctuation">=</span> <span class="token string">"assets"</span>          <span class="token comment"># 资源文件路径</span></span>
<span class="line"></span>
<span class="line"><span class="token punctuation">[</span><span class="token table class-name">log</span><span class="token punctuation">]</span></span>
<span class="line"><span class="token key property">level</span> <span class="token punctuation">=</span> <span class="token string">"debug"</span>                 <span class="token comment"># 日志级别</span></span>
<span class="line"></span>
<span class="line"><span class="token punctuation">[</span><span class="token table class-name">proxy</span><span class="token punctuation">]</span></span>
<span class="line"><span class="token key property">enabled</span> <span class="token punctuation">=</span> <span class="token boolean">false</span></span>
<span class="line"><span class="token key property">http</span> <span class="token punctuation">=</span> <span class="token string">"http://127.0.0.1:7890"</span></span>
<span class="line"><span class="token key property">https</span> <span class="token punctuation">=</span> <span class="token string">"http://127.0.0.1:7890"</span></span>
<span class="line"></span>
<span class="line"><span class="token comment"># 更多配置项请参考各个配置章节</span></span>
<span class="line"></span></code></pre>
<div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h2 id="配置优先级" tabindex="-1"><a class="header-anchor" href="#配置优先级"><span>配置优先级</span></a></h2>
<ol>
<li>命令行参数</li>
<li>环境变量</li>
<li>配置文件</li>
<li>默认值</li>
</ol>
<h2 id="环境变量" tabindex="-1"><a class="header-anchor" href="#环境变量"><span>环境变量</span></a></h2>
<p>所有配置项都可以通过环境变量进行覆盖，环境变量名称规则为：将配置项路径中的 <code v-pre>.</code> 替换为 <code v-pre>_</code>，并转换为大写。例如：</p>
<ul>
<li><code v-pre>server.listen_addr</code> -&gt; <code v-pre>SERVER_LISTEN_ADDR</code></li>
<li><code v-pre>proxy.http</code> -&gt; <code v-pre>PROXY_HTTP</code></li>
</ul>
<h2 id="配置验证" tabindex="-1"><a class="header-anchor" href="#配置验证"><span>配置验证</span></a></h2>
<p>启动时，系统会自动验证配置文件的正确性。如果配置有误，会输出详细的错误信息。</p>
</div></template>


