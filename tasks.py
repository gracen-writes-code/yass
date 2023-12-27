from invoke import Context, task

import engine.tasks as engine


@task
def build(c: Context, release=False):
    with c.cd("engine"):
        engine.build(c, release)


@task(build)
def run(c: Context):
    c.run("./engine.x86_64 game run", pty=True)
