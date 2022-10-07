#!/usr/bin/env python3

import csv
from getpass import getpass
from pathlib import Path
import sys
from typing import Dict, Optional
import json

import click
import requests
import urllib
from attrs import define

import urllib3

urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)  # type: ignore


@define
class Api:
    hostname: str
    port: int
    api_root: str = ""
    tls: bool = False
    headers: Dict[str, str] = {}
    verify: bool = False

    def __attrs_post_init__(self) -> None:
        self.headers: Dict[str, str] = {
            "accept": "application/json",
        }

    @property
    def server_url(self) -> str:
        api_root = self.api_root.strip("/")
        if api_root:
            api_root += "/"
        return f"http{'s' if self.tls else ''}://{self.hostname}:{self.port}/{api_root}"

    def login(self, email, password):
        success, resp_json = self.post(
            "login",
            json={"email": email, "password": password},
        )
        if not success or not "access_token" in resp_json or "error" in resp_json:
            raise Exception(f"authentication failed: {resp_json}")
        access_token = resp_json["access_token"]
        self.headers["Authorization"] = "Bearer " + access_token

    def get(self, method, params=None, **kwargs):
        r = requests.get(
            self.server_url + method,
            params=params,
            headers=self.headers,
            verify=self.verify,
            **kwargs,
        )
        success = True
        if r.status_code != 200:
            success = False
            print(f"ERROR [{r.status_code}]: {r}")
        return success, r.json()

    def post(self, method, data=None, json=None, **kwargs):
        r = requests.post(
            self.server_url + method,
            data=data,
            json=json,
            headers=self.headers,
            **kwargs,
            verify=self.verify,
        )
        success = True
        if r.status_code != 200:
            success = False
            print(f"ERROR [{r.status_code}]: {r}")
        return success, r.json()

    def get_user_profile(self):
        return self.get("user_profile")

    def add_user(self, email, password):
        return self.post(
            "register",
            json={"email": email, "password": password},
        )

    def submit(self, json):
        return self.post("submit", json=json)


@click.group()
@click.pass_context
@click.option("--server", type=str, default="127.0.0.1")
@click.option("--port", type=int, default=4000)
@click.option("--tls", default=True)
def cli(ctx, server, port, tls):
    ctx.ensure_object(dict)
    ctx.obj["api"] = Api(hostname=server, port=port, tls=tls)


@click.command()
@click.option("--username", prompt="Enter username", help="user name")
@click.option("--password", help="password", default=None)
@click.option("--submission-id", required=True)
@click.option("--submission-name", required=True)
@click.option("--submission-category", default="HW:LWC:NIST:finalist")
@click.option("--timing-results", type=Path)
@click.option("--synthesis-settings", type=Path)
@click.option("--synthesis-results", type=Path)
@click.pass_context
def submit(
    ctx,
    username,
    password,
    submission_id,
    submission_name,
    submission_category,
    timing_results,
    synthesis_settings,
    synthesis_results,
):
    api: Api = ctx.obj["api"]
    if not password:
        password = getpass("Enter admin password")
    api.login(username, password=password)

    metadata = {}

    if synthesis_settings:
        with open(synthesis_settings) as f:
            synthesis_settings = json.load(f)
        metadata = {**metadata, **synthesis_settings}

    if synthesis_results:
        with open(synthesis_results) as f:
            synthesis_results = json.load(f)

    if timing_results:
        with open(timing_results, encoding="utf-8") as f:
            csv_reader = csv.DictReader(f)
            timing_results = list(csv_reader)

    data = {
        "id": submission_id,  # must be unique, no spaces, use lower-case letters, numbers and and under-score,
        "name": submission_name,  #
        "category": submission_category,
        "metadata": metadata,
        "timing": timing_results,
        "synthesis": synthesis_results,
    }

    success, r = api.submit(data)

    if success:
        print("results submitted: ", r)
    else:
        sys.exit(f"operation failed: {r}")


@click.command()
@click.option("--username", prompt="Enter username", required=True)
@click.option("--password", prompt="Enter password", hide_input=True, required=True)
@click.option("--output", type=Path, default="all_data.json")
@click.pass_context
def retrieve(ctx, username, password, output):
    api: Api = ctx.obj["api"]

    api.login(username, password)

    params = {}
    # params = dict(filter="id = 'asco'", limit=10000, offset=0)

    success, r = api.get("retrieve", params=params)

    if success:
        with open(output, "w") as f:
            json.dump(r, f)
        print(f"results written to {output}")
    else:
        sys.exit(f"operation failed: {r}")


@click.command("adduser")
@click.argument("username")
@click.option("--admin-password", prompt="Enter admin password", hide_input=True, required=True)
@click.pass_context
def add_user(ctx, username, admin_password):
    api: Api = ctx.obj["api"]

    api.login(email="admin", password=admin_password)

    user_pass = getpass(f"Enter password for new user {username}: ")

    success, r = api.add_user(username, user_pass)

    if success:
        print("user added", r)
    else:
        sys.exit(f"operation failed: {r}")


cli.add_command(submit)
cli.add_command(add_user)
cli.add_command(retrieve)

if __name__ == "__main__":
    cli(auto_envvar_prefix="CERGDB")  # type: ignore
