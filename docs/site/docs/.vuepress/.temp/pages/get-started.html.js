import comp from "/Users/lyqingye/workspace/me/bangumi/docs/site/docs/.vuepress/.temp/pages/get-started.html.vue"
const data = JSON.parse("{\"path\":\"/get-started.html\",\"title\":\"Get Started\",\"lang\":\"zh-CN\",\"frontmatter\":{},\"headers\":[{\"level\":2,\"title\":\"Pages\",\"slug\":\"pages\",\"link\":\"#pages\",\"children\":[]},{\"level\":2,\"title\":\"Content\",\"slug\":\"content\",\"link\":\"#content\",\"children\":[]},{\"level\":2,\"title\":\"Configuration\",\"slug\":\"configuration\",\"link\":\"#configuration\",\"children\":[]},{\"level\":2,\"title\":\"Layouts and customization\",\"slug\":\"layouts-and-customization\",\"link\":\"#layouts-and-customization\",\"children\":[]}],\"git\":{\"updatedTime\":1741281966000,\"contributors\":[{\"name\":\"lyqingye\",\"username\":\"lyqingye\",\"email\":\"lyqingye@users.noreply.github.com\",\"commits\":1,\"url\":\"https://github.com/lyqingye\"}],\"changelog\":[{\"hash\":\"62356fb21f69ac5e76e96a3bef57b253c67c24fa\",\"date\":1741281966000,\"email\":\"lyqingye@users.noreply.github.com\",\"author\":\"lyqingye\",\"message\":\"checkpoint\"}]},\"filePathRelative\":\"get-started.md\"}")
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
