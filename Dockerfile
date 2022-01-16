FROM ubuntu:20.04

RUN \
	apt-get update && \
	apt-get install ca-certificates curl -y && \
	apt-get clean

COPY tea-camellia /usr/local/bin/

EXPOSE 9944
EXPOSE 9933

CMD ["tea-camellia", "--dev", "--ws-external"]
