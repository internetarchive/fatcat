FROM docker.elastic.co/elasticsearch/elasticsearch:6.4.2

RUN /usr/share/elasticsearch/bin/elasticsearch-plugin install --batch analysis-icu

# This part doesn't work because elastic isn't actually running at this point
# in boot. Don't know enough Docker to figure out how to make this work at
# build time.
#COPY --chown=elasticsearch:elasticsearch release_schema.json /release_schema.json
#RUN curl http://127.0.0.1:9200/fatcat --upload-file /release_schema.json
