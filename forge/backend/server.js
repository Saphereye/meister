const express = require("express");
const { Kafka } = require("kafkajs");
const cors = require("cors");

const app = express();
const port = 5000; // You can change this port if needed

// Middleware
app.use(cors());
app.use(express.raw({ type: "application/octet-stream" }));

// Kafka setup
const kafka = new Kafka({
  clientId: "my-app",
  brokers: ["localhost:9092"], // Adjust the broker address as needed
});

const producer = kafka.producer();

// Endpoint to handle Kafka messages
app.post("/send-to-kafka", async (req, res) => {
  const message = req.body.toString(); // Convert raw buffer to string

  try {
    await producer.connect();
    await producer.send({
      topic: "tomanageredits", // Specify the correct topic here
      messages: [{ value: message }],
    });

    console.log("Message sent to Kafka successfully");
    res.status(200).send("Message sent to Kafka successfully");
  } catch (error) {
    console.error("Error sending message to Kafka:", error);
    res.status(500).send("Failed to send message to Kafka");
  } finally {
    await producer.disconnect();
  }
});

// Start the server
app.listen(port, () => {
  console.log(`Server is running on http://localhost:${port}`);
});
