from invoke import Context, task

@task
def build(c: Context, profile="dev"):
    c.run("cargo build --profile " + profile, pty=True)
    
    
@task
def clean(c: Context):
    c.run("cargo clean")
    
@task
def release(c: Context):
    