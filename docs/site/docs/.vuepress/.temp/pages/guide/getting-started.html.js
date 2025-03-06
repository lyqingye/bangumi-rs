import comp from "/Users/lyqingye/workspace/me/bangumi/docs/site/docs/.vuepress/.temp/pages/guide/getting-started.html.vue"
const data = JSON.parse("{\"path\":\"/guide/getting-started.html\",\"title\":\"快速开始\",\"lang\":\"zh-CN\",\"frontmatter\":{},\"headers\":[{\"level\":2,\"title\":\"环境要求\",\"slug\":\"环境要求\",\"link\":\"#环境要求\",\"children\":[]},{\"level\":2,\"title\":\"Docker 部署（推荐）\",\"slug\":\"docker-部署-推荐\",\"link\":\"#docker-部署-推荐\",\"children\":[]},{\"level\":2,\"title\":\"手动部署\",\"slug\":\"手动部署\",\"link\":\"#手动部署\",\"children\":[{\"level\":3,\"title\":\"后端部署\",\"slug\":\"后端部署\",\"link\":\"#后端部署\",\"children\":[]},{\"level\":3,\"title\":\"前端部署\",\"slug\":\"前端部署\",\"link\":\"#前端部署\",\"children\":[]}]},{\"level\":2,\"title\":\"下一步\",\"slug\":\"下一步\",\"link\":\"#下一步\",\"children\":[]}],\"git\":{},\"filePathRelative\":\"guide/getting-started.md\"}")
export { comp, data }

if (import.meta.webpackHot) {
  import.meta.webpackHot.accept()
  if (__VUE_HMR_RUNTIME__.updatePageData) {
    __VUE_HMR_RUNTIME__.updatePageData(data)
  }
}

if (import.meta.hot) {
  import.meta.hot.accept(({ data }) => {
    __VUE_HMR_RUNTIME__.updatePageData(data)
  })
}
