import comp from "/Users/lyqingye/workspace/me/bangumi/docs/site/docs/.vuepress/.temp/pages/config/index.html.vue"
const data = JSON.parse("{\"path\":\"/config/\",\"title\":\"配置说明\",\"lang\":\"zh-CN\",\"frontmatter\":{},\"headers\":[{\"level\":2,\"title\":\"配置文件结构\",\"slug\":\"配置文件结构\",\"link\":\"#配置文件结构\",\"children\":[]},{\"level\":2,\"title\":\"配置示例\",\"slug\":\"配置示例\",\"link\":\"#配置示例\",\"children\":[]},{\"level\":2,\"title\":\"配置优先级\",\"slug\":\"配置优先级\",\"link\":\"#配置优先级\",\"children\":[]},{\"level\":2,\"title\":\"环境变量\",\"slug\":\"环境变量\",\"link\":\"#环境变量\",\"children\":[]},{\"level\":2,\"title\":\"配置验证\",\"slug\":\"配置验证\",\"link\":\"#配置验证\",\"children\":[]}],\"git\":{},\"filePathRelative\":\"config/README.md\"}")
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
