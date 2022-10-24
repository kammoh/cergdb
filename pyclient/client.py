#!/usr/bin/env python3

import csv
import json
import sys
from getpass import getpass
from json import JSONDecodeError
from pathlib import Path
from typing import Dict, Optional

import click
import requests
import urllib3
from attrs import define
from dotenv import load_dotenv

try:
    import tomllib  # type: ignore # pyright: reportMissingImports=none
except ModuleNotFoundError:
    # python_version < "3.11":
    import tomli as tomllib  # type: ignore

load_dotenv()


@define
class Api:
    hostname: str
    port: int
    api_root: str = ""
    tls: bool = False
    headers: Dict[str, str] = {}
    verify: bool = True
    username: Optional[str] = None
    password: Optional[str] = None

    def __attrs_post_init__(self) -> None:
        self.headers: Dict[str, str] = {
            "accept": "application/json",
        }
        if not self.verify:
            urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)  # type: ignore

    @property
    def server_url(self) -> str:
        api_root = self.api_root.strip("/")
        if api_root:
            api_root += "/"
        return f"http{'s' if self.tls else ''}://{self.hostname}:{self.port}/{api_root}"

    def login(self, username=None, password=None):
        if username is None:
            username = self.username
        if password is None:
            password = self.password
        if password is None:
            password = getpass("Enter admin password")

        assert username is not None, "Username is required"
        assert password is not None, "Password is required"

        print(f"logging in {username}...")
        success, resp_json = self.post(
            "login",
            json={"email": username, "password": password},
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
        try:
            resp_json = r.json()
        except JSONDecodeError:
            resp_json = None
        return success, resp_json

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
        try:
            resp_json = r.json()
        except JSONDecodeError:
            resp_json = None
        return success, resp_json

    def get_user_profile(self):
        return self.get("user_profile")

    def add_user(self, email, password):
        return self.post(
            "register",
            json={"email": email, "password": password},
        )

    def submit(self, json):
        return self.post("submit", json=json)

    def delete(self, id):
        return self.post("delete", json={"id": id})


@click.group()
@click.pass_context
@click.option("--server", type=str, default="127.0.0.1")
@click.option("--port", type=int, default=4000)
@click.option("--tls/--no-tls", default=True)
@click.option(
    "--tls-verify/--no-tls-verify", default=True, help="Verify TLS certificate."
)
@click.option("--username", default=None)
@click.option("--password", default=None)
def cli(ctx, server, port, tls, tls_verify, username, password):
    ctx.ensure_object(dict)
    ctx.obj["api"] = Api(
        hostname=server,
        port=port,
        tls=tls,
        verify=tls_verify,
        username=username,
        password=password,
    )


@click.command()
@click.argument(
    "submission-id",
)
@click.option("--submission-name", required=False)
@click.option("--submission-category", default="HW:LWC:NIST:finalist")
@click.option("--timing-results", type=Path)
@click.option("--synthesis-settings", type=Path)
@click.option("--synthesis-results", type=Path)
@click.option("--design-toml", default=None, type=Path)
@click.pass_context
def submit(
    ctx,
    submission_id,
    submission_name,
    submission_category,
    timing_results,
    synthesis_settings,
    synthesis_results,
    design_toml,
):
    api: Api = ctx.obj["api"]
    api.login()

    metadata = {}

    if design_toml:
        with open(design_toml, "rb") as f:
            design_description = tomllib.load(f)
        metadata["design"] = design_description

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
@click.option("--output", type=Path, default="all_data.json")
@click.pass_context
def retrieve(ctx, output):
    api: Api = ctx.obj["api"]

    api.login()

    params = dict(
        fields=[
            "id",
            "timing",
            "metadata",
            "synthesis.best.results.design",
            "synthesis.best.results.tools",
            "synthesis.best.results.Fmax",
            "synthesis.best.results._hierarchical_utilization.LWC_SCA_wrapper.@children.INST_LWC.Total LUTs",
            "synthesis.best.results._hierarchical_utilization.LWC_SCA_wrapper.@children.INST_LWC.FFs",
            "synthesis.best.results.lut",
            "synthesis.best.results.ff",
            "synthesis.best.results.slice",
            "synthesis.best.results.latch",
            "synthesis.best.results.dsp",
            "synthesis.best.results.bram_tile",
        ],
        flatten=False,
    )

    success, r = api.post("retrieve", json=params)

    if success:
        with open(output, "w") as f:
            json.dump(r, f, indent=4)
        print(f"results written to {output}")
    else:
        sys.exit(f"operation failed: {r}")


@click.command()
@click.argument("id")
@click.pass_context
def delete(ctx, id):
    api: Api = ctx.obj["api"]

    api.login()

    success, r = api.delete(id)

    if success:
        print(f"deleted record: {r.get('id')}")
    else:
        sys.exit(f"operation failed: {r}")


@click.command("adduser")
@click.argument("username")
@click.option(
    "--admin-password", prompt="Enter admin password", hide_input=True, required=True
)
@click.pass_context
def add_user(ctx, username, admin_password):
    api: Api = ctx.obj["api"]

    api.login(username="admin", password=admin_password)

    user_pass = getpass(f"Enter password for new user {username}: ")

    success, r = api.add_user(username, user_pass)

    if success:
        print("user added", r)
    else:
        sys.exit(f"operation failed: {r}")


cli.add_command(submit)
cli.add_command(add_user)
cli.add_command(retrieve)
cli.add_command(delete)

if __name__ == "__main__":
    cli(auto_envvar_prefix="CERGDB")  # type: ignore
