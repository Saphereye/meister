#!/usr/bin/bash

docker exec -it $ID /opt/kafka/bin/kafka-console-producer.sh --broker-list localhost:9092 --topic $TOPIC
