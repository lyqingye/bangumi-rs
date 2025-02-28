alter table bangumi
    drop key uk_tmdb_id;

create index uk_tmdb_id
    on bangumi (tmdb_id);