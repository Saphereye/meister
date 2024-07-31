from kafka import KafkaProducer, KafkaConsumer
from pprint import pprint
import re
import sys
from flask import Flask, jsonify
import threading
import json
from random import randint

MICROSERVICE_NAME = sys.argv[1]
app = Flask(MICROSERVICE_NAME)

producer = KafkaProducer(bootstrap_servers='localhost:9092', api_version=(0, 10), value_serializer=lambda v: json.dumps(v).encode('utf-8'))
consumer = KafkaConsumer('frommanager', bootstrap_servers='localhost:9092', api_version=(0, 10), value_deserializer=lambda m: m.decode('utf-8'))

def send_message(function: str, id: int | None = None):
    producer.send('tomanager', f'(uuid: "{randint(0, 9999999) if id is None else id}",name: "user_registration",version: Some("v0.1.0"),process: Process(service: "{MICROSERVICE_NAME}",function: "{function}"),status: Success,schema: "v0.1.0",data: "adarsh")')

# Flask routes for CRUD operations
@app.get('/create')
def create():
    send_message('create')
    return jsonify({"status": "created"}), 201

@app.get('/update')
def update():
    send_message('update')
    return jsonify({"status": "updated"}), 200

@app.get('/delete')
def delete():
    send_message('delete')
    return jsonify({"status": "deleted"}), 200

# Function to run Kafka consumer
def run_kafka_consumer():
    for msg in consumer:
        p = re.compile(r'(\w+):\"(\w+)\"')
        data = dict(p.findall(msg.value))
        print(f'{MICROSERVICE_NAME}: {data}')

        if data.get('service') != MICROSERVICE_NAME:
            continue
        print("It's show time")

        print(f'{MICROSERVICE_NAME}: ', end='')
        match data['function']:
            case 'create':
                send_message('create', data['uuid'])
                print("Create function called")
            case 'update':
                send_message('update', data['uuid'])
                print("Update function called")
            case 'delete':
                send_message('delete', data['uuid'])
                print("Delete function called")
            case _:
                print("Unknown function")

# Function to run Flask app
def run_flask_app():
    port = 0 if (len(sys.argv) < 2) else int(sys.argv[2])
    app.run(host='0.0.0.0', port=port)

# Create threads for Kafka consumer and Flask app
kafka_thread = threading.Thread(target=run_kafka_consumer)
flask_thread = threading.Thread(target=run_flask_app)

# Start the threads
kafka_thread.start()
flask_thread.start()

# Join the threads to the main thread
kafka_thread.join()
flask_thread.join()