# Python Built-Ins:
from dataclasses import dataclass
import logging
import os

logging.getLogger().setLevel(logging.DEBUG)
logger = logging.getLogger("handler")

# External Dependencies:
from langchain.schema import StrOutputParser
from langchain_aws import ChatBedrock
from opentelemetry.instrumentation.langchain import LangchainInstrumentor

LangchainInstrumentor().instrument()

BEDROCK_MODEL_ID = os.environ.get(
    "BEDROCK_MODEL_ID", "anthropic.claude-3-5-sonnet-20240620-v1:0"
)

llm = ChatBedrock(model_id=BEDROCK_MODEL_ID)


@dataclass
class EventData:
    input_text: str

    @classmethod
    def parse(cls, raw: dict) -> "EventData":
        try:
            input_text = raw["inputText"]
        except KeyError as ke:
            raise ValueError(f"Input event missing required field {ke}") from ke
        return cls(input_text=input_text)


def handler(event: dict, context):
    print("Got event")
    logger.info("Got event")
    evt = EventData.parse(event)

    chain = llm | StrOutputParser()
    reply = chain.invoke(evt.input_text)

    logger.info("Returning")
    return {
        "statusCode": 200,
        "body": reply,
        "headers": {
            "Content-Type": "text/plain",
        },
    }
