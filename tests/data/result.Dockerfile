FROM ubuntu:latest
ENV DEBIAN_FRONTEND noninteractive
RUN apt-get install -y nginx
RUN echo "this by default"
COPY parts/subdir/test.txt /var/www
COPY parts/subdir/test.txt /var/log
RUN mkdir -p /var/cache/plugin
EXPOSE 8000
