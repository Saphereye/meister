#!/usr/bin/bash

docker exec -it $ID /opt/kafka/bin/kafka-console-consumer.sh --bootstrap-server localhost:9092 --topic $TOPIC --from-beginning
