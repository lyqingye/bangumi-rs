export const redirects = JSON.parse("{}")

export const routes = Object.fromEntries([
  ["/", { loader: () => import(/* webpackChunkName: "index.html" */"/Users/lyqingye/workspace/me/bangumi/docs/site/docs/.vuepress/.temp/pages/index.html.js"), meta: {"title":"Home"} }],
  ["/get-started.html", { loader: () => import(/* webpackChunkName: "get-started.html" */"/Users/lyqingye/workspace/me/bangumi/docs/site/docs/.vuepress/.temp/pages/get-started.html.js"), meta: {"title":"Get Started"} }],
  ["/config/", { loader: () => import(/* webpackChunkName: "config_index.html" */"/Users/lyqingye/workspace/me/bangumi/docs/site/docs/.vuepress/.temp/pages/config/index.html.js"), meta: {"title":"配置说明"} }],
  ["/development/", { loader: () => import(/* webpackChunkName: "development_index.html" */"/Users/lyqingye/workspace/me/bangumi/docs/site/docs/.vuepress/.temp/pages/development/index.html.js"), meta: {"title":"开发指南"} }],
  ["/guide/", { loader: () => import(/* webpackChunkName: "guide_index.html" */"/Users/lyqingye/workspace/me/bangumi/docs/site/docs/.vuepress/.temp/pages/guide/index.html.js"), meta: {"title":"介绍"} }],
  ["/guide/getting-started.html", { loader: () => import(/* webpackChunkName: "guide_getting-started.html" */"/Users/lyqingye/workspace/me/bangumi/docs/site/docs/.vuepress/.temp/pages/guide/getting-started.html.js"), meta: {"title":"快速开始"} }],
  ["/404.html", { loader: () => import(/* webpackChunkName: "404.html" */"/Users/lyqingye/workspace/me/bangumi/docs/site/docs/.vuepress/.temp/pages/404.html.js"), meta: {"title":""} }],
]);

if (import.meta.webpackHot) {
  import.meta.webpackHot.accept()
  if (__VUE_HMR_RUNTIME__.updateRoutes) {
    __VUE_HMR_RUNTIME__.updateRoutes(routes)
  }
  if (__VUE_HMR_RUNTIME__.updateRedirects) {
    __VUE_HMR_RUNTIME__.updateRedirects(redirects)
  }
}

if (import.meta.hot) {
  import.meta.hot.accept(({ routes, redirects }) => {
    __VUE_HMR_RUNTIME__.updateRoutes(routes)
    __VUE_HMR_RUNTIME__.updateRedirects(redirects)
  })
}
