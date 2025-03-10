alter table torrents
    add source enum ('Mikan', 'AcgripOrg', 'NyaaLand', 'DmhyOrg', 'User') not null default 'Mikan';