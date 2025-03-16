alter table torrent_download_tasks
    add resource_type enum ('torrent', 'magnet', 'info_hash') default 'info_hash' not null;

alter table torrent_download_tasks
    add magnet TEXT null;