# 元数据管理

元数据管理是 Bangumi-rs 的重要功能，它负责获取、存储和展示番剧的详细信息，为用户提供丰富的内容展示和管理体验。

## 元数据功能概述

Bangumi-rs 的元数据管理功能包括：

- 自动获取番剧信息
- 从多个数据源整合元数据
- 提供丰富的番剧展示
- 支持手动刷新和更新

## 数据来源

Bangumi-rs 支持从多个数据源获取元数据，确保信息的全面性和准确性：

### TMDB (The Movie Database)

[TMDB](https://www.themoviedb.org/) 是一个全球性的影视数据库，提供丰富的影视信息。

### Bangumi.tv

[Bangumi.tv](https://bgm.tv/) 是一个专注于动漫、游戏的中文社区和数据库。

### Mikan

[Mikan](https://mikanani.me/) 是一个动漫资源站点，也提供基本的番剧信息。

## 每周放送列表

用户可以切换每年的每个季度的放送列表，第一次切换需要手动点击右上角的刷新按钮，以加载放送列表和相关的番剧元数据

::: info 提示
每分钟只能请求刷新一次，由于加载元数据需要较长时间，需耐心等待
:::

如下图所示:

![home](/screenshot/home.png)

## 刷新特定番剧的元数据

如下图所示, 你可以在番剧封面中点击刷新按钮即可手动刷新番剧元数据:

![home](/screenshot/refresh-bangumi-metadata.png){width=200px}


## 手动匹配元数据

在某些情况下，番剧可能会识别错误，导致刮削的元数据有问题，此时用户可以手动搜索TMDB选择正确的番剧, 如下图所示.
![home](/screenshot/search-bangumi-tmdb.png){width=200px}

---

**此时用户就可以选择正确的番剧**

![home](/screenshot/search-tmdb.png){width=400px}

