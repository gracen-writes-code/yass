from invoke import Context, task

@task
def build(c: Context, profile="dev"):
    c.run("cargo build --profile " + profile, pty=True)