FROM rust

RUN apt update
RUN apt-get update
RUN apt upgrade -y
RUN apt-get upgrade -y
RUN apt-get install -y lsb-release -y

# --- POSTGRES 
ARG POSTGRES_VERSION=12
# https://www.postgresql.org/download/linux/ubuntu/
RUN echo "deb http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" >> /etc/apt/sources.list.d/pgdg.list
RUN curl -fsSL https://www.postgresql.org/media/keys/ACCC4CF8.asc | apt-key add
RUN apt-get update
# make sure postgresql install is propmpt free
ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get install -y postgresql-${POSTGRES_VERSION}

# --- ADD POSTGRES CONFIG
WORKDIR /etc/postgresql/${POSTGRES_VERSION}/main
RUN echo "listen_addresses = '*'" >> postgresql.conf
COPY postgres_config/pg_hba.conf .

RUN service postgresql start

WORKDIR /usr/src
RUN cargo install sqlx-cli --no-default-features --features postgres

WORKDIR /usr/src/forcerebuild2
WORKDIR /usr/src
RUN git clone -b explore-aysnc-graphql-errors https://github.com/openmsupply/remote-server

WORKDIR /usr/src/init
COPY init.sh .
RUN ./init.sh

# Should be at the top, but didn't want to rebuild it again (cargo build takes ages)
RUN apt-get install nginx -y
COPY nginx_config/remote-server.conf /etc/nginx/conf.d/

WORKDIR /usr/src/entry
COPY entrypoint.sh .
WORKDIR /usr/src/remote-server
ENTRYPOINT /usr/src/entry/entrypoint.sh

EXPOSE 8001
