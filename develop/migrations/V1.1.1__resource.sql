alter table torrent_download_tasks
    modify resource_type enum ('torrent', 'torrent_url', 'magnet', 'info_hash') default 'info_hash' not null;
    
alter table torrent_download_tasks
    add torrent_url text null;