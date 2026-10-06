default: check

[private]
run-sub dir *recipe="":
    @if [ -n "${GITHUB_ACTIONS:-}" ]; then \
        echo "::group::{{dir}} {{recipe}}"; \
    elif [ -n "{{recipe}}" ]; then \
        printf "\n\033[1;36m━━━ [%s] just %s ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\033[0m\n" "{{dir}}" "{{recipe}}"; \
    else \
        printf "\n\033[1;36m━━━ [%s] ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\033[0m\n" "{{dir}}"; \
    fi
    @just -f {{dir}}/Justfile {{recipe}}
    @if [ -n "${GITHUB_ACTIONS:-}" ]; then echo "::endgroup::"; fi

check: check-nix-fmt check-md check-typos
    @just run-sub cad
    @just run-sub control-tools check
    @just run-sub firmware/lib/hoshiguma-api check
    @just run-sub firmware/lib/hoshiguma-common check
    @just run-sub firmware/lib/hoshiguma-state-machines check
    @just run-sub firmware/lib/hoshiguma-state-machines-test check
    @just run-sub firmware/cooler check
    @just run-sub firmware/hmi check
    @just run-sub firmware/orchestrator check
    @just run-sub firmware/rear-sensor-board check
    @just run-sub firmware/telemetry-bridge check

check-nix-fmt:
    alejandra -q -c .

check-md:
    git ls-files '*.md' | xargs -r mdl

check-typos:
    typos --force-exclude

fmt: fmt-nix

fmt-nix:
    alejandra -q .
