from invoke import Context, task
from pathlib import Path

import engine.tasks as engine


@task
def build(c: Context, profile="dev"):
    with c.cd("engine"):
        engine.build(c, profile)
        
@task(build)
def move_debug_bin(c: Context):
    if Path("engine.dbg").exists():
        c.run("rm engine.dbg")
        
    c.run("mv engine/target/debug/engine engine.dbg")


@task(build, move_debug_bin)
def debug(c: Context):
    c.run("./engine.dbg game run", pty=True)