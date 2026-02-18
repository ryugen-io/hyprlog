# hyprlog justfile

dev := "/home/ryu/code/.dev/scripts/shared"
local := "/home/ryu/code/.dev/scripts/specific/hyprlog"

default:
    @just --list

# === Build ===

build *args:
    @{{dev}}/build/build.sh --release {{args}}

build-debug *args:
    @{{dev}}/build/build.sh {{args}}

clean *args:
    @{{dev}}/build/clean.sh {{args}}

size:
    @{{dev}}/build/size.sh

bloat *args:
    @{{dev}}/build/bloat.sh {{args}}

# === Code Quality ===

fmt *args:
    @{{dev}}/code/fmt.sh {{args}}

lint *args:
    @{{dev}}/code/lint.sh {{args}}

todo:
    @{{dev}}/code/todo.sh

# === Dependencies ===

audit:
    @{{dev}}/deps/audit.sh

outdated:
    @{{dev}}/deps/outdated.sh

# === Testing ===

test:
    @{{dev}}/test/quick.sh

coverage:
    @{{dev}}/test/coverage.sh

bench *args:
    @{{dev}}/test/bench.sh {{args}}

fuzz *args:
    @{{dev}}/test/fuzz.sh {{args}}

test-app-detect:
    @{{local}}/test-app-detect.sh

# === Git ===

changes *args:
    @{{dev}}/git/changes.sh {{args}}

pre-commit:
    @{{dev}}/git/pre-commit.sh

# === Info ===

tree:
    @{{dev}}/info/tree.sh

loc:
    @{{dev}}/info/loc.sh

docs *args:
    @{{dev}}/info/docs.sh {{args}}

# === Install ===

install:
    @./install.sh
