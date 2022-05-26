import logging
import os
from azure.eventhub import EventHubConsumerClient

connection_str = os.environ.get("AZURE_EVENT_HUB_CONNECTION_STRING")
consumer_group = os.environ.get("AZURE_EVENT_HUB_CONSUMER_NAME")
eventhub_name = os.environ.get("AZURE_EVENT_HUB_NAME")
client = EventHubConsumerClient.from_connection_string(connection_str, consumer_group, eventhub_name=eventhub_name)

logger = logging.getLogger("azure.eventhub")
logging.basicConfig(level=logging.INFO)

def on_event(partition_context, event):
    logger.info("Received event from partition {}".format(partition_context.partition_id))
    partition_context.update_checkpoint(event)

with client:
    client.receive(
        on_event=on_event,
        starting_position="-1",  # "-1" is from the beginning of the partition.
    )
    # receive events from specified partition:
    # client.receive(on_event=on_event, partition_id='0')