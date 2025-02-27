alter table subscriptions
    add enforce_torrent_release_after_broadcast bool default true not null;