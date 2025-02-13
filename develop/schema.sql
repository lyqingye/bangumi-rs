
create table if not exists bangumi (
    id int primary key auto_increment,
    name varchar(255) not null comment '动漫名称',
    description text comment '动漫简介',
    bangumi_tv_id int null comment 'bangumi.tv 动漫ID',
    tmdb_id bigint unsigned null comment 'TMDB 动漫ID',
    mikan_id int null comment 'mikan 动漫ID',

    air_date datetime null comment '播出日期',
    air_week int null comment '播出星期',
    rating float8 null comment '评分',

    created_at datetime not null default current_timestamp comment '创建时间',
    updated_at datetime not null default current_timestamp on update current_timestamp comment '更新时间',

    poster_image_url varchar(255) null comment '海报图片',
    backdrop_image_url varchar(255) null comment '背景图片',

    season_number bigint unsigned null comment '季数',
    ep_count int not null comment '总集数',
    ep_start_number int not null comment '开始集数' default 1,
    calendar_season varchar(50) null comment '放送季',

    UNIQUE KEY `uk_mikan_id` (`mikan_id`),
    UNIQUE KEY `uk_bangumi_tv_id` (`bangumi_tv_id`),
    UNIQUE KEY `uk_tmdb_id` (`tmdb_id`),
    KEY `idx_calendar_season` (`calendar_season`),
    KEY `idx_air_date` (`air_date` DESC)
)comment '用于记录番剧的信息';

create table if not exists subscriptions (
    bangumi_id int not null primary key comment '番剧ID',
    subscribe_status enum('none','subscribed', 'downloaded') not null comment '状态, subscribed: 已订阅, downloaded: 已下载',
    start_episode_number int null comment '开始订阅集数',
    resolution_filter varchar(255) null comment '分辨率过滤',
    language_filter varchar(255) null comment '语言过滤',
    release_group_filter varchar(255) null comment '发布组过滤',
    created_at datetime not null default current_timestamp comment '创建时间',
    updated_at datetime not null default current_timestamp on update current_timestamp comment '更新时间',
    collector_interval int null comment '种子收集间隔',
    metadata_interval int null comment '元数据刷新间隔',
    task_processor_interval int null comment '任务处理间隔',

    KEY `idx_subscribe_status` (`subscribe_status`)
)comment '订阅信息，用于记录订阅的信息';

create table if not exists episodes (
    id int primary key auto_increment,
    bangumi_id int not null comment '番剧ID',
    number int not null comment '集数',
    sort_number int null comment '排序后的集数',
    name varchar(255) null comment '集名称',
    image_url varchar(255) null comment '图片',
    description text null comment '集简介',
    air_date date null comment '播出日期',
    duration_seconds bigint unsigned null comment '时长',
    kind enum('EP', 'SP', 'OP','ED','MAD', 'Other') not null comment '集类型' default 'EP',
    created_at datetime not null default current_timestamp comment '创建时间',
    updated_at datetime not null default current_timestamp on update current_timestamp comment '更新时间',

    UNIQUE KEY `uk_bangumi_id_number` (`bangumi_id`,`number`)
)comment '用于记录番剧的每一集的信息';

create table if not exists episode_download_tasks (
    bangumi_id int not null comment '动漫ID',
    episode_number int not null comment '集数',
    state enum('missing','ready', 'downloading', 'downloaded', 'failed', 'retrying') not null comment '任务状态',
    ref_torrent_info_hash varchar(40) null comment '下载中会填写该字段',

    created_at datetime not null default current_timestamp comment '创建时间',
    updated_at datetime not null default current_timestamp on update current_timestamp comment '更新时间',
    primary key (bangumi_id, episode_number),
    key status_idx (state)
)comment '用于记录番剧的每一集的下载状态';

create table if not exists torrents (
    info_hash varchar(40) primary key not null comment '种子文件的info hash, 用于去重',
    bangumi_id int not null comment '动漫ID',
    title varchar(255) not null comment '标题',
    size bigint not null comment '文件大小',
    magnet TEXT not null comment '磁力链接',
    data MEDIUMBLOB null comment '种子文件',
    download_url varchar(255) null comment '种子下载地址',
    created_at datetime not null default current_timestamp comment '创建时间',
    updated_at datetime not null default current_timestamp on update current_timestamp comment '更新时间',
    key `torrent_bangumi_id_idx` (bangumi_id)
)comment '种子信息，用于记录种子的信息';

create table if not exists torrent_download_tasks (
    info_hash varchar(40) primary key not null comment '种子文件的info hash, 用于去重',
    download_status enum('pending', 'downloading', 'completed', 'failed', 'retrying') not null comment '状态, pending: 等待下载, downloading: 下载中, completed: 下载完成, failed: 下载失败, retrying: 重试中',
    downloader varchar(255) comment '下载器名称',
    dir varchar(255) not null comment '下载目录',
    context text comment '下载器上下文',
    err_msg text null comment '错误信息',
    retry_count int not null default 0 comment '重试次数',
    next_retry_at datetime not null comment '重试时间',
    created_at datetime not null default current_timestamp comment '创建时间',
    updated_at datetime not null default current_timestamp on update current_timestamp comment '更新时间'
)comment '种子下载任务，用于记录种子的下载状态';

create table if not exists file_name_parse_record (
    file_name varchar(255) not null primary key comment '文件名称',
    release_group varchar(255) null comment '',
    bangumi_name varchar(255) comment '动漫名称',
    season_number int null comment '季数' default 0,
    episode_number int null comment '集数' default 0,
    language varchar(255) null comment '语言' ,
    video_resolution varchar(255) null comment '分辨率',
    year int null,
    parser_name varchar(255) not null comment '解析器名称',
    parser_status enum('pending', 'completed', 'failed') not null comment '状态, pending: 等待解析，completed: 解析完成，failed: 解析失败',
    err_msg text null comment '错误信息',

    created_at datetime not null default current_timestamp comment '创建时间',
    updated_at datetime not null default current_timestamp on update current_timestamp comment '更新时间',
    KEY `idx_parser_status` (`parser_status`)
) comment '文件名解析记录，从文件名解析出番剧的信息, 用于文件规范命名';

CREATE TABLE dictionary (
    code VARCHAR(50) NOT NULL PRIMARY KEY,  -- 字典代码
    group_code VARCHAR(50) NOT NULL,  -- 字典分组
    value VARCHAR(100) NOT NULL, -- 字典名称
    sort_order INTEGER DEFAULT 0,  -- 排序
    description TEXT null  -- 描述
);

update bangumi set calendar_season = '2025 冬季番组';

-- update file_name_parse_record set parser_status = 'pending' where parser_status = 'failed';