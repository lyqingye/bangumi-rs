alter table torrent_download_tasks
    modify download_status enum ('pending', 'downloading', 'completed', 'failed', 'retrying', 'cancelled', 'paused') not null 