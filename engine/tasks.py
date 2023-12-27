from invoke import Context, task

@task
def build(c: Context, release=False):
    c.run("cargo build" + (" --release" if release else ""), pty=True)