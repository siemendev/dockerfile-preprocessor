FROM ubuntu:{{ ubuntu_version }}
ENV DEBIAN_FRONTEND noninteractive

RUN apt-get install -y nginx

BLOCK content
    RUN echo "this by default"
ENDBLOCK

EXPOSE 8000