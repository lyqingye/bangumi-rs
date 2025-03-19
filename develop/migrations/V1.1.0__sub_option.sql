alter table subscriptions
    add preferred_downloader varchar(255) null;

alter table subscriptions
    add allow_fallback bool default true not null;

alter table torrent_download_tasks
    add allow_fallback bool default true not null;