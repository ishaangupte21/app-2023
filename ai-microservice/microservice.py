from dotenv import load_dotenv
import os
from flask import Flask, jsonify, request
from langchain.chat_models import ChatOpenAI
from langchain.prompts import PromptTemplate
from langchain.agents import initialize_agent, load_tools

load_dotenv()
MICROSERVICE_PORT = os.getenv("PORT")
OPENAI_KEY = os.getenv("OPENAI_API_KEY")

llm = ChatOpenAI(openai_api_key=OPENAI_KEY, model_name="gpt-4")

app = Flask(__name__)


@app.route("/", methods=["GET"])
def microservice_root():
    prompt = PromptTemplate.from_template(
        "When is the {cycle} deadline for {college_name}")
    prompt = prompt.format(cycle="early", college_name="Columbia University")
    resp = llm.predict(prompt)
    return resp, 200


@app.route("/get-general-info", methods=["POST"])
def get_general_info_route():
    req_body = request.json
    prompt = PromptTemplate.from_template(
        "Get the admissions URL, application URL, and financial aid URL in this html text: {input}")
    prompt = prompt.format(input=req_body['html_input'])
    resp = llm.predict(prompt)
    return resp, 200


@app.route("/get-application-statistics", methods=["POST"])
def get_application_info_route():
    req_body = request.json
    prompt = PromptTemplate.from_template(
        "Get the total number of applicants, total number of male applicants, total number of female applicants, total percent overall admitted, total percent of males admitted, total percent of females admitted, median SAT Evidence-Based Reading and Writing score, median SAT Math score, median ACT composite score in json with snake case names from this html text: {input}. No code. Just get the values. No text either. Just give me a JSON object. The json tags should be 'total_applicants', 'total_male_applicants', 'total_female_applicants', 'total_percent_admitted', 'total_percent_males_admitted', 'total_percent_females_admitted', 'sat_avg_english', 'sat_avg_math', 'act_avg'. Provide each field as a string.")
    prompt = prompt.format(input=req_body['input'])
    resp = llm.predict(prompt)
    print(resp)
    return resp, 200


@app.route("/get-application-requirements", methods=["POST"])
def get_application_reqs():
    req_body = request.json
    prompt = PromptTemplate.from_template(
        "List {name}'s application requirements as a Json Array of just each requirement string, no objects.")
    prompt = prompt.format(name=req_body['name'])
    resp = llm.predict(prompt)
    print(resp)
    return resp, 200


@app.route("/get-how-reviewed", methods=["POST"])
def get_how_reviewed():
    req_body = request.json
    prompt = PromptTemplate.from_template(
        "How does {name} review applications?")
    prompt = prompt.format(name=req_body['name'])
    resp = llm.predict(prompt)
    print(resp)
    return resp, 200


@app.route("/ask-question", methods=['POST'])
def ask_question():
    req_body = request.json
    prompt = req_body['question']
    resp = llm.predict(prompt)
    print(resp)
    return resp, 200


if __name__ == "__main__":
    app.run(host="0.0.0.0", port=MICROSERVICE_PORT)
