import { Contributors } from "/Users/lyqingye/workspace/me/bangumi/docs/site/node_modules/@vuepress/plugin-git/lib/client/components/Contributors.js";
import { Changelog } from "/Users/lyqingye/workspace/me/bangumi/docs/site/node_modules/@vuepress/plugin-git/lib/client/components/Changelog.js";

export default {
  enhance: ({ app }) => {
    app.component("GitContributors", Contributors);
    app.component("GitChangelog", Changelog);
  },
};
