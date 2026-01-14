# hyprlog justfile

default:
    @just --list

# === Build ===

build *args:
    @./dev/scripts/build/build.sh --release {{args}}

build-debug *args:
    @./dev/scripts/build/build.sh {{args}}

clean *args:
    @./dev/scripts/build/clean.sh {{args}}

size:
    @./dev/scripts/build/size.sh

bloat *args:
    @./dev/scripts/build/bloat.sh {{args}}

# === Code Quality ===

fmt *args:
    @./dev/scripts/code/fmt.sh {{args}}

lint *args:
    @./dev/scripts/code/lint.sh {{args}}

todo:
    @./dev/scripts/code/todo.sh

# === Dependencies ===

audit:
    @./dev/scripts/deps/audit.sh

outdated:
    @./dev/scripts/deps/outdated.sh

# === Testing ===

test:
    @./dev/scripts/test/quick.sh

coverage:
    @./dev/scripts/test/coverage.sh

# === Git ===

changes *args:
    @./dev/scripts/git/changes.sh {{args}}

pre-commit:
    @./dev/scripts/git/pre-commit.sh

# === Info ===

tree:
    @./dev/scripts/info/tree.sh

loc:
    @./dev/scripts/info/loc.sh

docs *args:
    @./dev/scripts/info/docs.sh {{args}}
