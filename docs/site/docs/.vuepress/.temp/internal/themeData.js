export const themeData = JSON.parse("{\"logo\":\"/images/home.png\",\"navbar\":[{\"text\":\"指南\",\"link\":\"/guide/\"},{\"text\":\"配置\",\"link\":\"/config/\"},{\"text\":\"开发\",\"link\":\"/development/\"},{\"text\":\"GitHub\",\"link\":\"https://github.com/lyqingye/bangumi-rs\"}],\"sidebar\":{\"/guide/\":[{\"text\":\"指南\",\"children\":[\"/guide/README.md\",\"/guide/getting-started.md\"]}],\"/config/\":[{\"text\":\"配置\",\"children\":[\"/config/README.md\"]}],\"/development/\":[{\"text\":\"开发\",\"children\":[\"/development/README.md\"]}]},\"editLink\":false,\"lastUpdated\":true,\"contributors\":false,\"locales\":{\"/\":{\"selectLanguageName\":\"English\"}},\"colorMode\":\"auto\",\"colorModeSwitch\":true,\"repo\":null,\"selectLanguageText\":\"Languages\",\"selectLanguageAriaLabel\":\"Select language\",\"sidebarDepth\":2,\"editLinkText\":\"Edit this page\",\"lastUpdatedText\":\"Last Updated\",\"contributorsText\":\"Contributors\",\"notFound\":[\"There's nothing here.\",\"How did we get here?\",\"That's a Four-Oh-Four.\",\"Looks like we've got some broken links.\"],\"backToHome\":\"Take me home\",\"openInNewWindow\":\"open in new window\",\"toggleColorMode\":\"toggle color mode\",\"toggleSidebar\":\"toggle sidebar\"}")

if (import.meta.webpackHot) {
  import.meta.webpackHot.accept()
  if (__VUE_HMR_RUNTIME__.updateThemeData) {
    __VUE_HMR_RUNTIME__.updateThemeData(themeData)
  }
}

if (import.meta.hot) {
  import.meta.hot.accept(({ themeData }) => {
    __VUE_HMR_RUNTIME__.updateThemeData(themeData)
  })
}
