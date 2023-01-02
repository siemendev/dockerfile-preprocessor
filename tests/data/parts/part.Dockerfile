INCLUDE subdir/Dockerfile WITH { "path": "/var/www" }
INCLUDE subdir/Dockerfile WITH { "path": "/var/log" } ONLY

RUN mkdir -p {{ plugin_folder }}