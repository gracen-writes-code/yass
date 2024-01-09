@default:
    just --list

build profile="dev":
    just engine/build {{profile}}

clean:
    just engine/clean

debug:
    engine/target/debug/engine debug_profile base_modules

release:
    -[ -e release ] && rm -r release

    just engine/release